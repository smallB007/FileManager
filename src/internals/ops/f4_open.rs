use std::process::Command;

use cursive::CursiveExt;
use cursive::{event::*, views::Dialog};

use crate::internals::file_explorer_utils::{get_active_panel, get_selected_paths};
pub fn open_externally(siv: &mut cursive::Cursive) {
    let active_panel = get_active_panel(siv);
    match get_selected_paths(siv, &active_panel) {
        Some(paths) => {
            for path in paths {
                open::that(path.1);
            }
        }
        None => {}
    }
}
