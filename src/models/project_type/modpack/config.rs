use serde::{Serialize, Deserialize};
use crate::models::{DiscordConfig, GithubConfig, ModrinthConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModpackConfig {
    pub config_format_version: i32,
    pub version_name_format: String,
    pub github: GithubConfig,
    pub modrinth: ModrinthConfig,
    pub discord: Option<DiscordConfig>
}