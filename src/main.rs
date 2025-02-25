use application::{Application, Message};
use directories::ProjectDirs;
use flexi_logger::{FileSpec, FlexiLoggerError};
use iced::Task;

mod application;
mod cli_args;
mod executable;
// mod core;
mod config;
mod game_launcher;
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
        // .log_to_file(
        //     FileSpec::default()
        //         .directory(proj_dirs.data_dir())
        //         .basename("sbi")
        //         .suppress_timestamp(),
        // )
        .start()?;

    let application = Application::new(proj_dirs);
    let profiles_dir = application.profiles_directory();
    let data_dir = application.data_directory();
    iced::application("SBI", Application::update, Application::view)
        .theme(Application::theme)
        .run_with(move || {
            (
                application,
                Task::perform(
                    profile::find_profiles(profiles_dir),
                    Message::FetchedProfiles,
                )
                .chain(Task::perform(
                    config::load_config(data_dir),
                    Message::FetchConfig,
                )),
            )
        })?;
    Ok(())
}
