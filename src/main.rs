use anyhow::{anyhow, Result};
use app::AppSBI;
use cli_args::CliArgs;
use directories::ProjectDirs;
use flexi_logger::FileSpec;
use interprocess::local_socket::LocalSocketStream;
use log::{info, warn};

mod json;
mod instance;
mod ui;
mod cli_args;
mod app;
mod workshop_downloader;
mod game_launcher;
mod mod_manifest;
mod core;

static ORGANIZATION_QUALIFIER: &str = "com";
static ORGANIZATION_NAME: &str = "";
static APPLICATION_NAME: &str = "sbi";
static INSTANCE_JSON_NAME: &str = "instance.json";
static SBI_CONFIG_JSON_NAME: &str = "config.json";
static STARBOUND_STEAM_ID: &str = "211820";
static STARBOUND_BOOT_CONFIG_NAME: &str = "sbinit.config";
static LOCAL_PIPE_NAME: &str = "@SBI_PIPE_NAME";

#[tokio::main]
async fn main() -> Result<()> {
    let proj_dirs = ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME).ok_or(anyhow!("Can't find home directory"))?;
    let cli_args: CliArgs = clap::Parser::try_parse()?;
    let _log_handle = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
            .directory(proj_dirs.data_dir())
            .basename("sbi")
            .suppress_timestamp()
        )
        .rotate(flexi_logger::Criterion::Age(flexi_logger::Age::Second), flexi_logger::Naming::Numbers, flexi_logger::Cleanup::KeepLogFiles(3))
        .start()?;
    if cli_args.query {
        // flexi_logger::Logger::try_with_env_or_str("info")?.start()?;
        match LocalSocketStream::connect(LOCAL_PIPE_NAME) {
            Ok(local_socket) => {

                info!("Launching starbound through steam!");
                let launch_message: json::SBILaunchMessageJson = serde_json::from_reader(local_socket)?;

                let executable_path = launch_message.exececutable_path;
                let maybe_extra_ld_path = launch_message.ld_library_path.as_deref();
                let instance_path = launch_message.instance_path.unwrap_or_else(|| std::env::current_dir().expect("Current working directory cannot be read from"));
                game_launcher::launch_instance_cli(&executable_path, &instance_path, maybe_extra_ld_path)?;

            },
            Err(_) => {
                warn!("Could not connect to sbi client!");
                if let Some(default_command) = cli_args.default_command {
                    info!("Attempting to launch game through default command: {:?}", default_command);
                    game_launcher::launch_default(default_command)?;
                }
            }

        }
        return Ok(());
    }
    AppSBI::run(proj_dirs).await
}
