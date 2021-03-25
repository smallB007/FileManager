use std::{path::PathBuf, process::Command};

use crate::internals::file_explorer_utils::{get_active_panel, get_selected_paths_only};
use crate::internals::ops::ops_utils::lzma::create_xz_archive;
use crate::internals::ops::ops_utils::tar::create_tar_archive;
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
    //todo repeat
    for path in paths {
        match zip_file(&path, &(String::from(path) + "zippped")) {
            Ok(_val) => {}
            Err(err) => {
                println!("Couldn't zip:{}", err);
            }
        }
    }
}

fn compress_lzma(paths: &Vec<String>) {
    for path in paths {
        let mut output_file = PathBuf::from(path);
        output_file.set_extension("7z");
        if let Some(output) = output_file.as_os_str().to_str() {
            match create_xz_archive(&path, output) {
                Ok(_val) => {}
                Err(err) => {
                    println!("Couldn't zip:{}", err);
                }
            }
        }
    }
}

fn compress_tar(paths: &Vec<String>) {
    create_tar_archive(&paths);
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
                                Compression::tar => compress_tar(&paths),
                                Compression::seven_z => {
                                    compress_lzma(&paths);
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
