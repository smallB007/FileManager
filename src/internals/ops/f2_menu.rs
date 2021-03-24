use std::process::Command;

use crate::internals::file_explorer_utils::{get_active_panel, get_selected_paths_only};
use crate::internals::ops::ops_utils::zip::zip_file;
use cursive::align::{HAlign, VAlign};
use cursive::views::Dialog;
use cursive::views::*;
enum Compression {
    zip,
    tar,
    seven_z,
}
enum MenuItems {
    Pack,
    Unpack,
    //TestArchive,
    //ChangeAttribs,
    //Properties,
    //OccupiedSpace,
    //Split,
    //Combine,
    //Encode,
    //Decode,
    //CreateChecksum,
    //VerifyChecksum,
}
fn compress_zip(paths: &Vec<String>) {
    for path in paths {
        println!("A path: {}", path);
        match zip_file(&path, &(String::from(path) + "zippped")) {
            Ok(_val) => {}
            Err(err) => {
                println!("Couldn't zip:{}", err);
            }
        }
    }
}
pub fn menu(siv: &mut cursive::Cursive) {
    let mut menu_select = SelectView::new().h_align(HAlign::Center);
    menu_select.add_item("Pack", MenuItems::Pack);
    menu_select.add_item("Unpack", MenuItems::Unpack);

    menu_select.set_on_submit(|s, menu_item| {
        s.pop_layer();
        match menu_item {
            &MenuItems::Pack => {
                let active_panel = get_active_panel(s);
                match get_selected_paths_only(s, &active_panel) {
                    Some(paths) => {
                        let mut radio_group = RadioGroup::new();
                        let dlg = Dialog::around(
                            LinearLayout::vertical().child(TextView::new("Compress files")).child(
                                LinearLayout::horizontal()
                                    .child(radio_group.button(Compression::zip, ".zip"))
                                    .child(radio_group.button(Compression::tar, ".tar.xz"))
                                    .child(radio_group.button(Compression::seven_z, ".7z")),
                            ),
                        )
                        .button("Ok", move |s| {
                            s.pop_layer();
                            // We retrieve the stored value for group.
                            match *radio_group.selection() {
                                Compression::zip => {
                                    compress_zip(&paths);
                                }
                                Compression::tar => {
                                    todo!("todo")
                                }
                                Compression::seven_z => {
                                    todo!("todo")
                                }
                            }
                        });
                        s.add_layer(dlg);
                    }
                    None => {}
                }
            }
            &MenuItems::Unpack => {}
        }
    });
    let menu_dlg = Dialog::around(menu_select);
    siv.add_layer(menu_dlg);
}
