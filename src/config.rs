use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    pub(crate) needle: String,

    #[arg(default_value_os_t = PathBuf::from("."), help = "directory name to (recursively) search for nix files")]
    pub(crate) path: PathBuf,

    #[arg(
        short,
        long,
        default_value_t = 0,
        help = "number of examples to show per binding"
    )]
    pub(crate) examples: usize,
}

impl Cli {
    pub fn parse() -> Cli {
        <Self as Parser>::parse()
    }
}
