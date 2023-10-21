
// pub fn upload_modrinth() -> Result<(), anyhow::Error> {
//     let modrinth_config = config_file.modrinth;
//
//     println!("Uploading to Modrinth...");
//
//     let modrinth_req = VersionRequest {
//         name: version_name.clone(),
//         version_number: pack_file.version,
//         changelog: Some(changelog_markdown.to_owned()),
//         dependencies: vec![],
//         game_versions: vec![pack_file.versions.minecraft],
//         version_type: VersionType::RELEASE,
//         loaders: vec![loader.to_string().to_ascii_lowercase()],
//         featured: false,
//         requested_status: VersionStatus::LISTED,
//         project_id: modrinth_config.project_id,
//         file_parts: vec!["file".to_string()],
//         primary_file: output_file_info.file_name.clone(),
//     };
//
//     let modrinth_token = match env::var("MODRINTH_TOKEN") {
//         Ok(token) => token,
//         Err(err) => return Err(anyhow!(
//                     "Failed to get `MODRINTH_TOKEN`: {}", err
//                 ))
//     };
//
//     let file_part = match reqwest::multipart::Part::bytes(mrpack_file_contents)
//         .file_name(output_file_info.file_name.clone())
//         .mime_str("application/zip") {
//         Ok(part) => part,
//         Err(err) => return Err(anyhow!(
//                     "Failed to get part from .mrpack file: {}", err
//                 ))
//     };
//
//     let form = reqwest::multipart::Form::new()
//         .text("data", serde_json::to_string(&modrinth_req).unwrap())
//         .part("file", file_part);
//
//     let knossos_url = match modrinth_config.staging {
//         Some(is_staging) => match is_staging {
//             true => "https://staging.modrinth.com",
//             false => "https://modrinth.com"
//         },
//         None => "https://modrinth.com"
//     };
//
//     let labrinth_url = match modrinth_config.staging {
//         Some(is_staging) => match is_staging {
//             true => "https://staging-api.modrinth.com/v2",
//             false => "https://api.modrinth.com/v2"
//         },
//         None => "https://api.modrinth.com/v2"
//     };
//
//     let req = match reqwest::Client::new()
//         .post(format!("{}/version", labrinth_url))
//         .header("Authorization", &modrinth_token)
//         .multipart(form)
//         .send().await {
//         Ok(res) => res,
//         Err(err) => return Err(anyhow!("Error uploading version: {}", err))
//     };
//
//     if req.status().is_success() {
//         println!("Successfully uploaded version to Modrinth!");
//     } else {
//         return Err(anyhow!(
//                     "Failed to upload version to Modrinth: {}",
//                     req.text().await.unwrap()
//                 ))
//     }
// }
