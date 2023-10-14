use std::{env, fs};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use crate::{
    models::pack::PackFile
};

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

            let pack_file = match version {
                Some(_) => {
                    let file = match fs::read_to_string("pack.toml") {
                        Ok(file) => file,
                        Err(err) => return Err(anyhow!("Failed to read pack.toml file: {}", err))
                    };

                    let file_parsed: PackFile = match toml::from_str(file.as_str()) {
                        Ok(pack) => pack,
                        Err(err) => return Err(anyhow!("Failed to parse pack.toml file: {}", err))
                    };
                    Some(file_parsed)
                },
                None => None
            };

            let new_uuid = uuid::Uuid::new_v4();
            let new_tmp_dir_name = format!("mrpack-dist_{}", new_uuid);
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

            match pack_file {
                Some(file) => {
                    let mut new_file_contents = file;
                    new_file_contents.version = version;
                    let file_contents_string = match toml::to_string(
                        &new_file_contents
                    ) {
                        Ok(file) => file,
                        Err(err) => return Err(anyhow!(
                            "Failed to parse new pack data to toml: {}", err
                        ))
                    };

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


            if modrinth {
                println!("Modrinth config selected")
            }
            if github {
                println!("GitHub config selected")
            }
        }
    }
    Ok(())
}
