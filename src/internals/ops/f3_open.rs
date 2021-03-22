use std::process::Command;

use cursive::CursiveExt;
use cursive::{event::*, views::Dialog};

use crate::internals::file_explorer_utils::{get_active_panel, get_selected_paths};

pub fn open(siv: &mut cursive::Cursive) {
    if let Ok(editor) = std::env::var("EDITOR") {
        let active_panel = get_active_panel(siv);
        match get_selected_paths(siv, &active_panel) {
            Some(paths) => {
                for path in paths {
                    let output = Command::new(editor.clone())
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
    } else {
        let dlg = Dialog::around(cursive::views::TextView::new("Please set editor first and try again"))
            .title("No editor specified")
            .dismiss_button("OK");
        siv.add_layer(dlg);
    }
}
