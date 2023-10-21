use std::process::Command;
use anyhow::anyhow;
use crate::models::github::ReleaseResponse;
use crate::models::meta::Config;

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


