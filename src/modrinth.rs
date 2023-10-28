use anyhow::anyhow;
use reqwest::multipart::{Form, Part};
use serde::{Serialize, Deserialize};
use crate::models::{
    modrinth::{
        ModrinthUrl,
        version::{
            VersionRequest,
            VersionStatus,
            VersionType,
        },
    },
    project_type::modpack::{
        config::ModpackConfig,
        PackFile
    },
    util::OutputFileInfo,
    version::VersionInfo
};
use crate::models::project_type::mc_mod::config::ModConfig;
use crate::models::project_type::mc_mod::version::ModVersionInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileParts {
    pub mod_file: JarPart,
    pub sources_file: Option<JarPart>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JarPart {
    pub file_name: String,
    pub file_part: Part
}


pub async fn create_modpack_release(
    config: &ModpackConfig,
    pack_file: &PackFile,
    output_file_info: &OutputFileInfo,
    version_info: &VersionInfo,
    changelog: &String,
    modrinth_token: String,
    modrinth_url: &ModrinthUrl
) -> Result<(), anyhow::Error> {
    let modrinth_config = config.modrinth.clone();

    println!("Uploading to Modrinth...");

    let modrinth_req = VersionRequest {
        name: version_info.version_name.clone(),
        version_number: pack_file.version.clone(),
        changelog: Some(changelog.to_string()),
        dependencies: vec![],
        game_versions: vec![pack_file.versions.minecraft.clone()],
        version_type: VersionType::RELEASE,
        loaders: vec![version_info.loader.clone()],
        featured: false,
        requested_status: VersionStatus::LISTED,
        project_id: modrinth_config.project_id,
        file_parts: vec!["file".to_string()],
        primary_file: output_file_info.file_name.clone(),
    };

    let file_part = match Part::bytes(
        version_info.file_contents.clone()
    )
        .file_name(output_file_info.file_name.clone())
        .mime_str("application/zip") {
        Ok(part) => part,
        Err(err) => return Err(anyhow!(
            "Failed to get part from .mrpack file: {}", err
        ))
    };

    let form = Form::new()
        .text("data", serde_json::to_string(&modrinth_req).unwrap())
        .part("file", file_part);

    let req = match reqwest::Client::new()
        .post(format!("{}/version", modrinth_url.labrinth))
        .header("Authorization", &modrinth_token)
        .multipart(form)
        .send().await {
        Ok(res) => res,
        Err(err) => return Err(anyhow!("Error uploading version: {}", err))
    };

    return if req.status().is_success() {
        Ok(println!("Successfully uploaded version to Modrinth!"))
    } else {
        Err(anyhow!(
            "Failed to upload version to Modrinth: {}",
            req.text().await.unwrap()
        ))
    }
}

pub async fn create_mod_release(
    config: &ModConfig,
    mod_files: &ModVersionInfo,
    changelog: &String,
    modrinth_token: String,
    modrinth_url: &ModrinthUrl,
    version_name: &String
) -> Result<(), anyhow::Error> {
    let modrinth_config = config.modrinth.clone();

    println!("Uploading to Modrinth...");

    let modrinth_req = VersionRequest {
        name: version_name.into(),
        version_number: pack_file.version.clone(),
        changelog: Some(changelog.to_string()),
        dependencies: vec![],
        game_versions: vec![pack_file.versions.minecraft.clone()],
        version_type: VersionType::RELEASE,
        loaders: vec![version_info.loader.clone()],
        featured: false,
        requested_status: VersionStatus::LISTED,
        project_id: modrinth_config.project_id,
        file_parts: vec!["file".to_string()],
        primary_file: output_file_info.file_name.clone(),
    };

    let file_part = match reqwest::multipart::Part::bytes(
        version_info.file_contents.clone()
    )
        .file_name(output_file_info.file_name.clone())
        .mime_str("application/zip") {
        Ok(part) => part,
        Err(err) => return Err(anyhow!(
                    "Failed to get part from .mrpack file: {}", err
                ))
    };

    let form = reqwest::multipart::Form::new()
        .text("data", serde_json::to_string(&modrinth_req).unwrap())
        .part("file", file_part);

    let req = match reqwest::Client::new()
        .post(format!("{}/version", modrinth_url.labrinth))
        .header("Authorization", &modrinth_token)
        .multipart(form)
        .send().await {
        Ok(res) => res,
        Err(err) => return Err(anyhow!("Error uploading version: {}", err))
    };

    return if req.status().is_success() {
        Ok(println!("Successfully uploaded version to Modrinth!"))
    } else {
        Err(anyhow!(
            "Failed to upload version to Modrinth: {}",
            req.text().await.unwrap()
        ))
    }
}

pub async fn get_mod_file_parts(
    mod_files: &ModVersionInfo
) -> Result<FileParts, anyhow::Error> {
    let mod_part = JarPart {
        file_name: mod_files.mod_file.name.clone(),
        file_part: Part::bytes(&mod_files.mod_file.contents)
            .file_name(&mod_files.mod_file.name)
            .mime_str("application/java-archive")?
    };

    let sources_part = match &mod_files.sources_file {
        Some(file) => {
            let part = Part::bytes(&file.contents)
                .file_name(&file.name)
                .mime_str("application/java-archive")?;

            Some(
                JarPart {
                    file_name: file.name.into(),
                    file_part: part
                }
            )
        },
        None => None
    };

    Ok(FileParts {
        mod_file: mod_part,
        sources_file: sources_part
    })
}

pub async fn create_mod_form(
    mod_parts: &FileParts,
    form_data: &String
) -> Result<Form, anyhow::Error> {

    match &mod_parts.sources_file {
        Some(sources_part) => {
            Ok(
                Form::new()
                    .text("data", form_data)
                    .part("mod_jar", mod_parts.mod_file.file_part.into())
                    .part("sources_jar", sources_part.file_part.into())
            )
        },
        None => {
            Ok(
                Form::new()
                    .text("data", form_data)
                    .part("mod_jar", mod_parts.mod_file.file_part.into())
            )
        }
    }

}
