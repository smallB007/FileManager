use crate::internals::file_manager::GLOBAL_FileManager;
use crate::internals::literals;
use std::io::Write;
use std::{thread, time};

use serde::{Deserialize, Serialize};

use std::io::Read;
/*
use lazy_static::*;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use config::Config;
use std::sync::mpsc::channel;
use std::sync::RwLock;
use std::time::Duration;
use std::{collections::HashMap, path::PathBuf};
lazy_static! {
    pub static ref SETTINGS: RwLock<Config> = RwLock::new({
        let mut settings = Config::default();
        match settings.merge(config::File::with_name(literals::config::config_keys::settings_path)) {
            Err(err) => {
                println!("settings error: {}", err);
            }
            _ => {}
        }

        settings
    });
}
fn show() {
    println!(
        " * Settings :: \n\x1b[31m{:?}\x1b[0m",
        SETTINGS
            .read()
            .unwrap()
            .clone()
            .try_into::<HashMap<String, String>>()
            .unwrap()
    );
}

fn exwatch() {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(literals::config::config_keys::settings_path)
    {
        Ok(file) => {
            watcher
                .watch(
                    literals::config::config_keys::settings_path,
                    RecursiveMode::NonRecursive,
                )
                .unwrap();

            // This is a simple loop, but you may want to use more complex logic here,
            // for example to handle I/O.
            loop {
                match rx.recv() {
                    Ok(DebouncedEvent::Write(_)) => {
                        println!(" * Settings.toml written; refreshing configuration ...");
                        SETTINGS.write().unwrap().refresh().unwrap();
                        show();
                    }

                    Err(e) => println!("watch error: {:?}", e),

                    _ => {
                        // Ignore event
                    }
                }
            }
        }
        Err(err) => {
            println!("Couldn't create/open file: {:?}", err)
        }
    }
}
*/
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
    let input_file = literals::config::config_keys::settings_path;
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
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
    /*let ten_millis = time::Duration::from_secs(10);

    thread::sleep(ten_millis);*/
    let input_file = literals::config::config_keys::settings_path;
    let mut file = std::fs::File::open(input_file).unwrap();
    let mut file_content = String::new();
    let _bytes_read = file.read_to_string(&mut file_content).unwrap();
    match toml::from_str(&file_content) {
        Ok(config) => config,
        Err(err) => FileMangerConfig::default(),
    }

    /* std::thread::spawn(|| exwatch());
    let mut left_panel_initial_path = std::env::var("HOME").unwrap();
    let mut right_panel_initial_path = std::env::var("HOME").unwrap();
    match SETTINGS.read() {
        Ok(res) => match res.clone().try_into::<HashMap<String, String>>() {
            Ok(ref mut hm) => {
                match hm.get(literals::config::config_keys::left_panel_initial_path) {
                    Some(path) => match PathBuf::from(path).metadata() {
                        Ok(val) => {
                            if val.is_dir() {
                                left_panel_initial_path = path.to_owned();
                            } else {
                                hm.remove(literals::config::config_keys::left_panel_initial_path);
                            }
                        }
                        Err(err) => {}
                    },
                    None => {
                        //log only
                    }
                }
                match hm.get(literals::config::config_keys::right_panel_initial_path) {
                    Some(path) => match PathBuf::from(path).metadata() {
                        Ok(val) => {
                            if val.is_dir() {
                                right_panel_initial_path = path.to_owned();
                            } else {
                                hm.remove(literals::config::config_keys::right_panel_initial_path);
                            }
                        }
                        Err(err) => {}
                    },
                    None => {
                        //log only
                    }
                }
            }
            Err(err) => {}
        },
        Err(err) => {}
    }*/
    /* FileMangerConfig {
        left_panel_initial_path,
        right_panel_initial_path,
    }*/
}
