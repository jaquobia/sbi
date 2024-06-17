use std::{path::{Path, PathBuf}, process::Stdio};

use anyhow::Result;
use log::info;

use crate::{STARBOUND_BOOT_CONFIG_NAME, STARBOUND_STEAM_ID};

const OS_LD_LIBRARY_NAME: &str = "LD_LIBRARY_PATH";

pub fn launch_default(default_launch_command: Vec<String>) -> Result<()> {
    if let [command_path, args @ ..] = default_launch_command.as_slice() {
        let mut command = tokio::process::Command::new(command_path);
        command.args(args);
        tokio::task::spawn(async move { command.spawn()?.wait().await });
    }
    Ok(())
}

pub fn launch_instance_cli(executable_path: &Path, instance_dir: &Path, maybe_extra_ld_path: Option<&Path>) -> Result<()> {

    let mut ld_paths = vec![];
    if let Some(extra_ld_path) = maybe_extra_ld_path {
        ld_paths.push(extra_ld_path.to_path_buf());
    }
    if let Ok(system_ld_path) = std::env::var(OS_LD_LIBRARY_NAME) {
        ld_paths.extend(std::env::split_paths(&system_ld_path).map(PathBuf::from));
    };
    let new_ld_path_var = std::env::join_paths(ld_paths)?;

    info!(
        "Launching {} with ld_path: {:?}",
        executable_path.display(),
        new_ld_path_var
    );

    let mut command = tokio::process::Command::new(executable_path);
    command.current_dir(instance_dir);
    let bootconfig = instance_dir
        .join(STARBOUND_BOOT_CONFIG_NAME)
        .display()
        .to_string();
    command.env(OS_LD_LIBRARY_NAME, new_ld_path_var);
    command.args(["-bootconfig", &bootconfig]);

    // This little shit line caused me so
    // many issues with zombie processes.
    // Remember to unhook stdio for
    // children you give up

    // This async thread is not necessary as we don't want to own children
    // but this also causes no harm
    command.stdout(Stdio::null()).stderr(Stdio::null());                                                          
    tokio::task::spawn(async move { command.spawn()?.wait().await });
    Ok(())
}


pub fn launch_instance_steam(boot_config: Option<&Path>) -> Result<()> {
    let mut command = tokio::process::Command::new("steam");
    command.args(["-applaunch", STARBOUND_STEAM_ID]);
    if let Some(boot_config) = boot_config {
        command.arg("-bootconfig"); 
        command.arg(boot_config); 
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    tokio::task::spawn(async move { child.wait().await });
    Ok(())
}
