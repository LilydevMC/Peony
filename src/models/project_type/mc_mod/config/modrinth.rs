use serde::{Serialize, Deserialize};
use crate::models::modrinth::DependencyType;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthConfig {
    pub project_id: String,
    pub staging: Option<bool>,
    #[serde(rename = "dependency")]
    pub dependencies: Vec<ModrinthDependency>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthDependency {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub dependency_type: DependencyType
}

