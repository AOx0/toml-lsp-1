use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(long, short)]
    pub parse: Option<PathBuf>,
}
