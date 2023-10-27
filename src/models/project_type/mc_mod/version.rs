use serde::{Serialize, Deserialize};
use crate::models::modrinth::Loader;
use crate::models::project_type::mc_mod::config::ModConfig;
use crate::models::project_type::mc_mod::{ModInfo, ModJars};
use crate::util::read_file;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModVersionInfo {
    pub name: String,
    pub version: String,
    pub loaders: Vec<Loader>,
    pub mod_file: ModFile,
    pub sources_file: Option<ModFile>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFile {
    pub name: String,
    pub contents: Vec<u8>
}

impl ModVersionInfo {
    pub fn loaders_formatted(&self) -> String {
        let loaders: &Vec<String> = &self.loaders.iter().map(|l| {
            l.formatted()
        }).collect();

        loaders.join("/")
    }
}


impl ModVersionInfo {
    pub fn new(
        config: &ModConfig, mod_jars: &ModJars, mod_info: &ModInfo
    ) -> Result<Self, anyhow::Error> {
        let mod_jar_contents = read_file(&mod_jars.mod_jar.file_path)?;
        let sources_jar_contents = match &mod_jars.sources_jar {
            Some(jar) => Some(read_file(&jar.file_path)?),
            None => None
        };

        let mod_jar_info = ModFile {
            name: mod_jars.mod_jar.file_name.clone(),
            contents: mod_jar_contents
        };

        let sources_jar_info = match sources_jar_contents {
            Some(contents) => Some(
                ModFile {
                    name: mod_jars.mod_jar.file_name.clone(),
                    contents
                }
            ),
            None => None
        };

        let loaders = config.loaders.clone();

        let loaders_formatted: String = loaders.iter().map(|l| {
            l.formatted()
        }).collect::<Vec<String>>().join("/");

        let version_name = config.version_name_format
            .replace("%project_name%", &mod_info.name)
            .replace("%project_version%", &mod_info.version)
            .replace("%mc_version%", &config.mc_version_alias)
            .replace("%loader%", &loaders_formatted);

        Ok(Self {
            name: version_name,
            version: mod_info.version.clone().into(),
            loaders,
            mod_file: mod_jar_info,
            sources_file: sources_jar_info,
        })
    }
}
