use crate::internals::file_explorer_utils::{create_main_layout, create_main_menu};
use crate::internals::file_manager_config::read_config;
use crate::internals::ops::f5_cpy::{AtomicFileTransitFlags, CpyData};
use std::sync::mpsc::{Receiver, Sender};

use super::file_manager_config;
pub struct FileManager {
    pub config: file_manager_config::FileMangerConfig,
    pub id: i64,
    pub active_table: String, //change to &str
    pub tx_rx: (Sender<AtomicFileTransitFlags>, Receiver<AtomicFileTransitFlags>),
    cpy_data: Option<CpyData>,
}
impl Default for FileManager {
    fn default() -> Self {
        FileManager {
            config: file_manager_config::FileMangerConfig::default(),
            id: 0,
            active_table: String::from(""),
            tx_rx: std::sync::mpsc::channel(),
            cpy_data: None,
        }
    }
}
impl FileManager {
    pub fn clear(&mut self) {
        self.cpy_data = None;
    }
    pub fn get_cpy_data_mut(&mut self) -> &mut Option<CpyData> {
        &mut self.cpy_data
    }
    pub fn get_cpy_data(&self) -> &Option<CpyData> {
        &self.cpy_data
    }
    pub fn set_cpy_data(&mut self, data: Option<CpyData>) {
        self.cpy_data = data;
    }
}

pub static GLOBAL_FileManager: state::Storage<std::sync::Mutex<std::cell::RefCell<FileManager>>> =
    state::Storage::new();

impl FileManager {
    pub fn new(mut siv: &mut cursive::CursiveRunnable) {
        GLOBAL_FileManager.set(std::sync::Mutex::new(std::cell::RefCell::new(FileManager::default())));

        let fm_config = read_config();
        let v = GLOBAL_FileManager.get();
        let tmp = v.lock().unwrap();
        let mut fm = tmp.borrow_mut();
        fm.config = fm_config;
        create_main_menu(&mut siv, true, true);

        create_main_layout(&mut siv, &fm.config);
    }
}
