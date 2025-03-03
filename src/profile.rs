use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::PROFILE_JSON_NAME;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProfileJson {
    pub name: String,
    pub additional_assets: Option<Vec<String>>,
    pub collection_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Profile {
    path: PathBuf,
    profile_json: ProfileJson,
}

impl Profile {
    fn from_json(json: ProfileJson, path: PathBuf) -> Profile {
        let path = path
            .parent()
            .expect("Profile json should have a parent, but does not??")
            .to_path_buf();
        Profile {
            path,
            profile_json: json,
        }
    }

    pub fn name(&self) -> &str {
        &self.profile_json.name
    }

    pub fn json_path(&self) -> &Path {
        &self.path
    }

    pub fn folder_path(&self) -> PathBuf {
        self.path.parent().expect("Existing profile does not have a parent folder??").to_path_buf()
    }
}

/// Returns a collection of all valid profiles in the profiles directory.
/// A valid profile consists of a folder in the profiles directory which contains a valid json.
pub async fn find_profiles(profiles_directory: std::path::PathBuf) -> Vec<Profile> {
    let paths = crate::profile::collect_profile_json_paths(&profiles_directory);
    match paths {
        Ok(paths) => crate::profile::parse_profile_paths_to_json(&paths),
        Err(e) => {
            log::error!("Error gathering profiles: {e}");
            vec![]
        }
    }
}

/// Turns instance.json into Instance struct
fn parse_profile_paths_to_json(instance_json_paths: &[PathBuf]) -> Vec<Profile> {
    instance_json_paths
        .iter()
        .map(|ins_path| std::fs::read_to_string(ins_path).map(|str| (str, ins_path.clone())))
        .filter_map(Result::ok)
        .map(|(data, path)| serde_json::from_str(&data).map(|data| Profile::from_json(data, path)))
        .filter_map(Result::ok)
        .collect()
}

/// Returns an owned iterator of paths to the instance.json of each instance
fn collect_profile_json_paths(profiles_dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
    let instances = profiles_dir
        .read_dir()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() || path.is_symlink())
        .map(|path| path.join(PROFILE_JSON_NAME))
        .filter(|path| path.is_file())
        .collect();
    Ok(instances)
}

pub async fn write_profile_then_find_list(p: Profile, profiles_directory: std::path::PathBuf) -> Vec<Profile> {
    if let Err(e) = write_profile(p).await {
        log::error!("Error while writing profile to disk: {e}");
    }
    find_profiles(profiles_directory).await
}

async fn write_profile(p: Profile) -> std::io::Result<()> {
    tokio::fs::create_dir_all(&p.path).await?;
    let instance_data = serde_json::to_vec(&p.profile_json)?;
    tokio::fs::write(p.path.join(PROFILE_JSON_NAME), instance_data).await?;
    Ok(())
}

pub async fn create_profile_then_find_list(p: ProfileJson, profiles_directory: std::path::PathBuf) -> Vec<Profile> {
    let profile_path = find_valid_profile_path(&p.name, &profiles_directory).await;
    let p = Profile { path: profile_path, profile_json: p };
    write_profile_then_find_list(p, profiles_directory).await
}

async fn find_valid_profile_path(name: &str, profiles_directory: &std::path::Path) -> PathBuf {
    let filtered_name = name.replace([' ', '-', '\\', '/'], "_");
    let mut path = profiles_directory.join(&filtered_name);
    let mut i: usize = 0;
    while path.exists() {
        path = profiles_directory.join(format!("{}_{}", &filtered_name, i));
        i += 1;
    }
    path
}
