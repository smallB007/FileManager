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
        }).title("Delete");
    del_dlg
}
pub fn del(siv: &mut cursive::Cursive) {
    let active_panel = get_active_panel(siv);
    match get_selected_paths(siv, &active_panel) {
        Some(selected_paths) => {
            //let selected_path_to = get_current_dir(siv, to);
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
    // *First, check if copying is in the progress:*/
    //if let Some(ref cpy_data) = GLOBAL_FileManager.get().lock().unwrap().borrow().cpy_data {
    //    if let None = siv.find_name::<ProgressDlgT>(copy_progress_dlg::labels::dialog_name) {
    //        let cpy_progress_dlg =
    //            create_thmd_cpy_pgrss_dlg(siv, cpy_data.files_total, cpy_data.cond_var_suspend.clone());
    //        siv.add_layer(cpy_progress_dlg);
    //        siv.set_autorefresh(true);
    //    }
    //} else {
    //    /*No copying, let'siv start it then but first: ;)*/
    //    /*Check if we already presenting CpyDlg and if not ...*/
    //    if let None = siv.screen_mut().find_layer_from_name(copy_dlg::labels::dialog_name) {
    //        let active_panel = get_active_panel(siv);
    //
    //        let (from, to) = if active_panel == main_ui::widget_names::left_panel_id {
    //            (
    //                main_ui::widget_names::left_panel_id,
    //                main_ui::widget_names::right_panel_id,
    //            )
    //        } else {
    //            (
    //                main_ui::widget_names::right_panel_id,
    //                main_ui::widget_names::left_panel_id,
    //            )
    //        };
    //
    //        match get_selected_paths(siv, from) {
    //            Some(selected_paths_from) => {
    //                let selected_path_to = get_current_dir(siv, to);
    //                let cpy_dlg = create_cpy_dialog(selected_paths_from, selected_path_to);
    //                let cpy_dlg = create_themed_view(siv, cpy_dlg).with_name(copy_dlg::labels::dialog_name);
    //                siv.add_layer(cpy_dlg);
    //            }
    //            None => {
    //                let info_dlg = create_themed_view(
    //                    siv,
    //                    Atomic_Dialog::around(TextView::new("Please select item to copy")).dismiss_button("[ OK ]"),
    //                );
    //                siv.add_layer(info_dlg);
    //            }
    //        }
    //    }
}
