use crate::models::modrinth::{DonationObject, GalleryObject, LicenseObject};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Mod,
    Plugin,
    Datapack,
    Shader,
    Resourcepack,
    Modpack,
}

impl ProjectType {
    pub fn formatted(&self) -> String {
        match self {
            Self::Mod => "Mod",
            Self::Plugin => "Plugin",
            Self::Datapack => "Data Pack",
            Self::Shader => "Shader",
            Self::Resourcepack => "Resource Pack",
            Self::Modpack => "Modpack",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub slug: String,
    pub title: String,
    #[serde(rename = "description")]
    pub summary: String,
    pub categories: Option<Vec<String>>,
    pub client_side: String,
    pub server_side: String,
    #[serde(rename = "body")]
    pub description: Option<String>,
    pub status: String,
    pub additional_categories: Option<Vec<String>>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Vec<DonationObject>,
    pub project_type: ProjectType,
    pub downloads: i32,
    pub icon_url: Option<String>,
    pub color: Option<i32>,
    pub thread_id: String,
    pub monetization_status: String,
    pub id: String,
    pub team: String,
    pub organization: Option<String>,
    pub body_url: Option<String>,
    pub moderator_message: Option<String>, // replaced with threads for newer projects
    pub published: String,
    pub updated: String,
    pub approved: Option<String>,
    pub queued: Option<String>,
    pub followers: i32,
    pub license: LicenseObject,
    pub versions: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub gallery: Vec<GalleryObject>,
}
