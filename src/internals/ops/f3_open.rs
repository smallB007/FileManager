use std::process::Command;

use cursive::event::*;
use cursive::CursiveExt;

use crate::internals::file_explorer_utils::{get_active_panel, get_selected_paths};

pub fn open(siv: &mut cursive::Cursive) {
    let active_panel = get_active_panel(siv);
    match get_selected_paths(siv, &active_panel) {
        Some(paths) => {
            for path in paths {
                let output = Command::new("nano")
                    .arg(path.1)
                    .status()
                    .expect("failed to execute process");
            }
        }
        None => {}
    }
    /*Funny enough, we need to add callback again...*/
    siv.add_global_callback(Key::F3, open);
    siv.run();
}
