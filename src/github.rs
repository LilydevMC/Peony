use std::{env, fs};
use std::process::Command;
use anyhow::anyhow;

use crate::models::{github::*, GithubConfig, project_type::modpack::{
    config::ModpackConfig,
    PackFile
}, util::OutputFileInfo, version::VersionInfo};
use crate::models::project_type::mc_mod::config::ModConfig;
use crate::models::project_type::mc_mod::{ModInfo, ModJars};

pub async fn generate_changelog(config: &GithubConfig) -> Result<String, anyhow::Error> {
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
                config.repo_owner,
                config.repo_name
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
        config.repo_owner,
        config.repo_name,
        compare_first
    );


    println!("Successfully generated changelog!");

    Ok(format!("[Full Changelog]({})", full_changelog))
}

pub async fn create_modpack_release(
    config: &ModpackConfig,
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

    let new_release_response = create_github_release(
        &config.github, &new_release_req_body, &github_token
    ).await;

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

pub async fn create_mod_release(
    config: &ModConfig,
    mod_info: &ModInfo,
    mod_jars: &ModJars,
    changelog: &String,
    version_name: &String
) -> Result<(), anyhow::Error> {
    println!("Creating GitHub release...");

    let github_token = match env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(err) => return Err(anyhow!(
            "Failed to get `GITHUB_TOKEN`: {}", err
        ))
    };

    let new_release_req_body = CreateReleaseRequest {
        tag_name: mod_info.version.clone(),
        name: Some(version_name.into()),
        body: Some(changelog.clone())
    };

    let new_release_response = create_github_release(
        &config.github, &new_release_req_body, &github_token
    ).await;

    upload_mod_jars(
        mod_jars,
        &config.github,
        new_release_response.unwrap().id,
        github_token
    ).await
}

pub async fn upload_mod_jars(
    mod_jars: &ModJars,
    github_config: &GithubConfig,
    release_id: i32,
    token: String
) -> Result<(), anyhow::Error> {
    let mod_jar_contents = match fs::read(&mod_jars.mod_jar.file_path) {
        Ok(file_contents) => file_contents,
        Err(err) => return Err(anyhow!(
            "Failed to read mod jar: {}", err
        ))
    };

    let sources_jar_contents = match &mod_jars.sources_jar {
        Some(jar) => match fs::read(&jar.file_path) {
            Ok(file_contents) => Some(file_contents),
            Err(err) => return Err(anyhow!(
                "Failed to read sources jar, even though it exists: {}", err
            ))
        },
        None => None
    };


    // Upload mod jar

    println!("Uploading mod jar as GitHub Release asset `{}`...", &mod_jars.mod_jar.file_name);

    match reqwest::Client::new()
        .post(
            format!(
                "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name=\"{}\"",
                github_config.repo_owner,
                github_config.repo_name,
                release_id,
                &mod_jars.mod_jar.file_name
            )
        )
        .header("User-Agent", env!("CARGO_PKG_NAME"))
        .header("Accept", "application/vnd.github+json")
        .header("Content-Type", "application/java-archive")
        .bearer_auth(&token)
        .body(mod_jar_contents)
        .send().await {
        Ok(_) => {
            println!(
                "Successfully uploaded GitHub Release asset `{}`!", &mod_jars.mod_jar.file_name
            )
        },
        Err(err) => return Err(anyhow!(
            "Failed to upload GitHub release asset `{}`: {}", &mod_jars.mod_jar.file_name, err
        ))
    }

    // Upload sources jar
    match sources_jar_contents {
        Some(file_contents) => {
            println!(
                "Uploading sources jar as GitHub Release asset `{}`...",
                 &mod_jars.sources_jar.clone().unwrap().file_name
            );

            match reqwest::Client::new()
                .post(
                    format!(
                        "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name=\"{}\"",
                        github_config.repo_owner,
                        github_config.repo_name,
                        release_id,
                        &mod_jars.sources_jar.clone().unwrap().file_name
                    )
                )
                .header("User-Agent", env!("CARGO_PKG_NAME"))
                .header("Accept", "application/vnd.github+json")
                .header("Content-Type", "application/java-archive")
                .bearer_auth(&token)
                .body(file_contents)
                .send().await {
                Ok(_) => {
                    println!(
                        "Successfully uploaded GitHub Release asset `{}`!",
                        &mod_jars.sources_jar.clone().unwrap().file_name
                    )
                },
                Err(err) => return Err(anyhow!(
                    "Failed to upload GitHub release asset `{}`: {}",
                    &mod_jars.sources_jar.clone().unwrap().file_name, err
                ))
            }
        },
        None => ()
    }

    Ok(())
}


pub async fn create_github_release(
    config: &GithubConfig,
    new_release_body: &CreateReleaseRequest,
    token: &String
) -> Result<ReleaseResponse, anyhow::Error> {
    match reqwest::Client::new()
        .post(
            format!(
                "https://api.github.com/repos/{}/{}/releases",
                config.repo_owner.clone(),
                config.repo_name.clone()
            )
        )
        .json(&new_release_body)
        .header("User-Agent", env!("CARGO_PKG_NAME"))
        .header("Accept", "application/vnd.github+json")
        .bearer_auth(token)
        .send().await {
        Ok(res) => {
            match res.json::<ReleaseResponse>().await {
                Ok(json) => Ok(json),
                Err(err) => Err(anyhow::Error::from(err))
            }
        },
        Err(err) => {
            return Err(anyhow::Error::from(err))
        }
    }
}

