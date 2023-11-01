use serde::{Serialize, Deserialize};
use crate::models::{
    DiscordConfig,
    GithubConfig,
    modrinth::Loader,
    project_type::mc_mod::config::modrinth::ModrinthConfig
};

pub mod modrinth;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModConfig {
    pub config_format_version: i32,
    pub version_name_format: String,
    pub loaders: Vec<Loader>,
    pub mc_versions: Vec<String>,
    pub mc_version_alias: String,
    pub github: GithubConfig,
    pub modrinth: ModrinthConfig,
    pub discord: Option<DiscordConfig>
}

