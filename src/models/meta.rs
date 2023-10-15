use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub config_format_version: i32,
    pub version_name_format: String,
    pub github: GithubConfig,
    pub modrinth: ModrinthConfig
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubConfig {
    pub repo_owner: String,
    pub repo_name: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthConfig {
    pub project_id: String,
    pub staging: Option<bool>
}
