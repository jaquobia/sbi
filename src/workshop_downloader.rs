use std::{os::fd::{AsFd, AsRawFd, FromRawFd}, path::Path, process::Stdio};

use log::{debug, error, info};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

use crate::{instance::Instance, mod_manifest::{read_manifest, write_manifest, ModManifestJson, ModManifestMod}};

#[derive(Serialize, Deserialize, Debug)]
struct CollectionDetailsRequest {
    pub response: CollectionDetailsList
}

#[derive(Serialize, Deserialize, Debug)]
struct CollectionDetailsList {
    pub collectiondetails: Vec<CollectionDetails>
}

#[derive(Serialize, Deserialize, Debug)]
struct CollectionDetails {
    pub children: Vec<CollectionChildren>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CollectionChildren {
    filetype: i32,
    publishedfileid: String
}

impl CollectionChildren {
    pub fn into_file_id(self) -> String {
        self.publishedfileid
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PublishedFileRequest {
    pub response: PublishedFileDetails
}

#[derive(Serialize, Deserialize, Debug)]
struct PublishedFileDetails {
    pub publishedfiledetails: Vec<PublishedFileChildren>
}

#[derive(Serialize, Deserialize, Debug)]
struct PublishedFileChildren {
    publishedfileid: String,
    time_updated: u64,
}

impl From<PublishedFileChildren> for ModManifestMod {
    fn from(val: PublishedFileChildren) -> Self {
        ModManifestMod { publishedfileid: val.publishedfileid, version: val.time_updated }
    }
}

/// Gather all the names of mods currently installed on disk.
/// Mods are discovered if it is a file in the ./instance/mods directory
/// and is a directory or a file with a .pak extension.
///
/// This function assumes mods_folder is a valid Path
///
/// # Errors
/// This function will return an error if a ReadDir could not be formed, otherwise per-file errors are
/// traced and handled internally.
async fn collect_installed_mods(mods_folder: &Path) -> Result<Vec<String>> {
    let mut installed_files = vec![];
    let mut dir_reader = tokio::fs::read_dir(mods_folder).await?;
    loop {
        match dir_reader.next_entry().await {
            Ok(Some(entry)) => {
                let file_name = match entry.file_name().into_string() {
                    Ok(string) => string,
                    Err(os_string) => { os_string.to_string_lossy().to_string() }
                };
                let ft = match entry.file_type().await {
                    Ok(ft) => { ft },
                    Err(e) => { error!("{e}"); continue; }
                };
                if (ft.is_dir()) || (ft.is_file() && entry.path().extension().is_some_and(|ext|ext.eq("pak"))) {
                    installed_files.push(file_name.replace(".pak", ""));
                }
            },

            // No more files left to read
            Ok(None) => { break; },

            //Error occured while reading an entry,
            //Lets not worry about it
            Err(e) => { error!("{e}"); continue; },
        }
    }
    Ok(installed_files)
}

/// Gathers a list of publishedfileid's from a 
/// steam collection and all linked collections.
///
/// # Errors
///
/// This function will return an error if a post request could not be sent or invalid json is returned.
async fn collect_mods_from_collections(client: &reqwest::Client, collections: &mut Vec<String>) -> Result<Vec<CollectionChildren>> {
    const COLLECTION_URL: &str = "https://api.steampowered.com/ISteamRemoteStorage/GetCollectionDetails/v1/";
    let mut children: Vec<CollectionChildren> = vec![];

    while !collections.is_empty() {
        let mut params = FxHashMap::default();
        params.insert(String::from("collectioncount"), collections.len().to_string());
        for (i, c) in collections.drain(..).enumerate() {
            let idx = format!("publishedfileids[{}]", i);
            params.insert(idx, c.clone());
        }
        let resp = client.post(COLLECTION_URL).form(&params).send().await?;
        children.extend({
            let response: CollectionDetailsRequest = resp.json().await?;
            // filetype == 0 is mod, filetype == 2 is linked collection
            let (children, linked_collections): (Vec<CollectionChildren>, Vec<CollectionChildren>) = response.response.collectiondetails.into_iter().flat_map(|cd| cd.children).partition(|c| c.filetype == 0);
            collections.extend(linked_collections.into_iter().map(CollectionChildren::into_file_id));
            children
        });
    }
    Ok(children)
}

async fn get_versioned_mods(client: &reqwest::Client, mods: &[CollectionChildren]) -> Result<Vec<ModManifestMod>> {
    const PUBLISHED_FILE_URL: &str = "https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/";
    let mut params = FxHashMap::default();
    params.insert(String::from("itemcount"), mods.len().to_string());
    for (i, m) in mods.iter().enumerate() {
        let idx = format!("publishedfileids[{}]", i);
        params.insert(idx, m.publishedfileid.clone());
    }
    let resp = client.post(PUBLISHED_FILE_URL).form(&params).send().await?;
    let response: PublishedFileRequest = resp.json().await?;
    let children: Vec<ModManifestMod> = response.response.publishedfiledetails.into_iter().map(PublishedFileChildren::into).collect();
    Ok(children)
}

// Create SteamCMD command & spawn command
async fn download_mods_from_workshop(force_install_dir: &Path, mods_to_install: &[ModManifestMod], log_file: Option<&Path>) -> Result<()> {
    let force_install_dir_str = force_install_dir.to_str().ok_or(anyhow!("force_install_dir ({}) not a valid directory", force_install_dir.display()))?;
    let steamcmd_params = {
        let mut steamcmd_params = vec!["+force_install_dir", force_install_dir_str, "+login", "anonymous"];
        mods_to_install.iter().for_each(|m| steamcmd_params.extend(["+workshop_download_item", "211820" , &m.publishedfileid]));
        steamcmd_params.extend(["+quit"]);
        steamcmd_params
    };

    info!("Count steamcmd parameters: {:?}", &steamcmd_params);
    
    // Reset the directory to prevent previous downloads
    tokio::fs::remove_dir_all(force_install_dir).await?;
    tokio::fs::create_dir(force_install_dir).await?;

    let (file_io, output_file) = if let Some(log_file) = log_file {
        unsafe {
            let output_file = tokio::fs::File::create(log_file).await?;
            (Stdio::from_raw_fd(output_file.as_fd().as_raw_fd()), Some(output_file) )
        }
    } else {
        ( Stdio::null(), None )
    };

    let _cmd = tokio::process::Command::new("steamcmd")
        .args(steamcmd_params)
        .stdout(file_io)
        .spawn()?
        .wait().await;

    std::mem::drop(output_file);

    Ok(())
}

pub async fn download_collection<P: AsRef<Path>>(instance: Instance, force_install_dir: P) -> Result<()> {
    download_collection_internal(instance, force_install_dir.as_ref()).await
}

async fn download_collection_internal(instance: Instance, force_install_dir: &Path) -> Result<()> {

    let client = reqwest::Client::new();
    let collection_id = instance
        .collection_id()
        .ok_or(anyhow!("No Collection Assigned"))?;
    let mut collections: Vec<String> = vec![collection_id.to_string()];

    let mods_folder = instance.folder_path().join("mods");
    if !mods_folder.exists() {
        tokio::fs::create_dir(&mods_folder).await?;
    }

    let mods_on_disk: Vec<String> = collect_installed_mods(&mods_folder).await?;

    let mods_in_collections = collect_mods_from_collections(&client, &mut collections).await?;
    let mods_in_collections: Vec<ModManifestMod> = get_versioned_mods(&client, &mods_in_collections).await?;

    let mods_in_manifest: Vec<ModManifestMod> = match read_manifest(&mods_folder).await {
        Ok(v) => v.mods,
        Err(e) => {
            error!("Missing manifest json: {e}");
            vec![]
        }
    };
    debug!("Mods on disk: ({}) {:?}", mods_on_disk.len(), mods_on_disk);
    debug!("Mods in manifest: ({}) {:?}", mods_in_manifest.len(), mods_in_manifest);

    let mods_on_disk_map: FxHashSet<_> = mods_on_disk.iter().cloned().collect();
    let mods_in_manifest_map: FxHashMap<_, _> = mods_in_manifest.iter().map(|m| (m.publishedfileid.clone(), m.version)).collect();
    let mods_in_collection_map: FxHashMap<_, _> = mods_in_collections.iter().map(|m| (m.publishedfileid.clone(), m.version)).collect();

    info!("Count mods in collection: {}", mods_in_collection_map.len());
    info!("Count mods on disk: {}", mods_on_disk_map.len());
    info!("Count mods in manifest: {}", mods_in_manifest_map.len());

    let should_be_removed = |currently_installed_mod: &String| -> bool {
        !mods_in_collection_map.contains_key(currently_installed_mod)
            // || currently_installed_mod.version.lt(mods_in_collection_map.get(&currently_installed_mod.publishedfileid).unwrap())
    };

    let should_be_installed = |to_be_installed_mod: &&ModManifestMod| -> bool {
        !mods_on_disk_map.contains(&to_be_installed_mod.publishedfileid)
            || !mods_in_manifest_map.contains_key(&to_be_installed_mod.publishedfileid)
            || to_be_installed_mod.version.gt(mods_in_manifest_map.get(&to_be_installed_mod.publishedfileid).unwrap())
    };

    for m in &mods_in_manifest {
        if !mods_on_disk_map.contains(&m.publishedfileid) {
            info!("Missing {}", m.publishedfileid);
        }
    }

    let mods_to_install: Vec<ModManifestMod> = mods_in_collections.iter().filter(should_be_installed).cloned().collect(); 
    // let mut mods_to_remove: FxHashSet<String> = mods_in_manifest.into_iter()
    //     .map(|e|e.publishedfileid)
    //     .filter(should_be_removed)
    //     .collect();
    // mods_to_remove.extend(mods_on_disk.into_iter().filter(should_be_removed));
    // let mods_to_remove: Vec<String> = mods_to_remove.into_iter().collect();
    let mods_to_remove: Vec<String> = mods_on_disk.into_iter().filter(should_be_removed).collect();

    info!("Mods to install: {}", mods_to_install.len());
    info!("Mods to remove: {}", mods_to_remove.len());

    let steamcmd_log_file = mods_folder.join("steamcmd.log");
    if !mods_to_install.is_empty() {
        download_mods_from_workshop(force_install_dir, &mods_to_install, Some(&steamcmd_log_file)).await?;
        info!("Finished Downloading Mods!");
    }

    // Steamcmd stores mods as
    // 'workshop/content/211820/{publishedfileid}/{name}[.pak]'
    let mods_dir = force_install_dir.join("steamapps/workshop/content/211820/");

    for m in mods_to_remove {
        let mod_id = &m;
        let pak_path = mods_folder.join(mod_id);
        let pak_file_path = mods_folder.join(format!("{}.pak", mod_id));
        if pak_path.is_dir() {
            tokio::fs::remove_dir_all(pak_path).await?;
        } else if pak_file_path.is_file() {
            tokio::fs::remove_file(pak_file_path).await?;
        }
    }

    info!("Finished Removing Mods");

    for m in mods_to_install {
        let mod_id = &m.publishedfileid;
        let mod_path = mods_dir.join(mod_id);
        // We do not care for symlinks, we are downloading shit from steamcmd
        if !mod_path.is_dir() {
            continue;
        }
        for mod_file in mod_path.read_dir()? {
            let pak_path = mod_file?.path();

            if pak_path.is_file() && pak_path.extension().is_some_and(|ext| ext.eq("pak")) {
                tokio::fs::rename(pak_path, mods_folder.join(format!("{}.pak", mod_id))).await?;
            } else if pak_path.is_dir() {
                tokio::fs::rename(pak_path, mods_folder.join(mod_id)).await?;
            }
        }
        tokio::fs::remove_dir_all(mod_path).await?;
    }
    info!("Finished Moving Mods");

    // Overwrite current manifest with mods defined in collection
    let new_manifest = ModManifestJson { mods: mods_in_collections };
    write_manifest(&mods_folder, &new_manifest).await?;
    info!("Wrote new manifest to mods folder");

    Ok(())
}
