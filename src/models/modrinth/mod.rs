use serde::{Serialize, Deserialize};
use crate::models::ModrinthConfig;

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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    QUILT,
    FABRIC,
    NEOFORGE,
    FORGE,
    LITELOADER
}

impl Loader {
    pub fn formatted(&self) -> String {
        match self {
            Self::QUILT => "Quilt",
            Self::FABRIC => "Fabric",
            Self::NEOFORGE => "NeoForge",
            Self::FORGE => "Forge",
            Self::LITELOADER => "LiteLoader"
        }.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DonationObject {
    pub id: String,
    pub platform: String,
    pub url: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseObject {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryObject {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: String,
    pub ordering: i32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    REQUIRED,
    OPTIONAL,
    INCOMPATIBLE,
    EMBEDDED
}
