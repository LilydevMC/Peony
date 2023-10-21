use std::{env, fs};
use std::path::Path;
use std::process::Command;
use anyhow::anyhow;
use chrono::Utc;
use clap::{Parser, Subcommand, command};
use serenity::model::channel::Embed;
use serenity::model::webhook::Webhook;
use crate::github::{create_github_release, generate_changelog};
use crate::models::meta::Config;
use crate::models::modrinth::{ProjectResponse, VersionRequest, VersionStatus, VersionType};
use crate::pack::{get_output_file, get_pack_file, write_pack_file};
use crate::util::create_temp;
use crate::version::get_version_info;

mod github;
mod models;
mod modrinth;
mod pack;
mod util;
mod version;


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

            let mut pack_file = match get_pack_file() {
                Ok(file) => file,
                Err(err) => return Err(err)
            };

            let tmp_info = match create_temp() {
                Ok(info) => info,
                Err(err) => return Err(err)
            };

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

                    match write_pack_file(&tmp_info.dir_path, file_contents_string) {
                        Ok(_) => (),
                        Err(err) => return Err(err)
                    }
                },
                None => ()
            }

            match Command::new("packwiz")
                .arg("mr")
                .arg("export")
                .current_dir(&tmp_info.dir_path).output() {
                Ok(_) => (),
                Err(err) => return Err(anyhow!(
                    "Failed to export with packwiz: {}", err
                ))
            }

            let output_file_info = match get_output_file(&tmp_info) {
                Ok(file_info) => file_info,
                Err(err) => return Err(err)
            };

            let version_info = match get_version_info(
                &config_file,
                &pack_file,
                &output_file_info
            ) {
                Ok(info) => info,
                Err(err) => return Err(err)
            };


            // Changelog

            let changelog_markdown = match generate_changelog(
                &config_file
            ).await {
                Ok(changelog) => changelog,
                Err(err) => return Err(err)
            };

            // GitHub Release

            match create_github_release(
                &config_file,
                &pack_file,
                &output_file_info,
                &version_info,
                &changelog_markdown
            ).await {
                Ok(_) => (),
                Err(err) => println!("Failed to create GitHub release: {}", err)
            }


            // Modrinth Release

            let modrinth_config = config_file.modrinth;

            println!("Uploading to Modrinth...");

            let modrinth_req = VersionRequest {
                name: version_info.version_name.clone(),
                version_number: pack_file.version,
                changelog: Some(changelog_markdown.to_owned()),
                dependencies: vec![],
                game_versions: vec![pack_file.versions.minecraft],
                version_type: VersionType::RELEASE,
                loaders: vec![version_info.loader.clone().to_ascii_lowercase()],
                featured: false,
                requested_status: VersionStatus::LISTED,
                project_id: modrinth_config.project_id,
                file_parts: vec!["file".to_string()],
                primary_file: output_file_info.file_name.clone(),
            };

            let modrinth_token = match env::var("MODRINTH_TOKEN") {
                Ok(token) => token,
                Err(err) => return Err(anyhow!(
                    "Failed to get `MODRINTH_TOKEN`: {}", err
                ))
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

            let knossos_url = match modrinth_config.staging {
                Some(is_staging) => match is_staging {
                    true => "https://staging.modrinth.com",
                    false => "https://modrinth.com"
                },
                None => "https://modrinth.com"
            };

            let labrinth_url = match modrinth_config.staging {
                Some(is_staging) => match is_staging {
                    true => "https://staging-api.modrinth.com/v2",
                    false => "https://api.modrinth.com/v2"
                },
                None => "https://api.modrinth.com/v2"
            };

            let req = match reqwest::Client::new()
                .post(format!("{}/version", labrinth_url))
                .header("Authorization", &modrinth_token)
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

                let modrinth_project = match reqwest::Client::new()
                    .get(format!("{}/project/{}", labrinth_url, modrinth_req.project_id))
                    .header("Authorization", modrinth_token)
                    .send().await {
                        Ok(res) => {
                            match res.json::<ProjectResponse>().await {
                                Ok(json) => json,
                                Err(err) => return Err(anyhow!(
                                    "Error parsing response from get project: {}\n\
                                    Make sure this project is not a draft!",
                                    err.to_string()
                                ))
                            }
                        },
                        Err(err) => return Err(anyhow!(
                            "Error getting project from project id: {}",
                            err
                        ))
                };

                let description = format!("\
                **New release!** {}\n\n\
                {} [GitHub](https://github.com/{}/{}/releases/latest)\n\
                {} [Modrinth]({}/modpack/{})\n\n\
                {}
                ",
                    discord_config.discord_ping_role,
                    discord_config.github_emoji_id,
                    config_file.github.repo_owner,
                    config_file.github.repo_name,
                    discord_config.modrinth_emoji_id,
                    knossos_url,
                    modrinth_project.slug,
                    changelog_markdown
                );

                let embed = Embed::fake(|e| {
                    e.title(format!("{} {}", discord_config.title_emoji, version_info.version_name))
                        .description(description)
                        .image(discord_config.embed_image_url)
                        .footer(|f| {
                            f.text(format!("{} UTC", Utc::now().format("%b, %d %Y %r")))
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


            util::clean_up(&tmp_info.dir_path)?
        }
    }
    Ok(())
}
