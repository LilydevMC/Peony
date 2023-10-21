use serde::{Serialize, Deserialize};
use crate::models::meta::ModrinthConfig;

pub struct ModrinthUrl {
    pub labrinth: String,
    pub knossos: String
}

impl ModrinthUrl {
    pub fn new(&self, modrinth_config: ModrinthConfig) -> Self {
        let knossos_url = match modrinth_config.staging {
            Some(is_staging) => match is_staging {
                true => "https://staging.modrinth.com",
                false => "https://modrinth.com"
            },
            None => "https://modrinth.com"
        };

        let labrinth_url = match modrinth_config.staging {
            Some(is_staging) => match is_staging {
                true => "https://staging-api.modrinth.com/v2",
                false => "https://api.modrinth.com/v2"
            },
            None => "https://api.modrinth.com/v2"
        };

        Self {
            knossos: knossos_url.to_owned(),
            labrinth: labrinth_url.to_owned()
        }
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
    pub project_type: String,
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
    pub gallery: Vec<GalleryObject>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DonationObject {
    pub id: String,
    pub platform: String,
    pub url: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseObject {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GalleryObject {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: String,
    pub ordering: i32
}

// Based on the `Create Version` schema here:
// https://docs.modrinth.com/api-spec#tag/versions/operation/createVersion
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionRequest {
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub dependencies: Vec<VersionDependency>,
    pub game_versions: Vec<String>,
    pub version_type: VersionType,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub requested_status: VersionStatus,
    pub project_id: String,
    pub file_parts: Vec<String>,
    pub primary_file: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: DependencyType
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VersionType {
    #[serde(rename = "release")]
    RELEASE,
    #[serde(rename = "beta")]
    BETA,
    #[serde(rename = "alpha")]
    ALPHA
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VersionStatus {
    #[serde(rename = "listed")]
    LISTED,
    #[serde(rename = "archived")]
    ARCHIVED,
    #[serde(rename = "draft")]
    DRAFT,
    #[serde(rename = "unlisted")]
    UNLISTED
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DependencyType {
    #[serde(rename = "required")]
    REQUIRED,
    #[serde(rename = "optional")]
    OPTIONAL,
    #[serde(rename = "incompatible")]
    INCOMPATIBLE,
    #[serde(rename = "embedded")]
    EMBEDDED
}

