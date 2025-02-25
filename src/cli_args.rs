use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct CliArgs {
    #[arg(last=true)]
    pub default_command: Option<Vec<String>>,
}
