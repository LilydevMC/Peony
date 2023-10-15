use serde::{Serialize, Deserialize};

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

