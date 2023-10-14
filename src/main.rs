use std::fs;
use std::process::Command;
use anyhow::anyhow;
use clap::{Parser, Subcommand, command, arg};
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
        #[clap(long, short, help = "Runs Modrinth configuration.")]
        modrinth: bool,
        #[clap(long, short, help = "Runs GitHub configuration.")]
        github: bool
    }
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();

    match which::which("packwiz") {
        Ok(_) => (),
        Err(err) => return Err(anyhow!("Failed to find packwiz executable: {}", err))
    }

    match args.commands {
        Commands::Run { modrinth, github } => {
            if !modrinth && !github {
                return Err(anyhow!("No run configurations selected"));
            }

            let pack_file = match fs::read_to_string("pack.toml") {
                Ok(file) => file,
                Err(err) => return Err(anyhow!("Failed to read pack.toml file: {}", err))
            };

            let pack_file_parsed: PackFile = match toml::from_str(pack_file.as_str()) {
                Ok(pack) => pack,
                Err(err) => return Err(anyhow!("Failed to parse pack.toml file: {}", err))
            };

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
