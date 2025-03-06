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
mod mod_manifest;
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
    #[error("Could not find the relevant folders to store application data in")]
    NoProjectDirectories,
    #[error("{0}")]
    LoggerFailure(#[from] flexi_logger::FlexiLoggerError),
    #[error("{0}")]
    IcedApplicationError(#[from] iced::Error),
    #[error("No vanilla assets directory! Please launch the game through steam with `sbi -- %command%` or directly with `SBI_VANILLA_ASSETS_DIR=some/path/ sbi`. SBI_VANILLA_ASSETS will take priority over the arguments.")]
    NoVanillaAssets,
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

#[derive(Debug, Clone)]
struct SBIDirectories {
    data_directory: PathBuf,
    profiles_directory: PathBuf,
    vanilla_assets: PathBuf,
}

impl SBIDirectories {
    fn new(cli: CliArgs) -> Result<Self, SBIInitializationError> {
        let default_proj_dirs =
            ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME)
                .ok_or(SBIInitializationError::NoProjectDirectories)?;

        let data_dir = parse_path_from_env("SBI_DATA_DIR")
            .unwrap_or_else(|| default_proj_dirs.data_dir().to_path_buf());

        let profiles_dir = data_dir.join("profiles");

        let starbound_steam_dir = match steamlocate::SteamDir::locate() {
            Err(e) => {
                log::error!("Error while parsing steam installtion: {e}");
                None
            }
            Ok(steam) => match steam.find_app(STARBOUND_STEAM_ID) {
                Err(e) => {
                    log::error!("{e}");
                    None
                },
                Ok(None) => {
                    log::error!("Starbound in not installed via steam. Please specify the location to find vanilla assets via SBI_VANILLA_ASSETS_DIR or the `--assets=/path/to/vanilla/assets` argument.");
                    None
                },
                Ok(Some((starbound, library))) => {
                    Some(library.resolve_app_dir(&starbound))
                }
            }
        };

        let vanilla_assets = {
            let vanilla_assets_source_cli = cli.assets;
            let vanilla_assets_source_env = parse_path_from_env("SBI_VANILLA_ASSETS_DIR");
            let vanilla_assets_source_steam = starbound_steam_dir.map(|d|d.join("assets"));

            vanilla_assets_source_cli
                .or(vanilla_assets_source_env)
                .or(vanilla_assets_source_steam)
                .ok_or(SBIInitializationError::NoVanillaAssets)?
        };

        Ok(Self {
            data_directory: data_dir,
            profiles_directory: profiles_dir,
            vanilla_assets,
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
                    Task::perform(config::load_config(data_dir), Message::FetchedConfig),
                ]),
            )
        })?;
    Ok(())
}
