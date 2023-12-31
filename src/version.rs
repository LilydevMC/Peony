use crate::models::modrinth::Loader::*;
use crate::models::project_type::modpack::{config::ModpackConfig, PackFile};
use crate::models::util::OutputFileInfo;
use crate::models::version::VersionInfo;
use anyhow::anyhow;
use std::fs;

pub fn get_modpack_version_info(
    config_file: &ModpackConfig,
    pack_file: &PackFile,
    output_info: &OutputFileInfo,
) -> Result<VersionInfo, anyhow::Error> {
    let loader_opt = if pack_file.versions.quilt.is_some() {
        Some(Quilt)
    } else if pack_file.versions.fabric.is_some() {
        Some(Fabric)
    } else if pack_file.versions.forge.is_some() {
        Some(Forge)
    } else if pack_file.versions.liteloader.is_some() {
        Some(Liteloader)
    } else {
        None
    };

    let loader = match loader_opt {
        Some(loader) => loader,
        None => return Err(anyhow!("Failed to parse loader name")),
    };

    let version_name = config_file
        .version_name_format
        .replace("%project_name%", &pack_file.name)
        .replace("%project_version%", &pack_file.version)
        .replace("%mc_version%", &pack_file.versions.minecraft)
        .replace("%loader%", &loader.formatted());

    let file_contents = match fs::read(output_info.file_path.clone()) {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("Failed to read .mrpack file: {}", err)),
    };

    Ok(VersionInfo {
        version_name,
        loader,
        file_contents,
    })
}
