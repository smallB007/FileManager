use crate::internals::file_explorer_utils::{create_main_layout, create_main_menu};
use crate::internals::file_manager_config::read_config;
use crate::internals::ops::f5_cpy::{AtomicFileTransitFlags, CpyData};
use std::sync::mpsc::{Receiver, Sender};
pub struct FileManager {
    pub id: i64,
    pub active_table: String, //change to &str
    pub tx_rx: (Sender<AtomicFileTransitFlags>, Receiver<AtomicFileTransitFlags>),
    pub cpy_data: Option<CpyData>,
}
impl Default for FileManager {
    fn default() -> Self {
        FileManager {
            id: 0,
            active_table: String::from(""),
            tx_rx: std::sync::mpsc::channel(),
            cpy_data: None,
        }
    }
}

pub static GLOBAL_FileManager: state::Storage<std::sync::Mutex<std::cell::RefCell<FileManager>>> =
    state::Storage::new();
//static GLOBAL_FileManager: state::LocalStorage<std::cell::RefCell<FileManager>> = state::LocalStorage::new();
impl FileManager {
    pub fn new(mut siv: &mut cursive::CursiveRunnable) {
        GLOBAL_FileManager.set(std::sync::Mutex::new(std::cell::RefCell::new(FileManager::default())));
        let fm_config = read_config();
        create_main_menu(&mut siv, true, true);

        create_main_layout(&mut siv, &fm_config);
        /* let v = GLOBAL_FileManager.get();
        let tmp = v.lock().unwrap();
        let fm = tmp.borrow_mut();
        //let fm = FileManager{id:1};
        fm.init(&mut siv);*/
    }
}
