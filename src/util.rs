use std::{env, fs};
use std::fs::File;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use crate::models::util::TempInfo;

pub fn create_temp() -> Result<TempInfo, anyhow::Error> {
    let new_uuid = uuid::Uuid::new_v4();
    let new_tmp_dir_name = format!("{}_{}", env!("CARGO_PKG_NAME"), new_uuid);
    let new_tmp_dir = Path::new(env::temp_dir().as_path())
        .join(&new_tmp_dir_name);

    let current_dir = match env::current_dir() {
        Ok(dir) => dir,
        Err(err) => return Err(anyhow!("Failed to get current directory: {}", err))
    };

    match fs::create_dir(&new_tmp_dir) {
        Ok(_) => (),
        Err(err) => return Err(anyhow!(
            "Failed to create temporary directory: {}", err
        ))
    }

    let copy_files_res: Result<(), anyhow::Error> = match fs_extra::dir::copy(
        current_dir,
        &new_tmp_dir,
        &fs_extra::dir::CopyOptions::new().content_only(true)
    ) {
        Ok(_) => Ok(()),
        Err(err) => return Err(anyhow!(
            "Failed to copy files to temporary directory: {}", err
        ))
    };

    match copy_files_res {
        Ok(_) => {
            Ok(
                TempInfo {
                    dir_name: new_tmp_dir_name.to_owned(),
                    dir_path: new_tmp_dir
                }
            )
        },
        Err(err) => return Err(anyhow!(
            "Failed to get temp directory info: {}", err
        ))
    }

}

pub fn clean_up(tmp_dir: &PathBuf) -> Result<(), anyhow::Error> {
    println!("Cleaning up...");

    match fs_extra::dir::remove(tmp_dir) {
        Ok(_) => {
            println!("Removed temporary directory!");
            Ok(())
        },
        Err(err) => return Err(anyhow!(
            "Failed to remove temporary directory: {}", err
        ))
    }
}

pub fn file_exists_in_zip(archive: &mut zip::ZipArchive<File>, file_name: &str) -> bool {
    if archive.by_name(file_name).is_ok() {
        true
    } else {
        false
    }
}

pub fn file_name_from_path(path: &PathBuf) -> Result<String, anyhow::Error> {
    match path.clone().file_name() {
        Some(os_name) => match os_name.to_str() {
            Some(name) => Ok(name.to_string()),
            None => return Err(anyhow!("Failed to parse file name from OsString to &str"))
        },
        None => return Err(anyhow!("Failed to get file name from path: {:?}", path))
    }
}

pub fn trim_quotes(string: String) -> String {
    string.trim_matches(&['\"'] as &[_]).to_string()
}

pub fn read_file(path: &PathBuf) -> Result<Vec<u8>, anyhow::Error> {
    match fs::read(path) {
        Ok(file) => Ok(file),
        Err(err) => Err(anyhow!(
            "Failed to read file: {}", err
        ))
    }
}

