#![doc = include_str!("../README.md")]

use std::sync::mpsc::{Sender, channel};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, read_dir},
    io::{self, Write},
    path::{Path, PathBuf},
};

use rayon::Scope;
use thiserror::Error;

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

fn main() -> Result<()> {
    let config = Cli::parse();

    let (tx, rx) = channel();
    rayon::scope(|scope| {
        scope.spawn(|s| {
            visit_dirs(&config.path, s, tx);
        });
    });

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_nix::LANGUAGE.into())
        .expect("error loading nix grammar");

    let mut map = <HashMap<String, ContextVec>>::new();
    while let Ok(entry) = rx.recv() {
        let entry = entry?;
        let source_code = fs::read_to_string(&entry)?;
        let cv = ContextVec::try_from_source(source_code, &mut parser, &config.needle, entry)?;
        for ctx in cv {
            map.entry(ctx.name.clone())
                .or_insert(ContextVec::new())
                .push(ctx);
        }
    }

    if map.is_empty() {
        writeln!(
            io::stdout(),
            "no results for `{}` in {}",
            config.needle,
            config.path.display()
        )?;
    }
    let ordered = {
        let mut v: Vec<_> = map.into_iter().collect();
        // most occurrences first
        v.sort_by(|(_, a), (_, b)| b.cmp(a));
        v
    };
    for (idx, (name, cv)) in ordered.into_iter().enumerate() {
        if idx > 0 {
            writeln!(io::stdout())?;
        }
        writeln!(io::stdout(), "{} ({})", name, cv.len())?;
        if config.examples > 0 {
            writeln!(io::stdout(), "=== EXAMPLES ===")?;
            for ctx in cv.iter().take(config.examples) {
                writeln!(io::stdout(), "{}:", ctx.path.display())?;
                writeln!(io::stdout(), "    {}", ctx.code)?;
            }
        }
    }
    Ok(())
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
