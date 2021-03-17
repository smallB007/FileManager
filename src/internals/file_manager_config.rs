use crate::internals::file_manager::GLOBAL_FileManager;
use crate::internals::literals;
use std::io::Write;
use std::{thread, time};

use serde::{Deserialize, Serialize};

use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMangerConfig {
    pub left_panel_initial_path: String,
    pub right_panel_initial_path: String,
}

impl Default for FileMangerConfig {
    fn default() -> Self {
        FileMangerConfig {
            left_panel_initial_path: std::env::var("HOME").unwrap(),
            right_panel_initial_path: std::env::var("HOME").unwrap(),
        }
    }
}

pub fn write_config(config: &FileMangerConfig) {
    let input_file = literals::config::config_keys::SETTINGS_PATH;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(input_file)
        .unwrap();

    match toml::to_string(config) {
        Ok(val) => match file.write_all(&val.as_bytes()) {
            Ok(res) => {
                println!("Wrote res")
            }
            Err(err) => {
                println!("Cannot write, err: {}", err)
            }
        },
        Err(err) => {}
    }
}
pub fn read_config() -> FileMangerConfig {
    let input_file = literals::config::config_keys::SETTINGS_PATH;
    let mut file = std::fs::File::open(input_file).unwrap();
    let mut file_content = String::new();
    let _bytes_read = file.read_to_string(&mut file_content).unwrap();
    match toml::from_str(&file_content) {
        Ok(config) => config,
        Err(err) => FileMangerConfig::default(),
    }
}
