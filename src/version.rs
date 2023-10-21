use std::fs;
use anyhow::anyhow;
use crate::models::meta::Config;
use crate::models::pack::PackFile;
use crate::models::util::OutputFileInfo;
use crate::models::version::VersionInfo;

pub fn get_version_info(
    config_file: &Config,
    pack_file: &PackFile,
    output_info: &OutputFileInfo
) -> Result<VersionInfo, anyhow::Error> {
    let loader_opt = if pack_file.versions.quilt.is_some() {
        Some("Quilt")
    } else if pack_file.versions.fabric.is_some() {
        Some("Fabric")
    } else if pack_file.versions.forge.is_some() {
        Some("Forge")
    } else if pack_file.versions.liteloader.is_some() {
        Some("LiteLoader")
    } else {
        None
    };

    let loader = match loader_opt {
        Some(loader) => loader,
        None => return Err(anyhow!("Failed to parse loader name into string"))
    };

    let version_name = config_file.version_name_format
        .replace("%project_name%", &pack_file.name)
        .replace("%project_version%", &pack_file.version)
        .replace("%mc_version%", &pack_file.versions.minecraft)
        .replace("%loader%", &loader);

    let file_contents = match fs::read(output_info.file_path.clone()) {
        Ok(file) => file,
        Err(err) => return Err(anyhow!(
            "Failed to read .mrpack file: {}", err
        ))
    };

    Ok(
        VersionInfo {
            version_name,
            loader: loader.to_owned(),
            file_contents
        }
    )
}