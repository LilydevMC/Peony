use anyhow::anyhow;
use crate::models::Config;
use crate::models::modrinth::{
    ModrinthUrl,
    version::{
        VersionRequest,
        VersionStatus,
        VersionType
    }
};
use crate::models::pack::PackFile;
use crate::models::util::OutputFileInfo;
use crate::models::version::VersionInfo;

pub async fn create_modrinth_release(
    config: &Config,
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
        loaders: vec![version_info.loader.clone().to_ascii_lowercase()],
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
