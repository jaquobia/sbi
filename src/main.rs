use std::{
    env::VarError,
    path::{Path, PathBuf},
};

use application::{Application, Message};
use directories::ProjectDirs;
use iced::Task;

mod application;
mod cli_args;
mod config;
mod executable;
mod game_launcher;
mod mod_manifest;
mod profile;

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
    LoggerFailure(#[from] flexi_logger::FlexiLoggerError),
    #[error("...")]
    IcedApplicationError(#[from] iced::Error),
}

#[derive(Debug, Clone)]
struct SBIDirectories {
    data_directory: PathBuf,
    profiles_directory: PathBuf,
}

impl SBIDirectories {
    fn new() -> Result<Self, SBIInitializationError> {
        let proj_dirs =
            ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME)
                .ok_or(SBIInitializationError::NoProjectDirectories)?;

        let data_dir = match std::env::var("SBI_DATA_DIR") {
            Err(e) => {
                if let VarError::NotUnicode(s) = e {
                    log::error!(
                        "SBI_DATA_DIR exists but contains non-unicode characters: {}",
                        s.to_string_lossy()
                    );
                }
                None
            }
            Ok(d) => Some(PathBuf::from(d)),
        }
        .unwrap_or_else(|| proj_dirs.data_dir().to_path_buf());

        let profiles_dir = data_dir.join("profiles");

        Ok(Self {
            data_directory: data_dir,
            profiles_directory: profiles_dir,
        })
    }

    pub fn data(&self) -> &Path {
        &self.data_directory
    }

    pub fn profiles(&self) -> &Path {
        &self.profiles_directory
    }
}

fn main() -> Result<(), SBIInitializationError> {
    let _log_handle = flexi_logger::Logger::try_with_env_or_str("info")?
        // .log_to_file(
        //     flexi_logger::FileSpec::default()
        //         .directory(&data_dir)
        //         .basename("sbi")
        //         .suppress_timestamp(),
        // )
        .start()?;

    let dirs = SBIDirectories::new()?;

    if !dirs.data().exists() {
        if let Err(e) = std::fs::create_dir_all(dirs.data()) {
            log::error!("Data directory does not exist and could not be created: {e}");
        }
    }

    // let cli_args: CliArgs = clap::Parser::try_parse()?;

    let application = Application::new(dirs);
    let profiles_dir = application.dirs().profiles().to_path_buf();
    let data_dir = application.dirs().data().to_path_buf();
    iced::application("SBI", Application::update, Application::view)
        .theme(Application::theme)
        .run_with(move || {
            (
                application,
                Task::batch([
                    Task::perform(
                        profile::find_profiles(profiles_dir),
                        Message::FetchedProfiles,
                    ),
                    Task::perform(config::load_config(data_dir), Message::FetchConfig),
                ]),
            )
        })?;
    Ok(())
}
