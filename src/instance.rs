use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

use crate::json::InstanceDataJson;

pub enum ModifyInstance {
    Name(String),
    Executable(Option<String>),
    Collection(Option<String>)
}

#[derive(Clone)]
pub struct Instance {
    folder_path: PathBuf,
    json: InstanceDataJson,
}

impl Instance {

    pub fn from_json(json: InstanceDataJson, path: &Path) -> Result<Self> {
        let folder_path = path.parent().ok_or(anyhow!("Instance not in a valid folder??"))?.to_owned();
        Ok(Self {
            folder_path,
            json
        })
    }

    pub fn modify(&mut self, modification: ModifyInstance) {
        match modification {
            ModifyInstance::Name(name) => { self.json.name = name; }
            ModifyInstance::Executable(exec) => { self.json.executable = exec; }
            ModifyInstance::Collection(maybe_collection_id) => { self.json.collection_id = maybe_collection_id; }
        }
    }

    pub fn executable(&self) -> &Option<String> {
        &self.json.executable
    }

    pub fn name(&self) -> &str {
        &self.json.name
    }

    pub fn additional_assets(&self) -> Option<&Vec<String>> {
        self.json.additional_assets.as_ref()
    }

    pub fn collection_id(&self) -> Option<String> {
        self.json.collection_id.clone()
    }

    pub fn folder_path(&self) -> &Path {
        &self.folder_path
    }

    pub fn to_json(&self) -> &InstanceDataJson {
        &self.json
    }
}
