use std::io::{prelude::*, BufReader};

use anyhow::{anyhow, Result};
use app::AppSBI;
use cli_args::CliArgs;
use directories::ProjectDirs;
use flexi_logger::FileSpec;
use interprocess::local_socket::LocalSocketStream;

mod json;
mod instance;
mod ui;
mod cli_args;
mod app;
mod workshop_downloader;
mod mod_manifest;

static ORGANIZATION_QUALIFIER: &str = "com";
static ORGANIZATION_NAME: &str = "";
static APPLICATION_NAME: &str = "sbi";
static INSTANCE_JSON_NAME: &str = "instance.json";
static SBI_CONFIG_JSON_NAME: &str = "config.json";
static STARBOUND_STEAM_ID: &str = "211820";
static STARBOUND_BOOT_CONFIG_NAME: &str = "sbinit.config";
static LOCAL_PIPE_NAME: &str = "@SBI_PIPE_NAME";

fn main() -> Result<()> {
    let proj_dirs = ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME).ok_or(anyhow!("Can't find home directory"))?;
    let _log_handle = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
            .directory(proj_dirs.data_dir())
            .basename("sbi")
            .suppress_timestamp()
        )
        .rotate(flexi_logger::Criterion::Age(flexi_logger::Age::Second), flexi_logger::Naming::Numbers, flexi_logger::Cleanup::KeepLogFiles(3))
        .start()?;
    let cli_args: CliArgs = clap::Parser::try_parse()?;
    if cli_args.query {
        let local_socket = LocalSocketStream::connect(LOCAL_PIPE_NAME)?;
        let mut socket = BufReader::new(local_socket);
        let mut buffer = String::with_capacity(1024);
        socket.read_line(&mut buffer)?;
        let val1 = buffer.trim().to_string();
        buffer.clear();
        socket.read_line(&mut buffer)?;
        let val2 = buffer.trim().to_string();
        println!("{}:{}", val1, val2);
        // Get and return instance runtime info from named pipe
        return Ok(());
    }
    let tk_rt = tokio::runtime::Runtime::new()?;
    tk_rt.block_on(AppSBI::run(proj_dirs))
}
