use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct CliArgs {
    /// Start the application in query mode
    #[arg(short, long)]
    pub query: bool
}
