// use std::sync::LazyLock;

// use anyhow::Result;
use application::{Application, Message};
use directories::ProjectDirs;
use flexi_logger::{FileSpec, FlexiLoggerError};
use iced::Task;

mod application;
mod cli_args;
// mod core;
mod game_launcher;
mod json;
mod mod_manifest;
mod profile;
// mod workshop_downloader;

static ORGANIZATION_QUALIFIER: &str = "";
static ORGANIZATION_NAME: &str = "";
static APPLICATION_NAME: &str = "sbi";

static PROFILE_JSON_NAME: &str = "profile.json";
static SBI_CONFIG_JSON_NAME: &str = "config.json";

static STARBOUND_STEAM_ID: &str = "211820";
static STARBOUND_BOOT_CONFIG_NAME: &str = "sbinit.config";

static LOCAL_PIPE_NAME: &str = "@SBI_PIPE_NAME";
static LOCAL_PIPE_FS_NAME: &str = "/tmp/@SBI_PIPE_NAME";

#[derive(Debug, thiserror::Error)]
enum SBIInitializationError {
    #[error("Could not find the relevant folders to store application data in")]
    NoProjectDirectories,
    #[error("...")]
    LoggerFailure(#[from] FlexiLoggerError),
    #[error("...")]
    IcedApplicationError(#[from] iced::Error),
}

fn main() -> Result<(), SBIInitializationError> {
    /*TODO: Introduce Environment Variables for application storage location*/
    let proj_dirs = ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME)
        .ok_or(SBIInitializationError::NoProjectDirectories)?;

    // let cli_args: CliArgs = clap::Parser::try_parse()?;
    let _log_handle = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(proj_dirs.data_dir())
                .basename("sbi")
                .suppress_timestamp(),
        )
        .start()?;

    let profiles_dir = proj_dirs.data_dir().join("profiles");
    iced::application("SBI", Application::update, Application::view)
        .theme(Application::theme)
        .run_with(move || {
            (
                Application::new(proj_dirs),
                Task::perform(
                    Application::find_profiles(profiles_dir),
                    Message::FetchedProfiles,
                ),
            )
        })?;
    Ok(())

    // if cli_args.query {
    //     // flexi_logger::Logger::try_with_env_or_str("info")?.start()?;
    //     let name = get_pipe_name()?;
    //     match interprocess::local_socket::tokio::Stream::connect(name).await {
    //         Ok(local_socket) => {
    //             connect_to_existing_sbi_service(local_socket).await?;
    //         },
    //         Err(_) => {
    //             warn!("Could not connect to sbi client!");
    //             if let Some(default_command) = cli_args.default_command {
    //                 info!("Attempting to launch game through default command: {:?}", default_command);
    //                 game_launcher::launch_default(default_command)?;
    //             }
    //             // Here, sbi closes quietly due to no fall-backs
    //         }
    //
    //     }
    //     return Ok(());
    // }
    //
    // AppSBI::run(proj_dirs).await
    // if let Some(default_command) = cli_args.default_command {
    //     info!("Attempting to launch game through default command: {:?}", default_command);
    //     // game_launcher::launch_default(default_command)?;
    // }
    // const OS_LD_LIBRARY_NAME: &str = "LD_LIBRARY_PATH";
    //
    // let exec = "/home/jaquobia/steamapps/common/Starbound/linux/starbound";
    // let instance = "/home/jaquobia/steamapps/common/Starbound/";
    // let maybe_extra_ld_path: Option<std::path::PathBuf> = Some(std::path::PathBuf::from("/home/jaquobia/steamapps/common/Starbound/linux/"));
    //
    // let mut ld_paths = vec![];
    // if let Some(extra_ld_path) = maybe_extra_ld_path {
    //     ld_paths.push(extra_ld_path.to_path_buf());
    // }
    // if let Ok(system_ld_path) = std::env::var(OS_LD_LIBRARY_NAME) {
    //     ld_paths.extend(std::env::split_paths(&system_ld_path).map(std::path::PathBuf::from));
    // };
    // let new_ld_path_var = std::env::join_paths(ld_paths)?;
    //
    // info!(
    //     "Launching {} with ld_path: {:?}",
    //     exec,
    //     new_ld_path_var
    // );
    //
    // let mut command = tokio::process::Command::new(exec);
    // command.current_dir(instance);
    // let bootconfig = std::path::PathBuf::from(instance)
    //     .join(STARBOUND_BOOT_CONFIG_NAME)
    //     .display()
    //     .to_string();
    // command.env(OS_LD_LIBRARY_NAME, new_ld_path_var);
    // // command.args(["-bootconfig", &bootconfig]);
    //
    // command.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    // // tokio::task::spawn(async move {
    //     let exit = command.spawn()?.wait().await?;
    //     info!("{exit}");
    // //     let ret: Result<()> = Ok(());
    // //     ret
    // // }).await??;
}
