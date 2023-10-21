use serde::{Serialize, Deserialize};

pub mod github;
pub mod meta;
pub mod modrinth;
pub mod pack;
pub mod util;
pub mod version;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub config_format_version: i32,
    pub version_name_format: String,
    pub github: GithubConfig,
    pub modrinth: ModrinthConfig,
    pub discord: Option<DiscordConfig>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubConfig {
    pub repo_owner: String,
    pub repo_name: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthConfig {
    pub project_id: String,
    pub staging: Option<bool>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordConfig {
    pub github_emoji_id: String,
    pub modrinth_emoji_id: String,
    pub discord_ping_role: String,
    pub title_emoji: String,
    pub embed_image_url: String,
    pub embed_color: Option<u32>
}
