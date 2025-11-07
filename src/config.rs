use std::{
    env,
    fs::{OpenOptions, create_dir_all},
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::utils::{Directories, get_project_dir};

const CONFIG_FILE_NAME: &str = "Config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub path_to_config_file: PathBuf,
    pub log_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path_to_config_file: get_project_dir(Directories::Config)
                .join(CONFIG_FILE_NAME),
            log_dir: get_project_dir(Directories::Log),
            cache_dir: get_project_dir(Directories::Cache),
            data_dir: get_project_dir(Directories::Data),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path_to_config =
            env::var("CONFIG_PATH").map(|path| path.into()).unwrap_or(
                get_project_dir(Directories::Config).join(CONFIG_FILE_NAME),
            );

        let mut buf: String = String::new();
        if let Ok(mut file) =
            OpenOptions::new().read(true).open(&path_to_config)
            && file.read_to_string(&mut buf).is_ok_and(|bytes| bytes > 0)
            && let Ok(loaded_config) = toml::from_str::<Config>(&buf)
        {
            return loaded_config;
        };

        let new_config = Self::default();
        new_config.save();
        new_config
    }

    pub fn save(&self) {
        create_dir_all(self.path_to_config_file.parent().unwrap()).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.path_to_config_file)
            .unwrap();
        file.write_all(toml::to_string_pretty(self).unwrap().as_bytes())
            .unwrap();
    }
}
