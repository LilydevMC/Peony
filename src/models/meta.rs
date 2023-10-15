use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub config_format_version: i32,
    pub version_name_format: String,
    pub modrinth: Option<ModrinthConfig>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthConfig {
    pub project_id: String
}
