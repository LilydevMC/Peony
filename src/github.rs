use std::env;
use std::process::Command;
use anyhow::anyhow;
use crate::models::github::{CreateReleaseRequest, ReleaseResponse};
use crate::models::Config;
use crate::models::pack::PackFile;
use crate::models::util::OutputFileInfo;
use crate::models::version::VersionInfo;

pub async fn generate_changelog(config: &Config) -> Result<String, anyhow::Error> {
    println!("Generating changelog...");

    let first_commit = match Command::new("git")
        .args(["rev-list", "--max-parents=0", "HEAD"]).output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(output_string) => output_string.replace("\n", ""),
            Err(err) => return Err(anyhow!(
                    "Failed to parse git output: {}", err
                ))
        },
        Err(err) => return Err(anyhow!(
                    "Failed to get first commit: {}", err
                ))
    };

    let latest_release = match reqwest::Client::new()
        .get(
            format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                config.github.repo_owner,
                config.github.repo_name
            )
        )
        .header("User-Agent", env!("CARGO_PKG_NAME"))
        .send().await {
        Ok(res) => {
            match res.json::<ReleaseResponse>().await {
                Ok(json) => Some(json),
                Err(_) => None
            }
        },
        Err(_) => None
    };


    let compare_first = match latest_release {
        Some(release) => release.tag_name,
        None => first_commit
    };


    let full_changelog = format!(
        "https://github.com/{}/{}/compare/{}..HEAD",
        config.github.repo_owner,
        config.github.repo_name,
        compare_first
    );


    println!("Successfully generated changelog!");

    Ok(format!("[Full Changelog]({})", full_changelog))
}

pub async fn create_github_release(
    config: &Config,
    pack_file: &PackFile,
    output_file_info: &OutputFileInfo,
    version_info: &VersionInfo,
    changelog: &String
) -> Result<(), anyhow::Error> {
    println!("Creating GitHub release...");

    let github_token = match env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(err) => return Err(anyhow!(
                    "Failed to get `GITHUB_TOKEN`: {}", err
                ))
    };

    let new_release_req_body = CreateReleaseRequest {
        tag_name: pack_file.version.clone(),
        name: Some(version_info.version_name.clone()),
        body: Some(changelog.clone())
    };

    let new_release_response = match reqwest::Client::new()
        .post(
            format!(
                "https://api.github.com/repos/{}/{}/releases",
                config.github.repo_owner.clone(),
                config.github.repo_name.clone()
            )
        )
        .json(&new_release_req_body)
        .header("User-Agent", env!("CARGO_PKG_NAME"))
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(github_token.clone())
        .send().await {
        Ok(res) => {
            match res.json::<ReleaseResponse>().await {
                Ok(json) => Ok(json),
                Err(err) => Err(err)
            }
        },
        Err(err) => {
            return Err(anyhow::Error::from(err))
        }
    };

    return match new_release_response {
        Ok(release_res) => {
            println!("Successfully created GitHub release!");
            println!("Uploading release asset to GitHub release...");

            match reqwest::Client::new()
                .post(
                    format!(
                        "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name=\"{}\"",
                        config.github.repo_owner,
                        config.github.repo_name,
                        release_res.id,
                        &output_file_info.file_name
                    )
                )
                .header("User-Agent", env!("CARGO_PKG_NAME"))
                .header("Accept", "application/vnd.github+json")
                .header("Content-Type", "application/zip")
                .bearer_auth(github_token)
                .body(version_info.file_contents.clone())
                .send().await {
                Ok(_) => {
                    println!("Successfully uploaded release asset!");
                    Ok(())
                },
                Err(err) => Err(anyhow::Error::from(err))
            }
        },
        Err(err) => Err(anyhow::Error::from(err))
    }
}
