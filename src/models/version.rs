use crate::models::modrinth::Loader;

pub struct VersionInfo {
    pub loader: Loader,
    pub version_name: String,
    pub file_contents: Vec<u8>,
}
