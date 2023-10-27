use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub mod config;
pub mod version;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModJars {
    pub mod_jar: Jar,
    pub sources_jar: Option<Jar>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jar {
    pub file_name: String,
    pub file_path: PathBuf
}
