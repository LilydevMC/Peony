use clap::ValueEnum;
use crate::models::{
    modrinth::{DependencyType, Loader},
    project_type::mc_mod::config::modrinth::ModrinthDependency,
};
use serde::{Deserialize, Serialize};


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
    pub loaders: Vec<Loader>,
    pub featured: bool,
    pub requested_status: VersionStatus,
    pub project_id: String,
    pub file_parts: Vec<String>,
    pub primary_file: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Serialize, Deserialize, Clone, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionStatus {
    Listed,
    Archived,
    Draft,
    Unlisted,
}

impl From<ModrinthDependency> for VersionDependency {
    fn from(dep: ModrinthDependency) -> Self {
        Self {
            version_id: dep.version_id,
            project_id: dep.project_id,
            file_name: None,
            dependency_type: dep.dependency_type,
        }
    }
}
