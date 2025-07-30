use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{menus::duplicate_profile::DuplicateData, PROFILE_JSON_NAME};

#[derive(Debug, Clone)]
pub enum ProfileData {
    Vanilla,
    Json(ProfileJson),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProfileJson {
    pub name: String,
    pub additional_assets: Option<Vec<PathBuf>>,
    pub collection_id: Option<String>,
    #[serde(default)]
    pub link_mods: bool,
    pub selected_executable: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Profile {
    path: PathBuf,
    data: ProfileData,
}

impl Profile {
    fn into_parts(self) -> (PathBuf, ProfileData) {
        (self.path, self.data)
    }

    fn from_json(json: ProfileJson, path: PathBuf) -> Profile {
        let path = path
            .parent()
            .expect("Profile json should have a parent, but does not??")
            .to_path_buf();
        Profile {
            path,
            data: ProfileData::Json(json),
        }
    }

    fn from_vanilla(path: PathBuf) -> Profile {
        Profile {
            path,
            data: ProfileData::Vanilla,
        }
    }

    pub fn name(&self) -> &str {
        match &self.data {
            ProfileData::Json(json) => &json.name,
            ProfileData::Vanilla => "Default",
        }
    }

    pub fn json(&self) -> Option<&ProfileJson> {
        match &self.data {
            ProfileData::Json(j) => Some(j),
            ProfileData::Vanilla => None,
        }
    }
    pub fn json_mut(&mut self) -> Option<&mut ProfileJson> {
        match &mut self.data {
            ProfileData::Json(j) => Some(j),
            ProfileData::Vanilla => None,
        }
    }

    pub fn set_json(&mut self, json: ProfileJson) {
        match &mut self.data {
            ProfileData::Json(j) => *j = json,
            ProfileData::Vanilla => log::error!("Trying to write json data to vanilla profile!"),
        }
    }

    pub fn clear_selected_executable(&mut self) {
        match &mut self.data {
            ProfileData::Json(json) => json.selected_executable = None,
            ProfileData::Vanilla => log::warn!("Clearing selected executable of vanilla profile does nothing... and this branch should have never been called"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    //TODO: Reduce overhead somehow? Is this necessary?
    pub fn additional_assets(&self) -> impl Iterator<Item = PathBuf> {
        match &self.data {
            ProfileData::Json(json) => json.additional_assets.clone(),
            ProfileData::Vanilla => None,
        }
        .into_iter()
        .flatten()
    }

    pub fn link_mods(&self) -> bool {
        match &self.data {
            ProfileData::Json(json) => json.link_mods,
            ProfileData::Vanilla => true,
        }
    }

    pub fn selected_executable(&self) -> Option<&str> {
        match &self.data {
            ProfileData::Json(json) => json.selected_executable.as_deref(),
            ProfileData::Vanilla => None,
        }
    }

    pub fn is_vanilla(&self) -> bool {
        matches!(self.data, ProfileData::Vanilla)
    }
}

/// Returns a collection of all valid profiles in the profiles directory.
/// A valid profile consists of a folder in the profiles directory which contains a valid json.
pub async fn find_profiles(
    profiles_directory: std::path::PathBuf,
    maybe_vanilla_profile_directory: Option<std::path::PathBuf>,
) -> Vec<Profile> {
    let paths = crate::profile::collect_profile_json_paths(&profiles_directory);
    match paths {
        Ok(paths) => {
            crate::profile::parse_profile_paths_to_json(&paths, maybe_vanilla_profile_directory)
        }
        Err(e) => {
            log::error!("Error gathering profiles: {e}");
            vec![]
        }
    }
}

/// Turns instance.json into Instance struct
fn parse_profile_paths_to_json(
    instance_json_paths: &[PathBuf],
    maybe_vanilla_profile_directory: Option<std::path::PathBuf>,
) -> Vec<Profile> {
    let vanilla_profile = maybe_vanilla_profile_directory
        .into_iter()
        .map(Profile::from_vanilla);
    let sbi_profiles = instance_json_paths
        .iter()
        .map(|ins_path| std::fs::read_to_string(ins_path).map(|str| (str, ins_path.clone())))
        .filter_map(Result::ok)
        .map(|(data, path)| serde_json::from_str(&data).map(|data| Profile::from_json(data, path)))
        .filter_map(Result::ok);
    vanilla_profile.chain(sbi_profiles).collect()
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

// pub async fn write_profile_then_find_list(
//     p: Profile,
//     profiles_directory: std::path::PathBuf,
//     maybe_vanilla_profile_directory: Option<std::path::PathBuf>,
// ) -> Vec<Profile> {
// }

pub async fn write_profile(p: Profile) -> std::io::Result<()> {
    if let ProfileData::Json(json) = &p.data {
        tokio::fs::create_dir_all(&p.path).await?;
        let instance_data = serde_json::to_vec(json)?;
        tokio::fs::write(p.path.join(PROFILE_JSON_NAME), instance_data).await?;
    }
    Ok(())
}

pub async fn create_profile_then_find_list(
    p: ProfileJson,
    profiles_directory: std::path::PathBuf,
    maybe_vanilla_profile_directory: Option<std::path::PathBuf>,
) -> Vec<Profile> {
    let profile_path = find_valid_profile_path(&p.name, &profiles_directory).await;
    let p = Profile {
        path: profile_path,
        data: ProfileData::Json(p),
    };
    // write_profile_then_find_list(p, profiles_directory, maybe_vanilla_profile_directory).await
    if let Err(e) = write_profile(p).await {
        log::error!("Error while writing profile to disk: {e}");
    }

    find_profiles(profiles_directory, maybe_vanilla_profile_directory).await
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

pub async fn duplicate_profile_then_find_list(
    current_profile: Profile,
    data: DuplicateData,
    profiles_directory: std::path::PathBuf,
    maybe_vanilla_profile_directory: Option<std::path::PathBuf>,
) -> Vec<Profile> {
    let (current_path, current_data) = current_profile.into_parts();
    let json: ProfileJson = match current_data {
        ProfileData::Json(mut json) => {
            json.name = data.name;
            json
        }
        // Duplicating vanilla profile
        ProfileData::Vanilla => {
            ProfileJson {
                name: data.name,
                additional_assets: None,
                collection_id: None,
                link_mods: true,
                selected_executable: None,
            }
        }
    };
    let new_profile_path = find_valid_profile_path(&json.name, &profiles_directory).await;
    // write_profile_then_find_list(p, profiles_directory, maybe_vanilla_profile_directory).await

    let new_profile = Profile {
        path: new_profile_path,
        data: ProfileData::Json(json),
    };
    if let Err(e) = copy_dir_all(&current_path, &new_profile.path).await {
        log::error!(
            "Error {e} while copying profile {} to {}",
            current_path.display(),
            new_profile.path.display()
        );
    };
    if let Err(e) = write_profile(new_profile).await {
        log::error!("Error while writing profile to disk: {e}");
    }

    find_profiles(profiles_directory, maybe_vanilla_profile_directory).await
}

async fn copy_dir_all<P: AsRef<Path>>(src: P, dst: P) -> std::io::Result<()> {
    let dst = dst.as_ref();
    tokio::fs::create_dir_all(dst).await?;
    let mut read_dir = tokio::fs::read_dir(src).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let ty = entry.file_type().await?;
        let new_name: PathBuf = dst.join(entry.file_name());
        if ty.is_dir() {
            // Pin recursive call to prevent infinte-sized state machine
            // https://rust-lang.github.io/async-book/07_workarounds/04_recursion.html
            Box::pin(copy_dir_all(entry.path(), new_name)).await?;
        } else {
            tokio::fs::copy(entry.path(), new_name).await?;
        }
    }
    Ok(())
}
