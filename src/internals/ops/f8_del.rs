use std::sync::Arc;

use fs_extra::remove_items;

use crate::internals::atomic_dialog::Atomic_Dialog;
use crate::internals::file_explorer_utils::{
    create_themed_view, get_active_panel, get_error_theme, get_selected_paths, unselect_inx, PathInfoT,
};

use cursive::views::{Dialog, Layer, TextView, ThemedView};
fn create_del_dlg(items: PathInfoT) -> Dialog {
    let del_dlg = Dialog::around(TextView::new(format!("Confirm removal of {} item/s", items.len())))
        .button("Ok", move |s| {
            for (inx, item) in items.iter().enumerate() {
                match remove_items(&vec![&item.1]) {
                    Ok(_) => {
                        unselect_inx(s, Arc::new((*item.0.to_owned()).to_string()), Arc::new(inx));
                    }
                    Err(e) => {
                        println!("Cannot remove: {}", e)
                    }
                }
            }
            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        })
        .title("Delete");
    del_dlg
}
pub fn del(siv: &mut cursive::Cursive) {
    let active_panel = get_active_panel(siv);
    match get_selected_paths(siv, &active_panel) {
        Some(selected_paths) => {
            let del_dlg = create_del_dlg(selected_paths);
            let theme_error = get_error_theme(siv);
            siv.add_layer(ThemedView::new(theme_error, Layer::new(del_dlg)));
        }
        None => {
            let info_dlg = create_themed_view(
                //todo refactor
                siv,
                Atomic_Dialog::around(TextView::new("Please select item to delete")).dismiss_button("[ OK ]"),
            );
            siv.add_layer(info_dlg);
        }
    }
}
