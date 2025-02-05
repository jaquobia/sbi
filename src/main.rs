// use std::sync::LazyLock;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use flexi_logger::FileSpec;
use interprocess::local_socket::{traits::tokio::{Listener, Stream}, GenericFilePath, GenericNamespaced, ListenerOptions, Name, NameType};
use json::SBILaunchMessageJson;
use log::{info, warn};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt}, sync::mpsc::UnboundedSender};
use application::Application;

mod json;
mod instance;
mod cli_args;
mod workshop_downloader;
mod game_launcher;
mod mod_manifest;
mod core;
mod application;

static ORGANIZATION_QUALIFIER: &str = "";
static ORGANIZATION_NAME: &str = "";
static APPLICATION_NAME: &str = "sbi";

static INSTANCE_JSON_NAME: &str = "instance.json";
static SBI_CONFIG_JSON_NAME: &str = "config.json";

static STARBOUND_STEAM_ID: &str = "211820";
static STARBOUND_BOOT_CONFIG_NAME: &str = "sbinit.config";

static LOCAL_PIPE_NAME: &str = "@SBI_PIPE_NAME";
static LOCAL_PIPE_FS_NAME: &str = "/tmp/@SBI_PIPE_NAME";

// static DIRECTORIES : LazyLock<ProjectDirs> = LazyLock::new(|| {
//     ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME).expect("Could not read project directories")
// });

// Returns a platform accepted pipe name, preferring namespaced names if available
fn get_pipe_name() -> Result<Name<'static>> {
    let name = if GenericNamespaced::is_supported() {
        interprocess::local_socket::ToNsName::to_ns_name::<GenericNamespaced>(LOCAL_PIPE_NAME)?
	} else {
        interprocess::local_socket::ToFsName::to_fs_name::<GenericFilePath>(LOCAL_PIPE_FS_NAME)?
	};
    Ok(name)
}

/// Run the sbi service asyncronously which can be connected to with `connect_to_existing_sbi_service`.
/// This service will create a local pipe that will transmit a `json::SBILaunchMessage` if one has
/// been queued.
/// This service expects clients to connect from a steam launch request or that clients have a
/// fallback if the steam service was launched manually
///
/// This function will return an error if the pipe is already in use or cannot be created for some other generic IO reason.
pub async fn spawn_sbi_service() -> Result<UnboundedSender<SBILaunchMessageJson>> {
    let (sender, reciver) = tokio::sync::mpsc::unbounded_channel::<SBILaunchMessageJson>();

    let name = get_pipe_name()?;
    let listener = match interprocess::local_socket::tokio::Listener::from_options(ListenerOptions::new().name(name)) {
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            return Err(anyhow!("SBI pipe {:?} is already running somewhere...", get_pipe_name()))
        }
        Err(e) => {
            return Err(e.into());
        }
        Ok(x) => x,
    };
    tokio::spawn(async {

        // Capture environment
        let mut reciver = reciver;
        let listener = listener;

        // "async block doesn't return Result type" so wrapping failable code in a function
        async fn accept_client(listener: &interprocess::local_socket::tokio::Listener, reciver: &mut tokio::sync::mpsc::UnboundedReceiver<SBILaunchMessageJson>) -> Result<()> {
            // SBI slave has requested launch information
            let (_, mut writer) = listener.accept().await?.split();

            if let Ok(x) = reciver.try_recv() {
                let bytes = serde_json::to_vec::<SBILaunchMessageJson>(&x)?;
                writer.write_all(&bytes).await?;
            }
            Ok(())
        } // end fn

        loop {
            let _ = accept_client(&listener, &mut reciver).await;
        }
    });
    Ok(sender)
}

/// Get the launch message from an SBI service and launch starbound.
///
/// # Errors
///
/// This function will return an error if the message cannot be parsed, the cwd does not have
/// read permissions, or the game launch fails for some reason.
async fn connect_to_existing_sbi_service(local_socket: interprocess::local_socket::tokio::Stream) -> Result<()> {
    info!("Launching starbound through steam!");

    let (recv, _) = local_socket.split();
    let mut recv = tokio::io::BufReader::new(recv);
    let mut buffer = String::with_capacity(2048);
    recv.read_line(&mut buffer).await?;
    let launch_message: json::SBILaunchMessageJson = serde_json::from_str(&buffer)?;

    let executable_path = launch_message.exececutable_path;
    let maybe_extra_ld_path = launch_message.ld_library_path.as_deref();
    let instance_path = launch_message.instance_path.unwrap_or_else(|| std::env::current_dir().expect("Current working directory cannot be read from"));
    game_launcher::launch_instance_cli(&executable_path, &instance_path, maybe_extra_ld_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: This does not change after initialization, maybe replace storage with static reference?
    let proj_dirs = ProjectDirs::from(ORGANIZATION_QUALIFIER, ORGANIZATION_NAME, APPLICATION_NAME).ok_or(anyhow!("Can't find home directory"))?;

    // let cli_args: CliArgs = clap::Parser::try_parse()?;
    let _log_handle = flexi_logger::Logger::try_with_env_or_str("info")?
        .log_to_file(
            FileSpec::default()
            .directory(proj_dirs.data_dir())
            .basename("sbi")
            .suppress_timestamp()
        )
        .start()?;

    let theme = |state: &Application| -> iced::Theme {
        iced::Theme::TokyoNight
    };

    iced::application("SBI", Application::update, Application::view).theme(theme).run()?;

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
    Ok(())
}
