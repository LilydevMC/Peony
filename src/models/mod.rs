use serde::{Deserialize, Serialize};

pub mod github;
pub mod modrinth;
pub mod project_type;
pub mod util;
pub mod version;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubConfig {
    pub repo_owner: String,
    pub repo_name: String,
}

// Eventually should be moved to models::project_type::modpack::config
// unnecessary for now, but should be moved before project types other than modpacks
// and mods are implemented.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthConfig {
    pub project_id: String,
    pub staging: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordConfig {
    pub github_emoji_id: String,
    pub modrinth_emoji_id: String,
    pub discord_ping_role: String,
    pub title_emoji: String,
    pub embed_image_url: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub embed_color: Option<u32>,
}
