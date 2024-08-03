use std::{fs, path::{Path, PathBuf}};

use anyhow::{anyhow, Result};
use itertools::Itertools;

use crate::{instance::{Instance, ModifyInstance}, json::{InstanceDataJson, SBIConfig}, INSTANCE_JSON_NAME, STARBOUND_BOOT_CONFIG_NAME};

fn write_instance(instance: &Instance) -> Result<()> {
    let instance_path = instance.folder_path();
    fs::create_dir_all(instance_path)?;
    let instance_data = serde_json::to_string(instance.to_json())?;
    fs::write(instance_path.join("instance.json"), instance_data)?;
    Ok(())
}


/// Write or replace the sbinit.config of the starbound instance.
///
/// # Errors
///
/// This function will return an error if 
/// 1: Both the instance and default executables are invalid,
/// 2: The Path to the executable is an invalid string (is this even possible),
/// 3: The fs fails to write to the sbinit.config file due to invalid permissions.
pub fn write_sbinit_config(instance: &Instance, config: &SBIConfig) -> Result<()> {
    let instance_folder = instance.folder_path();
    let sbinit_config_path = instance_folder.join(STARBOUND_BOOT_CONFIG_NAME);
    let executable = instance
        .executable()
        .as_ref()
        .and_then(|e| config.executables.get(e))
        .or_else(|| config.executables.get(&config.default_executable))
        .ok_or(anyhow!("Invalid Executable: {:?}", instance.executable()))?;
    let maybe_executable_assets = executable.custom_assets.as_ref();
    let mod_assets = instance_folder.join("mods");
    let vanilla_assets = config.vanilla_assets.clone();
    let maybe_additional_assets = instance.additional_assets();
    let storage_folder = instance_folder.join("storage");

    let mut assets = [vanilla_assets, mod_assets]
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect_vec();
    if let Some(executable_assets) = maybe_executable_assets {
        let executable_assets = PathBuf::from(&executable.bin)
            .parent()
            .unwrap()
            .join(executable_assets)
            .to_string_lossy()
            .to_string();
        assets.push(executable_assets);
    }
    if let Some(additional_assets) = maybe_additional_assets {
        // TODO: ~~apply instance_folder joining to the asset folder ONLY if its not a full path~~
        // Check if this works
        let additional_assets = additional_assets.iter().map(|f_name| {
            let p = PathBuf::from(f_name);
            if p.is_relative() {
                instance_folder.join(p).to_string_lossy().to_string()
            } else {
                f_name.to_owned()
            }
        });
        assets.extend(additional_assets);
    }
    let sbconfig_json = serde_json::json!({
        "assetDirectories": assets,
        "storageDirectory": storage_folder
    });

    let json_string = serde_json::to_string(&sbconfig_json)?;
    std::fs::write(sbinit_config_path, json_string).map_err(|e| anyhow!(e))
}

/// Write an instance for the first time, creating both the instance json and sbinit.config.
/// Will replace any spaces in the folder name with underscores.  
/// TODO: replace any special symbols  
///
/// # Errors
///
/// This function will return an error if writing the instance or config fails.
pub fn create_instance(instances_dir: &Path, data: InstanceDataJson, config: &SBIConfig) -> Result<()> {
    let name = data.name.replace(' ', "_");
    let mut instance_dir = instances_dir.join(&name);
    // Folders with the same name does not exactly mean instances with the same name
    let mut i: usize = 1;
    while instance_dir.exists() {
        instance_dir = instances_dir.join(format!("{}_{}", &name, i));
        i += 1;
    }
    let instance = Instance::from_json(data, &instance_dir.join(INSTANCE_JSON_NAME))?;
    write_instance(&instance)?;
    write_sbinit_config(&instance, config)
}

/// Deletes an instance.
///
/// # Errors
///
/// This function will return an error if the folder of the instance.json is invalid or can't be
/// written to.
pub fn delete_instance(instance: &Instance) -> Result<()> {
    fs::remove_dir_all(instance.folder_path())?;
    Ok(())
}

/// Modifies an instance with a know form of modification, then writes the changes to the
/// filesystem.
///
/// # Errors
///
/// This function will return an error if writing to the instance fails.
pub fn modify_instance(mut instance: Instance, modification: ModifyInstance, config: &SBIConfig) -> Result<()> {
    instance.modify(modification);
    write_instance(&instance)?;
    write_sbinit_config(&instance, config)
}
