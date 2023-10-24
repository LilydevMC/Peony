// use std::fs::File;
// use anyhow::anyhow;
// use zip::{ZipArchive, read::ZipFile};
// use crate::util::file_exists_in_zip;
//
// pub fn get_loader_file(archive: &mut ZipArchive<File>) -> Result<ZipFile, anyhow::Error> {
//     if file_exists_in_zip(&mut archive, "fabric.mod.json") {
//         match archive.by_name("fabric.mod.json") {
//             Ok(file) => Ok(file),
//             Err(err) => return Err(anyhow!(
//                 "Failed to get `fabric.mod.json` file from jar: {}", err
//             ))
//         }
//     } else if file_exists_in_zip(&mut archive, "quilt.mod.json") {
//         match archive.by_name("quilt.mod.json") {
//             Ok(file) => Some(file),
//             Err(err) => return Err(anyhow!(
//                 "Failed to get `quilt.mod.json` file from jar: {}", err
//             ))
//         }
//     } else {
//         return Err(anyhow!(
//             "Failed to get `fabric.mod.json` or `quilt.mod.json` from jar"
//         ))
//     }
// }
