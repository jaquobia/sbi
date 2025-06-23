use serde_json::json;
use std::{path::PathBuf, process::Stdio};

use crate::{executable::Executable, profile::Profile, STARBOUND_BOOT_CONFIG_NAME};

const OS_LD_LIBRARY_NAME: &str = "LD_LIBRARY_PATH";

#[derive(Debug, Copy, Clone)]
pub enum SBILaunchStatus {
    Success,
    Failure,
}

pub async fn write_init_config(
    profile: &Profile,
    vanilla_mods: Option<PathBuf>,
    vanilla_assets: PathBuf,
    executable_assets: Option<PathBuf>,
) -> anyhow::Result<()> {
    let config_path = profile.path().join(STARBOUND_BOOT_CONFIG_NAME);
    log::info!("Vanilla assets dir: {}", vanilla_assets.display());
    log::info!("Vanilla mods dir: {:?}", vanilla_mods);
    log::info!(
        "Attempting to write sbinit.config to: {}",
        config_path.display()
    );
    let mut asset_directories: Vec<PathBuf> = vec![vanilla_assets];
    asset_directories.extend(executable_assets);
    asset_directories.extend(profile.additional_assets());
    if let Some(p) = vanilla_mods.filter(|p| p.exists() && profile.link_mods()) {
        asset_directories.push(p);
    }
    let storage_directory = profile.path();
    if !storage_directory.exists() {
        if let Err(e) = tokio::fs::create_dir_all(storage_directory).await {
            log::error!("Failed to create missing storage directory: {e}");
        }
    }

    // TODO: Find a way to either configure these or determine a reasonable default
    let allow_admin_commands_from_anyone: bool = false;
    let anonymous_connections_are_admin: bool = false;

    let json = json!({
        "assetDirectories": asset_directories,
        "storageDirectory": storage_directory,
        "assetsSettings": {
            "pathIgnore": [],
            "digestIgnore": [ ".*" ]
        },
        "defaultConfiguration": {
            "allowAdminCommandsFromAnyone": allow_admin_commands_from_anyone,
            "anonymousConnectionsAreAdmin": anonymous_connections_are_admin,
        }
    });

    let bytes = serde_json::to_vec(&json)?;
    tokio::fs::write(config_path, &bytes).await?;
    Ok(())
}

async fn lauch_game_inner(executable: Executable, profile: Profile) -> anyhow::Result<()> {
    let executable_path = executable.bin;
    let executable_folder = executable_path.parent().expect("").to_path_buf();
    let instance_dir = profile.path();

    let new_ld_path_var = {
        let mut ld_paths = vec![executable_folder];
        if let Ok(system_ld_path) = std::env::var(OS_LD_LIBRARY_NAME) {
            ld_paths.extend(std::env::split_paths(&system_ld_path).map(PathBuf::from));
        }
        std::env::join_paths(ld_paths).ok()
    };

    std::env::set_current_dir(instance_dir)?;

    let mut command = tokio::process::Command::new(executable_path);
    // command.current_dir(instance_dir);
    let bootconfig = instance_dir
        .join(STARBOUND_BOOT_CONFIG_NAME)
        .display()
        .to_string();
    // let bootconfig = ["./", STARBOUND_BOOT_CONFIG_NAME].join("");
    if let Some(path) = new_ld_path_var {
        log::info!("Setting {OS_LD_LIBRARY_NAME} to {}", path.to_string_lossy());
        command.env(OS_LD_LIBRARY_NAME, path);
    }
    command.args(["-bootconfig", &bootconfig]);

    // This little shit line caused me so
    // many issues with zombie processes.
    // Remember to unhook stdio for
    // children you give up

    // This async thread is not necessary as we don't want to own children
    // but this also causes no harm
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    // tokio::task::spawn(async move {  });
    let exit_status = command.spawn()?.wait().await?;
    log::info!("{}", exit_status);
    Ok(())
}

pub async fn launch_game(
    executable: Executable,
    profile: Profile,
    vanilla_mods: Option<PathBuf>,
    vanilla_assets: PathBuf,
) -> SBILaunchStatus {
    if let Err(e) = write_init_config(&profile, vanilla_mods, vanilla_assets, executable.assets()).await {
        log::error!("Error writing sbinit.config: {e}");
        return SBILaunchStatus::Failure;
    }

    if let Err(e) = lauch_game_inner(executable, profile).await {
        log::error!("Error while launching executable: {e}");
        return SBILaunchStatus::Failure;
    }
    SBILaunchStatus::Success
}
