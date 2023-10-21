use std::fs;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use glob::glob;
use crate::models::pack::PackFile;
use crate::models::util::{OutputFileInfo, TempInfo};
use crate::util::clean_up;

pub fn get_pack_file() -> Result<PackFile, anyhow::Error> {
    let file = match fs::read_to_string("pack.toml") {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("Failed to read pack.toml file: {}", err))
    };

    let file_parsed: Result<PackFile, anyhow::Error> = match toml::from_str(file.as_str()) {
        Ok(pack) => Ok(pack),
        Err(err) => Err(anyhow!("Failed to parse pack.toml file: {}", err))
    };

    file_parsed
}

pub fn write_pack_file(dir_path: &PathBuf, file_contents: String) -> Result<(), anyhow::Error> {
    match fs::write(
        Path::new(dir_path).join("pack.toml"),
        file_contents
    ) {
        Ok(_) => Ok(()),
        Err(err) => {
            clean_up(dir_path)
                .expect("Failed to clean up temp directory");
            return Err(anyhow!(
                "Failed to write new pack.toml data: {}", err
            ));
        }
    }
}

pub fn get_output_file(tmp_dir_info: &TempInfo) -> Result<OutputFileInfo, anyhow::Error> {
    // This should work, as there shouldn't be any more than one .mrpack file at a given time
    let glob_pattern = match glob(
        match Path::new(&tmp_dir_info.dir_path).join("*.mrpack").to_str() {
            Some(path) => path,
            None => return Err(anyhow!(
                        "Failed to parse modpack glob to string."
                    ))
        }
    ) {
        Ok(paths) => paths,
        Err(err) => return Err(anyhow!(
                    "Failed to get paths with glob pattern: {}", err
                ))
    };

    let mut mrpack_path_res = None;
    for entry in glob_pattern {
        mrpack_path_res = Some(entry);
        break;
    };
    let file_path = match mrpack_path_res {
        Some(path) => match path {
            Ok(res) => res,
            Err(err) => return Err(anyhow!(
                        "Failed to parse modpack file path: {}", err
                    ))
        },
        None => return Err(anyhow!("Failed to get modpack file path"))
    };
    let file_name = match file_path.file_name() {
        Some(os_name) => match os_name.to_str() {
            Some(name) => name.to_string(),
            None => return Err(anyhow!("Failed to parse file name from OsString to &str"))
        },
        None => return Err(anyhow!("Failed to get mrpack file name"))
    };

    Ok(
        OutputFileInfo {
            file_name,
            file_path
        }
    )
}

