use serde::{Serialize, Deserialize};
use crate::models::meta::ModrinthConfig;

pub mod project;
pub mod version;

pub struct ModrinthUrl {
    pub labrinth: String,
    pub knossos: String
}

impl ModrinthUrl {
    pub fn new(modrinth_config: &ModrinthConfig) -> Self {
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
