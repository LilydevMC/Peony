use std::{env, fs};
use std::io::Read;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::anyhow;
use chrono::Utc;
use clap::{Parser, Subcommand, command};
use glob::glob;
use serenity::model::channel::Embed;
use serenity::model::webhook::Webhook;

use crate::{
    github::generate_changelog,
    models::{
        project_type::{
            modpack::config::ModpackConfig,
            mc_mod::config::ModConfig
        },
        modrinth::{
            project::ProjectResponse,
            ModrinthUrl
        }
    },
    pack::*,
    util::*,
    version::*
};
use crate::models::project_type::mc_mod::{Jar, ModInfo, ModJars};
use crate::models::project_type::mc_mod::version::{ModVersionInfo};

mod github;
mod mc_mod;
mod models;
mod modrinth;
mod pack;
mod util;
mod version;


#[derive(Debug, Parser)]
#[command(
    name = "peony",
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
    #[command(about = "Export and upload Packwiz modpack")]
    Modpack {
        #[clap(long, short, help = "Whether or not to send Discord webhook")]
        discord: bool,
        #[clap(long, short, help = "Custom version number")]
        version: Option<String>
    },
    #[command(about = "Build and upload Fabric/Quilt mc_mod")]
    Mod {
        #[clap(long, short, help = "Whether or not to send Discord webhook")]
        discord: bool,
        #[clap(long, short, help = "Args to pass to Gradle", default_value = "build")]
        gradle_args: String
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    match dotenvy::dotenv() {
        Ok(_) => (),
        Err(_) => ()
    };

    let args = CliArgs::parse();

    match args.commands {
        Commands::Modpack { discord, version } => {

            match which::which("packwiz") {
                Ok(_) => (),
                Err(err) => return Err(anyhow!("Failed to find packwiz executable: {}", err))
            }

            if !Path::new("mrpack.toml").exists() {
                return Err(anyhow!("Failed to find `mrpack.toml` file."))
            }

            let config_file = match fs::read_to_string("mrpack.toml") {
                Ok(content_string) => {
                    let parsed_config: ModpackConfig = match toml::from_str(&*content_string) {
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

            let version_info = match get_modpack_version_info(
                &config_file,
                &pack_file,
                &output_file_info
            ) {
                Ok(info) => info,
                Err(err) => return Err(err)
            };


            // Changelog

            let changelog_markdown = match generate_changelog(
                &config_file.github
            ).await {
                Ok(changelog) => changelog,
                Err(err) => return Err(err)
            };

            // GitHub Release

            match github::create_modpack_release(
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

            let modrinth_token = match env::var("MODRINTH_TOKEN") {
                Ok(token) => token,
                Err(err) => return Err(anyhow!(
                    "Failed to get `MODRINTH_TOKEN`: {}", err
                ))
            };

            let modrinth_url = ModrinthUrl::new(
                &config_file.modrinth.staging
                );

            match modrinth::create_modpack_release(
                &config_file,
                &pack_file,
                &output_file_info,
                &version_info,
                &changelog_markdown,
                modrinth_token.clone(),
                &modrinth_url
            ).await {
                Ok(_) => (),
                Err(err) => println!("{}", err)
            }

            // Send Discord webhook

            if discord {
                let discord_config = match config_file.discord {
                    Some(config) => config,
                    None => return Err(anyhow!(
                        "Failed to get Discord config"
                    ))
                };

                let modrinth_project = match reqwest::Client::new()
                    .get(format!(
                        "{}/project/{}",
                        modrinth_url.labrinth,
                        config_file.modrinth.project_id
                    ))
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
                **New release!**\n\n\
                {} [GitHub](https://github.com/{}/{}/releases/latest)\n\
                {} [Modrinth]({}/modpack/{})\n\n\
                {}
                ",
                    discord_config.github_emoji_id,
                    config_file.github.repo_owner,
                    config_file.github.repo_name,
                    discord_config.modrinth_emoji_id,
                    modrinth_url.knossos,
                    modrinth_project.slug,
                    changelog_markdown
                );

                let embed_color = match discord_config.embed_color {
                    Some(color) => color as i32,
                    None => match modrinth_project.color {
                        Some(color) => color,
                        None => 0x1e1f22
                    }
                };

                let release_time = Utc::now().format("%b, %d %Y %r");

                let embed = Embed::fake(|e| {
                    e.title(format!("{} {}", discord_config.title_emoji, version_info.version_name))
                        .color(embed_color)
                        .description(description)
                        .image(discord_config.embed_image_url)
                        .footer(|f| {
                            f.text(format!(
                                "{} | {} UTC",
                                modrinth_project.project_type.formatted(),
                                release_time
                            ))
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
                        .content(discord_config.discord_ping_role)
                        .embeds(vec![embed])
                }).await?;

            }


            clean_up(&tmp_info.dir_path)?
        },
        Commands::Mod { discord, gradle_args } => {
            match which::which("java") {
                Ok(_) => (),
                Err(err) => return Err(anyhow!("Failed to find Java executable: {}", err))
            }

            let mut gradlew_path: &Path;

            if env::consts::OS == "windows" {
                gradlew_path = Path::new(".\\gradlew.bat");
            } else {
                gradlew_path = Path::new("./gradlew");
            }

            if !Path::new(gradlew_path).exists() {
                return Err(anyhow!("Failed to find gradle script at `{:?}`", gradlew_path))
            }

            if !Path::new("peony_mod.toml").exists() {
                return Err(anyhow!("Failed to find `peony_mod.toml` file"))
            }


            let config_file = match fs::read_to_string("peony_mod.toml") {
                Ok(content_string) => {
                    let parsed_config: ModConfig = match toml::from_str(&*content_string) {
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


            let tmp_info = match create_temp() {
                Ok(info) => {
                    info
                },
                Err(err) => {
                    return Err(anyhow!("Failed to create temporary directory: {}", err))
                }
            };

            // remove previously-compiled jars, if any
            match fs::remove_dir(&tmp_info.dir_path.join("build").join("libs")) {
                Ok(_) => (),
                Err(_) => ()
            }

            let mut gradle_command = Command::new(gradlew_path);

            let gradle_command = gradle_command
                .arg(gradle_args)
                .current_dir(&tmp_info.dir_path);

            let mut gradle_child = match gradle_command.spawn() {
                Ok(child) => child,
                Err(err) => return Err(anyhow!(
                    "Failed to build with Gradle: {}", err
                ))
            };

            gradle_child.wait().unwrap();

            let jars = match glob(
                match Path::new(&tmp_info.dir_path)
                    .join("build")
                    .join("libs")
                    .join("*.jar")
                    .to_str() {
                        Some(path) => path,
                        None => return Err(anyhow!(
                            "Failed to parse glob to string"
                        ))
                    }
            ) {
                Ok(paths) => paths,
                Err(err) => return Err(anyhow!(
                    "Failed to find files with mod glob: {}", err
                ))
            };

            let mut possible_jars: Vec<PathBuf> = vec![];
            let mut possible_sources_jars: Vec<PathBuf> = vec![];

            for jar in jars {
                let jar_path = match jar {
                    Ok(path) => path,
                    Err(err) => return Err(anyhow!(
                        "Failed to parse jar glob result as path: {}", err
                    ))
                };

                let file_name = match jar_path.file_name() {
                    Some(name) => match name.to_os_string().into_string() {
                        Ok(name) => name,
                        // better error message needed, but fine for now
                        Err(_) => return Err(anyhow!(
                            "Failed to parse file name from OsString to String"
                        ))
                    },
                    None => return Err(anyhow!(
                        "Failed to parse file name from jar path"
                    ))
                };

                if file_name.ends_with("-sources.jar") {
                    possible_sources_jars.push(jar_path)
                } else if file_name.ends_with(".jar") {
                    possible_jars.push(jar_path)
                }
            }


            if possible_jars.len() != 1 {
                clean_up(&tmp_info.dir_path)?;
                return Err(anyhow!(
                    "Found an invalid amount of mod jars: {}",
                    possible_jars.len())
                )
            }


            let jar_path = &possible_jars[0];
            let sources_jar_path: Option<&PathBuf> = match possible_sources_jars.len() {
                0 => None,
                1 => Some(&possible_sources_jars[0]),
                _ => return Err(anyhow!(
                    "Found an invalid amount of sources jars: {}",
                    possible_sources_jars.len())
                )
            };

            let jar_file = fs::File::open(jar_path)?;

            let mut archive = zip::ZipArchive::new(jar_file)?;

            let mut loader_file =
                if file_exists_in_zip(&mut archive, "fabric.mod.json") {
                    match archive.by_name("fabric.mod.json") {
                        Ok(file) => file,
                        Err(err) => return Err(anyhow!(
                            "Failed to get `fabric.mod.json` file from jar: {}", err
                        ))
                    }
                } else if file_exists_in_zip(&mut archive, "quilt.mod.json") {
                    match archive.by_name("quilt.mod.json") {
                        Ok(file) => file,
                        Err(err) => return Err(anyhow!(
                            "Failed to get `quilt.mod.json` file from jar: {}", err
                        ))
                    }
                } else {
                    return Err(anyhow!(
                        "Failed to get `fabric.mod.json` or `quilt.mod.json` from jar"
                    ))
                };

            let mut loader_file_string = String::new();
            loader_file.read_to_string(&mut loader_file_string)?;

            let parsed_loader_file: serde_json::Value = serde_json::from_str(
                &*loader_file_string
            )?;

            let mod_info = ModInfo {
                name: trim_quotes(parsed_loader_file["name"].to_string()),
                id: trim_quotes(parsed_loader_file["id"].to_string()),
                version: trim_quotes(parsed_loader_file["version"].to_string())
            };

            let mod_jar_name = file_name_from_path(jar_path)?;

            let sources_jar_name = match sources_jar_path {
                Some(path) => Some(file_name_from_path(path)?),
                None => None
            };

            let mod_jar = Jar {
                file_name: mod_jar_name,
                file_path: jar_path.into()
            };

            let sources_jar = match sources_jar_name {
                Some(name) => Some(
                    Jar {
                        file_name: name,
                        file_path: sources_jar_path.unwrap().into()
                    }
                ),
                None => None
            };

            let mod_jars = ModJars {
                mod_jar,
                sources_jar
            };


            let version_info = ModVersionInfo::new(
                &config_file, &mod_jars, &mod_info
            )?;


            // Generate changelog from previous GitHub Releases
            let changelog_markdown = match generate_changelog(
                &config_file.github
            ).await {
                Ok(changelog) => changelog,
                Err(err) => return Err(err)
            };


            // Create GitHub Release

            let create_github_release = match github::create_mod_release(
                &config_file, &mod_info, &mod_jars,
                &changelog_markdown, &version_info.name
            ).await {
                Ok(_) => {},
                Err(err) => return Err(err),
            };


            // Create Modrinth Release

            let modrinth_url = ModrinthUrl::new(
                &config_file.modrinth.staging
            );

            let create_modrinth_version = match modrinth::create_mod_release(
                &config_file, &version_info,
                &changelog_markdown, &modrinth_url,
                &version_info.name
            ).await {
                Ok(_) => {},
                Err(err) => return Err(err)
            };

            clean_up(&tmp_info.dir_path)?

        }
    }
    Ok(())
}
