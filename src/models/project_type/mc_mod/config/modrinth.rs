use crate::models::modrinth::version::VersionDependency;
use crate::models::modrinth::DependencyType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthConfig {
    pub project_id: String,
    pub staging: Option<bool>,
    #[serde(rename = "dependency")]
    pub dependencies: Option<Vec<ModrinthDependency>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub dependency_type: DependencyType,
}

impl From<VersionDependency> for ModrinthDependency {
    fn from(dep: VersionDependency) -> Self {
        Self {
            version_id: dep.version_id,
            project_id: dep.project_id,
            dependency_type: dep.dependency_type,
        }
    }
}
