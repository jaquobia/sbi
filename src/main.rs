use std::{
    env::VarError,
    path::{Path, PathBuf},
};

use application::{Application, Message};
use clap::Parser;
use cli_args::CliArgs;
use directories::ProjectDirs;
use iced::Task;

mod application;
mod cli_args;
mod config;
mod executable;
mod game_launcher;
mod menus;
mod profile;

static ORGANIZATION_QUALIFIER: &str = "";
static ORGANIZATION_NAME: &str = "";
static APPLICATION_NAME: &str = "sbi";

static PROFILE_JSON_NAME: &str = "profile.json";
static SBI_CONFIG_JSON_NAME: &str = "config.json";

static STARBOUND_STEAM_ID: u32 = 211820;
static STARBOUND_BOOT_CONFIG_NAME: &str = "sbinit.config";

// static LOCAL_PIPE_NAME: &str = "@SBI_PIPE_NAME";
// static LOCAL_PIPE_FS_NAME: &str = "/tmp/@SBI_PIPE_NAME";

#[derive(Debug, thiserror::Error)]
enum SBIInitializationError {
    #[error("{0}")]
    NoProjectDirectories(#[from] SBIDirectoryError),
    #[error("{0}")]
    LoggerFailure(#[from] flexi_logger::FlexiLoggerError),
    #[error("{0}")]
    IcedApplicationError(#[from] iced::Error),
    #[error("{0}")]
    ClapFailedToParseCLI(#[from] clap::Error),
}

/// Reads an environment variable and returns the value as a PathBuf, or None if parsing failed.
/// If parsing the variable leads to a NotUnicode error, a relevant error message will also be printed.
pub fn parse_path_from_env(variable: &str) -> Option<PathBuf> {
    match std::env::var(variable) {
        Err(e) => {
            // If it is a Unicode error, then the variable exists but we can't read it. Report it
            // to the user.
            if let VarError::NotUnicode(s) = e {
                log::error!(
                    "{variable} exists but contains non-unicode characters: {}",
                    s.to_string_lossy()
                );
            }
            // Else, the variable does not exist and the user did not intend set a value.
            None
        }
        Ok(d) => Some(PathBuf::from(d)),
    }
}

#[derive(Debug, thiserror::Error)]
enum SBIDirectoryError {
    #[error("No home directory could be found. Default directories cannot be constructed.")]
    NoDefaultDirectories,
    #[error("No vanilla assets directory! Please launch the game through steam with `sbi -- %command%` or directly with `SBI_VANILLA_ASSETS_DIR=some/path/ sbi`. SBI_VANILLA_ASSETS will take priority over the arguments.")]
    NoVanillaAssetsDirectory,
    #[error("SBI data directory cannot be created: {0}")]
    FailedToCreateDataDir(std::io::Error),
    #[error("SBI profiles directory cannot be created: {0}")]
    FailedToCreateProfilesDir(std::io::Error),
}

#[derive(Debug, Clone)]
struct SBIDirectories {
    data_directory: PathBuf,
    profiles_directory: PathBuf,
    vanilla_assets: PathBuf,
    vanilla_storage: Option<PathBuf>,
    vanilla_mods: Option<PathBuf>,
}

impl SBIDirectories {
    fn new(cli: CliArgs) -> Result<Self, SBIDirectoryError> {
        // TODO: Ensure this error gets ignored if the data directory is specified through environment
        // variables.
        let default_proj_dirs =
            ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME)
                .ok_or(SBIDirectoryError::NoDefaultDirectories)?;

        let data_dir = parse_path_from_env("SBI_DATA_DIR")
            .unwrap_or_else(|| default_proj_dirs.data_dir().to_path_buf());
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).map_err(SBIDirectoryError::FailedToCreateDataDir)?;
        }

        let profiles_dir = data_dir.join("profiles");
        if !profiles_dir.exists() {
            std::fs::create_dir_all(&profiles_dir)
                .map_err(SBIDirectoryError::FailedToCreateProfilesDir)?;
        }

        let starbound_steam_dir = match steamlocate::SteamDir::locate() {
            Err(e) => {
                log::error!("Error while parsing steam installtion: {e}");
                None
            }
            Ok(steam) => match steam.find_app(STARBOUND_STEAM_ID) {
                Err(e) => {
                    log::error!("{e}");
                    None
                }
                Ok(None) => {
                    log::error!("Starbound in not installed via steam. Please specify the location to find vanilla assets via SBI_VANILLA_ASSETS_DIR or the `--assets=/path/to/vanilla/assets` argument.");
                    None
                }
                Ok(Some((starbound, library))) => Some(library.resolve_app_dir(&starbound)),
            },
        };

        let vanilla_assets = {
            let vanilla_assets_source_cli = cli.assets;
            let vanilla_assets_source_env = parse_path_from_env("SBI_VANILLA_ASSETS_DIR");
            let vanilla_assets_source_steam =
                starbound_steam_dir.as_ref().map(|d| d.join("assets"));

            vanilla_assets_source_cli
                .or(vanilla_assets_source_env)
                .or(vanilla_assets_source_steam)
                .ok_or(SBIDirectoryError::NoVanillaAssetsDirectory)?
        };

        let vanilla_storage = starbound_steam_dir.as_ref().map(|d| d.join("storage"));
        let vanilla_mods = starbound_steam_dir.map(|d| d.join("mods"));

        Ok(Self {
            data_directory: data_dir,
            profiles_directory: profiles_dir,
            vanilla_assets,
            vanilla_storage,
            vanilla_mods,
        })
    }

    pub fn data(&self) -> &Path {
        &self.data_directory
    }

    pub fn profiles(&self) -> &Path {
        &self.profiles_directory
    }

    pub fn vanilla_assets(&self) -> &Path {
        &self.vanilla_assets
    }

    pub fn vanilla_storage(&self) -> Option<&Path> {
        self.vanilla_storage.as_deref()
    }

    pub fn vanilla_mods(&self) -> Option<&Path> {
        self.vanilla_mods.as_deref()
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

    let cli = CliArgs::parse();

    let dirs = SBIDirectories::new(cli)?;

    let application = Application::new(dirs);
    let profiles_dir = application.dirs().profiles().to_path_buf();
    let vanilla_profile_dir = application.dirs().vanilla_storage().map(PathBuf::from);
    let data_dir = application.dirs().data().to_path_buf();
    iced::application("SBI", Application::update, Application::view)
        .theme(Application::theme)
        .run_with(move || {
            (
                application,
                Task::batch([
                    Task::perform(
                        profile::find_profiles(profiles_dir, vanilla_profile_dir),
                        Message::FetchedProfiles,
                    ),
                    Task::perform(config::load_config(data_dir), Message::FetchedConfig),
                ]),
            )
        })?;
    Ok(())
}
