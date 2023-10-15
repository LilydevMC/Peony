use std::{env, fs};
use std::path::Path;
use std::process::Command;
use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use glob::glob;
use crate::{
    models::pack::PackFile
};
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
        #[clap(long, short, help = "Custom version number")]
        version: Option<String>,
        #[clap(long, short, help = "Runs Modrinth configuration")]
        modrinth: bool,
        #[clap(long, short, help = "Runs GitHub configuration")]
        github: bool,
    }
}

fn main() -> anyhow::Result<()> {
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
        Commands::Run { version, modrinth, github } => {
            if !modrinth && !github {
                return Err(anyhow!("No run configurations selected"));
            }

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
                mrpack_path_res = Some(entry)
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
                    Some(name) => name,
                    None => return Err(anyhow!("Failed to parse file name from OsString to &str"))
                },
                None => return Err(anyhow!("Failed to get file name"))
            };


            if modrinth {
                let modrinth_config = match config_file.modrinth {
                    Some(mr) => mr,
                    None => return Err(anyhow!("No Modrinth configuration supplied!"))
                };

                println!("Uploading to Modrinth...");

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

                let modrinth_req = VersionRequest {
                    name: version_name,
                    version_number: pack_file.version,
                    changelog: None,
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

                let form = match reqwest::blocking::multipart::Form::new()
                    .text("data", serde_json::to_string(&modrinth_req).unwrap())
                    .file("file", mrpack_path) {
                    Ok(res) => res,
                    Err(err) => return Err(anyhow!("Error building form: {}", err))
                };

                let req = match reqwest::blocking::Client::new()
                    .post("https://api.modrinth.com/v2/version")
                    .header("Authorization", modrinth_token)
                    .multipart(form)
                    .send() {
                    Ok(res) => res,
                    Err(err) => return Err(anyhow!("Error uploading version: {}", err))
                };

                if req.status().is_success() {
                    println!("Successfully uploaded version to Modrinth!");
                } else {
                    return Err(anyhow!(
                        "Failed to upload version to Modrinth: {}",
                        req.text().unwrap()
                    ))
                }
            }
            if github {
                println!("GitHub config selected")
            }
        }
    }
    Ok(())
}
