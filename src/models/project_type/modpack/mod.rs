use serde::{Deserialize, Serialize};

pub mod config;

// Based on packwiz's pack.toml format here:
// https://packwiz.infra.link/reference/pack-format/pack-toml/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackFile {
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: String,
    #[serde(rename = "pack-format")]
    pub pack_format: String,
    pub index: PackFileIndex,
    pub versions: PackFileVersions,
    pub options: Option<PackFileOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackFileIndex {
    pub file: String,
    #[serde(rename = "hash-format")]
    pub hash_format: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackFileVersions {
    pub minecraft: String,
    pub quilt: Option<String>,
    pub fabric: Option<String>,
    pub forge: Option<String>,
    pub liteloader: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackFileOptions {
    #[serde(rename = "acceptable-game-versions")]
    pub acceptable_game_versions: Option<Vec<String>>,
    #[serde(rename = "mods-folder")]
    pub mods_folder: Option<String>,
    #[serde(rename = "meta-folder")]
    pub meta_folder: Option<String>,
    #[serde(rename = "meta-folder-base")]
    pub meta_folder_base: Option<String>,
    #[serde(rename = "no-internal-hashes")]
    pub no_internal_hashes: Option<bool>,
    #[serde(rename = "datapack-folder")]
    pub datapack_folder: Option<String>,
}
