use std::path::PathBuf;

use crate::{json::ProfileJson, PROFILE_JSON_NAME};

#[derive(Debug, Clone)]
pub struct Profile {
    path: PathBuf,
    profile_json: ProfileJson,
}

impl Profile {
    fn from_json(json: ProfileJson, path: PathBuf) -> Profile {
        let path = path
            .parent()
            .expect("Profile should have a parent, but does not??")
            .to_path_buf();
        Profile {
            path,
            profile_json: json,
        }
    }

    pub fn name(&self) -> &str {
        &self.profile_json.name
    }
}

/// Turns instance.json into Instance struct
pub fn parse_profile_paths_to_json(instance_json_paths: &[PathBuf]) -> Vec<Profile> {
    instance_json_paths
        .iter()
        .map(|ins_path| std::fs::read_to_string(ins_path).map(|str| (str, ins_path.clone())))
        .filter_map(Result::ok)
        .map(|(data, path)| serde_json::from_str(&data).map(|data| Profile::from_json(data, path)))
        .filter_map(Result::ok)
        .collect()
}

/// Returns an owned iterator of paths to the instance.json of each instance
pub fn collect_profile_json_paths(profiles_dir: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
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
