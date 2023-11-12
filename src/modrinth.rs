use crate::models::modrinth::version::VersionDependency;
use crate::models::project_type::mc_mod::config::ModConfig;
use crate::models::project_type::mc_mod::version::ModVersionInfo;
use crate::models::{
    modrinth::{
        version::{VersionRequest, VersionStatus, VersionType},
        ModrinthUrl,
    },
    project_type::modpack::{config::ModpackConfig, PackFile},
    util::OutputFileInfo,
    version::VersionInfo,
};
use anyhow::anyhow;
use reqwest::multipart::{Form, Part};
use std::env;

#[derive(Debug)]
pub struct VersionForm {
    pub form: Form,
    pub part_names: Vec<String>,
}

#[derive(Debug)]
pub struct JarPart {
    pub file_name: String,
    pub file_part: Part,
    pub file_type: FileType,
}

#[derive(Debug, Copy, Clone)]
pub enum FileType {
    Mod,
    Sources,
}

impl FileType {
    pub fn part_name(&self) -> String {
        match self {
            Self::Mod => "mod_jar",
            Self::Sources => "sources_jar",
        }
        .to_string()
    }
}

pub async fn create_modpack_release(
    config: &ModpackConfig,
    pack_file: &PackFile,
    output_file_info: &OutputFileInfo,
    version_info: &VersionInfo,
    changelog: &String,
    modrinth_token: String,
    modrinth_url: &ModrinthUrl,
    version_type: VersionType
) -> Result<(), anyhow::Error> {
    let modrinth_config = config.modrinth.clone();

    println!("Uploading to Modrinth...");

    let modrinth_req = VersionRequest {
        name: version_info.version_name.clone(),
        version_number: pack_file.version.clone(),
        changelog: Some(changelog.to_string()),
        dependencies: vec![],
        game_versions: vec![pack_file.versions.minecraft.clone()],
        version_type,
        loaders: vec![version_info.loader],
        featured: false,
        requested_status: VersionStatus::Listed,
        project_id: modrinth_config.project_id,
        file_parts: vec!["file".to_string()],
        primary_file: output_file_info.file_name.clone(),
    };

    let file_part = match Part::bytes(version_info.file_contents.clone())
        .file_name(output_file_info.file_name.clone())
        .mime_str("application/zip")
    {
        Ok(part) => part,
        Err(err) => return Err(anyhow!("Failed to get part from .mrpack file: {}", err)),
    };

    let form = Form::new()
        .text("data", serde_json::to_string(&modrinth_req).unwrap())
        .part("file", file_part);

    let req = match reqwest::Client::new()
        .post(format!("{}/version", modrinth_url.labrinth))
        .header("Authorization", &modrinth_token)
        .multipart(form)
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => return Err(anyhow!("Error uploading version: {}", err)),
    };

    if req.status().is_success() {
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
    modrinth_url: &ModrinthUrl,
    version_name: &String,
    version_type: VersionType
) -> Result<(), anyhow::Error> {
    let modrinth_config = config.modrinth.clone();
    let modrinth_token = match env::var("MODRINTH_TOKEN") {
        Ok(token) => token,
        Err(err) => {
            return Err(anyhow!(
                "Failed to get Modrinth token from environment: {}",
                err
            ))
        }
    };

    println!("Uploading to Modrinth...");

    let mut file_part_names = vec![FileType::Mod.part_name()];

    if mod_files.sources_file.is_some() {
        file_part_names.push(FileType::Sources.part_name())
    }

    let mut dependencies: Vec<VersionDependency> = vec![];

    if modrinth_config.dependencies.is_some() {
        for dep in modrinth_config.dependencies.unwrap() {
            dependencies.push(VersionDependency::from(dep))
        }
    }

    let form_data = VersionRequest {
        name: version_name.into(),
        version_number: mod_files.version.to_owned(),
        changelog: Some(changelog.to_string()),
        dependencies,
        game_versions: config.mc_versions.to_owned(),
        version_type,
        loaders: config.loaders.to_owned(),
        featured: false,
        requested_status: VersionStatus::Listed,
        project_id: modrinth_config.project_id,
        file_parts: file_part_names,
        primary_file: FileType::Mod.part_name(),
    };

    let form = match create_mod_form(mod_files, &form_data).await {
        Ok(form) => form,
        Err(err) => return Err(anyhow!("Failed to create mod form: {}", err)),
    };

    let req = match reqwest::Client::new()
        .post(format!("{}/version", modrinth_url.labrinth))
        .header("Authorization", &modrinth_token)
        .multipart(form)
        .send()
        .await
    {
        Ok(res) => res,
        Err(err) => return Err(anyhow!("Error uploading mod version: {}", err)),
    };

    if req.status().is_success() {
        Ok(println!("Successfully uploaded version to Modrinth!"))
    } else {
        Err(anyhow!(
            "Failed to upload version to Modrinth: {}",
            req.text().await.unwrap()
        ))
    }
}

pub async fn create_mod_form(
    mod_files: &ModVersionInfo,
    request_data: &VersionRequest,
) -> Result<Form, anyhow::Error> {
    let mut file_names: Vec<String> = vec![mod_files.mod_file.clone().name];

    let mod_part = JarPart {
        file_name: mod_files.mod_file.name.clone(),
        file_part: Part::bytes(mod_files.mod_file.contents.clone())
            .file_name(mod_files.mod_file.clone().name)
            .mime_str("application/java-archive")?,
        file_type: FileType::Mod,
    };

    let sources_part = match &mod_files.sources_file {
        Some(file) => {
            file_names.push(file.name.to_owned());

            let part = Part::bytes(file.contents.clone())
                .file_name(file.name.clone())
                .mime_str("application/java-archive")?;

            Some(JarPart {
                file_name: file.name.clone(),
                file_part: part,
                file_type: FileType::Sources,
            })
        }
        None => None,
    };

    let mut file_parts = vec![mod_part];

    if let Some(sources_part_val) = sources_part {
        file_parts.push(sources_part_val);
    }

    let form_data = serde_json::to_string(request_data)
        .map_err(|err| anyhow!("Failed to serialize version request body: {}", err))?;

    let mut form = Form::new().text("data", form_data);

    for part in file_parts {
        form = form.part(part.file_type.part_name(), part.file_part);
    }

    Ok(form)
}
