use std::{env, fs};
use std::path::Path;
use std::process::Command;
use anyhow::anyhow;
use chrono::Utc;
use clap::{Parser, Subcommand, command};
use glob::glob;
use serenity::model::channel::Embed;
use serenity::model::webhook::Webhook;
use crate::{
    models::pack::PackFile
};
use crate::models::github::{CreateReleaseRequest, ReleaseResponse};
use crate::models::meta::Config;
use crate::models::modrinth::{VersionRequest, VersionStatus, VersionType};

mod models;


#[derive(Debug, Parser)]
#[command(
    name = "mrpack distributor",
    author,
    version,
    about
)]
struct CliArgs {
    #[command(subcommand)]
    commands: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Runs configurations.")]
    Run {
        #[clap(long, short, help = "Whether or not to send Discord webhook")]
        discord: bool,
        #[clap(long, short, help = "Custom version number")]
        version: Option<String>
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    match dotenvy::dotenv() {
        Ok(_) => (),
        Err(_) => ()
    };

    let args = CliArgs::parse();

    match which::which("packwiz") {
        Ok(_) => (),
        Err(err) => return Err(anyhow!("Failed to find packwiz executable: {}", err))
    }

    match args.commands {
        Commands::Run { discord, version } => {
            if !Path::new("mrpack.toml").exists() {
                return Err(anyhow!("Failed to find `mrpack.toml` file."))
            }

            let config_file = match fs::read_to_string("mrpack.toml") {
                Ok(content_string) => {
                    let parsed_config: Config = match toml::from_str(&*content_string) {
                        Ok(config) => config,
                        Err(err) => return Err(anyhow!(
                            "Failed to parse config file: {}", err
                        ))
                    };
                    parsed_config
                },
                Err(err) => return Err(anyhow!(
                    "Failed to read config file: {}", err
                ))
            };


            let file = match fs::read_to_string("pack.toml") {
                Ok(file) => file,
                Err(err) => return Err(anyhow!("Failed to read pack.toml file: {}", err))
            };

            let file_parsed: PackFile = match toml::from_str(file.as_str()) {
                Ok(pack) => pack,
                Err(err) => return Err(anyhow!("Failed to parse pack.toml file: {}", err))
            };
            let mut pack_file = file_parsed;

            let new_uuid = uuid::Uuid::new_v4();
            let new_tmp_dir_name = format!("{}_{}", env!("CARGO_PKG_NAME"), new_uuid);
            let new_tmp_dir = Path::new(env::temp_dir().as_path())
                .join(new_tmp_dir_name);

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

            match fs_extra::dir::copy(
                current_dir,
                &new_tmp_dir,
                &fs_extra::dir::CopyOptions::new().content_only(true)
            ) {
                Ok(_) => (),
                Err(err) => return Err(anyhow!(
                    "Failed to copy files to temporary directory: {}", err
                ))
            }

            match version {
                Some(ver) => {
                    let mut new_file_contents = pack_file.clone();
                    new_file_contents.version = ver;
                    let file_contents_string = match toml::to_string(
                        &new_file_contents
                    ) {
                        Ok(file) => file,
                        Err(err) => return Err(anyhow!(
                            "Failed to parse new pack data to toml: {}", err
                        ))
                    };

                    pack_file = new_file_contents;

                    match fs::write(
                        Path::new(&new_tmp_dir).join("pack.toml"),
                        file_contents_string
                    ) {
                        Ok(_) => (),
                        Err(err) => return Err(anyhow!(
                            "Failed to write new pack.toml data: {}", err
                        ))
                    }
                },
                None => ()
            }

            match Command::new("packwiz")
                .arg("mr")
                .arg("export")
                .current_dir(&new_tmp_dir).output() {
                Ok(_) => (),
                Err(err) => return Err(anyhow!(
                    "Failed to export with packwiz: {}", err
                ))
            }


            let glob_pattern = match glob(
                match Path::new(&new_tmp_dir).join("*.mrpack").to_str() {
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
            let mrpack_path = match mrpack_path_res {
                Some(path) => match path {
                    Ok(res) => res,
                    Err(err) => return Err(anyhow!(
                        "Failed to parse modpack file path: {}", err
                    ))
                },
                None => return Err(anyhow!("Failed to get modpack file path"))
            };
            let file_name = match mrpack_path.file_name() {
                Some(os_name) => match os_name.to_str() {
                    Some(name) => name.to_string(),
                    None => return Err(anyhow!("Failed to parse file name from OsString to &str"))
                },
                None => return Err(anyhow!("Failed to get mrpack file name"))
            };

            let loader_opt = if pack_file.versions.quilt.is_some() {
                Some("Quilt")
            } else if pack_file.versions.fabric.is_some() {
                Some("Fabric")
            } else if pack_file.versions.forge.is_some() {
                Some("Forge")
            } else if pack_file.versions.liteloader.is_some() {
                Some("LiteLoader")
            } else {
                None
            };

            let loader = match loader_opt {
                Some(loader) => loader,
                None => return Err(anyhow!("Failed to parse loader name into string"))
            };

            let version_name = config_file.version_name_format
                .replace("%project_name%", &pack_file.name)
                .replace("%project_version%", &pack_file.version)
                .replace("%mc_version%", &pack_file.versions.minecraft)
                .replace("%loader%", loader);

            let mrpack_file_contents = match fs::read(&*mrpack_path) {
                Ok(file) => file,
                Err(err) => return Err(anyhow!(
                    "Failed to read .mrpack file: {}", err
                ))
            };

            // Changelog

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
                        config_file.github.repo_owner,
                        config_file.github.repo_name
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
                config_file.github.repo_owner,
                config_file.github.repo_name,
                compare_first
            );

            let changelog_markdown = format!("[Full Changelog]({})", full_changelog);

            println!("Successfully generated changelog!");

            // GitHub Release

            println!("Creating GitHub release...");

            let github_token = match env::var("GITHUB_TOKEN") {
                Ok(token) => token,
                Err(err) => return Err(anyhow!(
                    "Failed to get `GITHUB_TOKEN`: {}", err
                ))
            };

            let new_release_req_body = CreateReleaseRequest {
                tag_name: pack_file.version.clone(),
                name: Some(version_name.clone()),
                body: Some(changelog_markdown.clone())
            };

            let new_release_response = match reqwest::Client::new()
                .post(
                    format!(
                        "https://api.github.com/repos/{}/{}/releases",
                        config_file.github.repo_owner.clone(),
                        config_file.github.repo_name.clone()
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




            match new_release_response.as_ref() {
                Ok(release_res) => {
                    println!("Successfully created GitHub release!");
                    println!("Uploading release asset to GitHub release...");

                    match reqwest::Client::new()
                        .post(
                            format!(
                                "https://uploads.github.com/repos/{}/{}/releases/{}/assets?name=\"{}\"",
                                config_file.github.repo_owner,
                                config_file.github.repo_name,
                                release_res.id,
                                file_name.replace(" ", "%20")
                            )
                        )
                        .header("User-Agent", env!("CARGO_PKG_NAME"))
                        .header("Accept", "application/vnd.github+json")
                        .header("Content-Type", "application/zip")
                        .bearer_auth(github_token)
                        .body(mrpack_file_contents.clone())
                        .send().await {
                            Ok(_) => println!("Successfully uploaded release asset!"),
                            Err(_) => println!("Failed to upload release asset.")
                    };

                },
                Err(err) => println!("Failed to create GitHub release: {}", err)
            }


            // Modrinth Release

            let modrinth_config = config_file.modrinth;

            println!("Uploading to Modrinth...");

            let modrinth_req = VersionRequest {
                name: version_name.clone(),
                version_number: pack_file.version,
                changelog: Some(changelog_markdown.to_owned()),
                dependencies: vec![],
                game_versions: vec![pack_file.versions.minecraft],
                version_type: VersionType::RELEASE,
                loaders: vec![loader.to_string().to_ascii_lowercase()],
                featured: false,
                requested_status: VersionStatus::LISTED,
                project_id: modrinth_config.project_id,
                file_parts: vec!["file".to_string()],
                primary_file: file_name.to_string(),
            };

            let modrinth_token = match env::var("MODRINTH_TOKEN") {
                Ok(token) => token,
                Err(err) => return Err(anyhow!(
                    "Failed to get `MODRINTH_TOKEN`: {}", err
                ))
            };

            let file_part = match reqwest::multipart::Part::bytes(mrpack_file_contents)
                .file_name(file_name)
                .mime_str("application/zip") {
                Ok(part) => part,
                Err(err) => return Err(anyhow!(
                    "Failed to get part from .mrpack file: {}", err
                ))
            };

            let form = reqwest::multipart::Form::new()
                .text("data", serde_json::to_string(&modrinth_req).unwrap())
                .part("file", file_part);

            let modrinth_req_url = match modrinth_config.staging {
                Some(is_staging) => match is_staging {
                    true => "https://staging-api.modrinth.com/v2/version",
                    false => "https://api.modrinth.com/v2/version"
                },
                None => "https://api.modrinth.com/v2/version"
            };

            let req = match reqwest::Client::new()
                .post(modrinth_req_url)
                .header("Authorization", modrinth_token)
                .multipart(form)
                .send().await {
                    Ok(res) => res,
                    Err(err) => return Err(anyhow!("Error uploading version: {}", err))
            };

            if req.status().is_success() {
                println!("Successfully uploaded version to Modrinth!");
            } else {
                return Err(anyhow!(
                    "Failed to upload version to Modrinth: {}",
                    req.text().await.unwrap()
                ))
            }

            if discord {

                let discord_config = match config_file.discord {
                    Some(config) => config,
                    None => return Err(anyhow!(
                        "Failed to get Discord config"
                    ))
                };

                let description = format!("\
                **New release!** {}\n\n\
                {} [GitHub](https://github.com/{}/{}/releases/latest)\n\
                {} [Modrinth](https://modrinth.com/modpacks/{})\n\n\
                {}
                ",
                    discord_config.discord_ping_role,
                    discord_config.github_emoji_id,
                    config_file.github.repo_owner,
                    config_file.github.repo_name,
                    discord_config.modrinth_emoji_id,
                    modrinth_req.project_id,
                    changelog_markdown
                );

                let embed = Embed::fake(|e| {
                    e.title(format!("{} {}", discord_config.title_emoji, version_name))
                        .description(description)
                        .image(discord_config.embed_image_url)
                        .footer(|f| {
                            f.text(format!("{} UTC", Utc::now().format("%b, %d %Y")))
                        })
                });

                let http = serenity::http::Http::new("token");
                let url = match env::var("WEBHOOK_URL") {
                    Ok(url) => url,
                    Err(err) => return Err(anyhow!(
                        "Failed to get webhook url: {}", err
                    ))
                };

                let webhook = Webhook::from_url(&http, &*url).await?;

                webhook.execute(&http, true, |w| {
                    w
                        .embeds(vec![embed])
                }).await?;

            }



            // Clean Up

            println!("Cleaning up...");

            match fs_extra::dir::remove(new_tmp_dir) {
                Ok(_) => {
                    println!("Removed temporary directory!")
                },
                Err(err) => return Err(anyhow!(
                    "Failed to remove temporary directory: {}", err
                ))
            }
        }
    }
    Ok(())
}
