use std::process::Command;

use crate::internals::file_explorer_utils::{get_active_panel, get_selected_paths};
use crate::internals::ops::ops_utils::zip::zip_file;
use cursive::align::{HAlign, VAlign};
use cursive::views::Dialog;
use cursive::views::*;
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
pub fn menu(siv: &mut cursive::Cursive) {
    let mut time_select = SelectView::new().h_align(HAlign::Center);
    time_select.add_item("Pack", MenuItems::Pack);
    time_select.add_item("Unpack", MenuItems::Unpack);

    time_select.set_on_submit(|s, menu_item| {
        s.pop_layer();
        match menu_item {
            &MenuItems::Pack => {
                let active_panel = get_active_panel(s);
                match get_selected_paths(s, &active_panel) {
                    Some(paths) => {
                        for path in paths {
                            println!("A path: {}", path.1);
                            match zip_file(&path.1, &(String::from(&path.1) + "zippped")) {
                                Ok(_val) => {}
                                Err(err) => {
                                    println!("Couldn't zip:{}", err);
                                }
                            }
                            //xcompress::ArchiveFormat use zip
                            /* match Command::new(xcompress).arg("a").arg(path.1).status() {
                                Ok(res) => {
                                    println!("res: {}", res)
                                }
                                Err(err) => {
                                    println!("err: {}", err)
                                }
                            }*/
                            //pack.1
                        }
                    }
                    None => {}
                }
            }
            &MenuItems::Unpack => {}
        }
    });
    let menu_dlg = Dialog::around(time_select);
    siv.add_layer(menu_dlg);
}
