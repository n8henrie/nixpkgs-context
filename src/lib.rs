#![doc = include_str!("../README.md")]

use std::fmt;
use std::io::Write;
use std::sync::mpsc::{Sender, channel};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, read_dir},
    io::{self},
    path::{Path, PathBuf},
};

use rayon::Scope;
use tree_sitter::Parser;

mod config;
use config::Cli;

mod context;
use context::ContextVec;

mod error;
use error::{Error, Result};

#[allow(clippy::needless_pass_by_value)]
fn visit_dirs(dir: &Path, scope: &Scope, tx: Sender<io::Result<PathBuf>>) {
    match fs::symlink_metadata(dir) {
        Ok(md) if md.is_symlink() => {
            return;
        }
        Ok(_) => (),
        Err(e) => {
            tx.send(Err(e)).expect("couldn't send");
            return;
        }
    }
    if dir.is_dir() {
        let rd = match read_dir(dir) {
            Ok(r) => r,
            Err(e) => {
                tx.send(Err(e)).expect("couldn't send");
                return;
            }
        };
        for entry in rd {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tx.send(Err(e)).expect("couldn't send");
                    return;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                let tx = tx.clone();
                scope.spawn(move |s| {
                    visit_dirs(&path, s, tx);
                });
            } else if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("nix") {
                tx.send(Ok(path)).expect("couldn't send");
            }
        }
    }
}

#[derive(Debug)]
pub struct Report {
    config: Cli,
    data: Vec<(String, ContextVec)>,
    pub errors: Vec<Error>,
}

type CVMap = HashMap<String, ContextVec>;

impl From<(CVMap, Cli, Vec<Error>)> for Report {
    fn from(input: (CVMap, Cli, Vec<Error>)) -> Self {
        let (map, config, errors) = input;
        let mut data: Vec<_> = map.into_iter().collect();
        // most occurrences first
        data.sort_by(|(_, a), (_, b)| b.cmp(a));
        Self {
            config,
            data,
            errors,
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.is_empty() {
            writeln!(
                f,
                "no results for `{}` in {}",
                self.config.needle,
                self.config.path.display()
            )
        } else {
            for (idx, (name, cv)) in self.data.iter().enumerate() {
                if self.config.examples > 0 && idx > 0 {
                    writeln!(f)?;
                }
                writeln!(f, "{name} ({})", cv.len())?;
                if self.config.examples > 0 {
                    writeln!(f, "=== EXAMPLES ===")?;
                    for ctx in cv.iter().take(self.config.examples) {
                        writeln!(f, "{}:", ctx.path.display())?;
                        writeln!(f, "    {}", ctx.code)?;
                    }
                }
            }
            Ok(())
        }
    }
}

/// # Errors
///
/// Skips files or filesystems with errors (with a warning)
/// Returns an error if it can't write to stdout
pub fn run() -> Result<Report> {
    let config = Cli::parse();

    let (tx, rx) = channel();
    rayon::scope(|scope| {
        scope.spawn(|s| {
            visit_dirs(&config.path, s, tx);
        });
    });

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_nix::LANGUAGE.into())?;

    let mut map = CVMap::new();
    let mut errs = Vec::new();
    while let Ok(entry) = rx.recv() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errs.push(e.into());
                continue;
            }
        };
        let source_code = match fs::read_to_string(&entry) {
            Ok(source) => source,
            Err(e) => {
                errs.push(e.into());
                continue;
            }
        };
        let cv = match ContextVec::try_from_source(source_code, &mut parser, &config.needle, entry)
        {
            Err(err @ Error::Parse(_)) => {
                let mut stderr = std::io::stderr().lock();
                writeln!(stderr, "{err}")?;
                drop(stderr);
                continue;
            }
            Err(e) => return Err(e),
            Ok(cv) => cv,
        };
        for ctx in cv {
            map.entry(ctx.name.clone())
                .or_insert(ContextVec::new())
                .push(ctx);
        }
    }

    Ok((map, config, errs).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_callpackage_params() {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_nix::LANGUAGE.into())
            .expect("error loading nix grammar");
        let source = "{stdenv, foo, bar}: stdenv.mkDerivation { buildInputs = [ foo bar ]; }";
        let cv = ContextVec::try_from_source(source, &mut parser, "foo", "fakepath").unwrap();
        assert_eq!(cv.len(), 1);
        let ctx = cv.into_iter().next().unwrap();
        assert_eq!(ctx.name, "buildInputs");
        assert_eq!(ctx.code, "buildInputs = [ foo bar ];");
        assert_eq!(ctx.path.to_str().unwrap(), "fakepath");
    }
}
