use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct CliArgs {
    #[arg(short, long, value_name = "DIR")]
    pub assets: Option<PathBuf>,

    #[arg(last = true)]
    pub default_command: Option<Vec<String>>,
}
