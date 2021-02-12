#![allow(warnings, unused)]
use cursive::align::{HAlign, VAlign};
use cursive::event::*;
use cursive::menu::Tree;
use cursive::traits::*;
use cursive::traits::*;
use cursive::utils::Counter;
use cursive::view::Boxable;
use cursive::views::*;
use cursive::*;
use cursive::{Cursive, CursiveExt};
// STD Dependencies -----------------------------------------------------------
use super::cursive_table_view::{TableView, TableViewItem};
use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::{
    io::{Error, ErrorKind},
    os::unix::prelude::MetadataExt,
};
/*FileManager crate*/
use super::delimiter::Delimiter;
use crate::internals::atomic_button::Atomic_Button;
use crate::internals::atomic_dialog::Atomic_Dialog;
use crate::internals::atomic_dialog_try::AtomicDialog;
use crate::internals::atomic_text_view::AtomicTextView;
// ----------------------------------------------------------------------------
//use std::cmp::Ordering;
// External Dependencies ------------------------------------------------------
// ----------------------------------------------------------------------------

use config::Config;
use fs_extra::dir::{copy, TransitProcessResult};
use notify::{watcher, INotifyWatcher, RecursiveMode, Watcher};
// This examples shows how to configure and use a menubar at the top of the
// application.

struct FileMangerConfig {
    left_panel_initial_path: String,
    right_panel_initial_path: String,
}
fn read_config() -> FileMangerConfig {
    FileMangerConfig {
        left_panel_initial_path: "/home/artie/Desktop/Left".to_owned(),
        right_panel_initial_path: "/home/artie/Desktop/Right".to_owned(),
    }
}
pub struct FileManager {
    id: i64,
    active_table: String, //change to &str
    tx_rx: (Sender<fs_extra::dir::TransitProcessResult>, Receiver<fs_extra::dir::TransitProcessResult>),
    cancel_current_operation: bool,
}
impl Default for FileManager {
    fn default() -> Self {
        FileManager {
            id: 0,
            active_table: String::from(""),
            tx_rx: std::sync::mpsc::channel(),
            cancel_current_operation: false,
        }
    }
}

static GLOBAL_FileManager: state::Storage<std::sync::Mutex<std::cell::RefCell<FileManager>>> = state::Storage::new();
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
pub fn create_main_menu(siv: &mut cursive::CursiveRunnable, showMenu: bool, alwaysVisible: bool) {
    //    let mut siv = cursive::default();

    // We'll use a counter to name new files.
    let counter = AtomicUsize::new(1);

    // The menubar is a list of (label, menu tree) pairs.
    siv.menubar()
        // We add a new "File" tree
        .add_subtree(
            "File",
            Tree::new()
                // Trees are made of leaves, with are directly actionable...
                .leaf("New", move |s| {
                    // Here we use the counter to add an entry
                    // in the list of "Recent" items.
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    let filename = format!("New {}", i);
                    s.menubar()
                        .find_subtree("File")
                        .unwrap()
                        .find_subtree("Recent")
                        .unwrap()
                        .insert_leaf(0, filename, |_| ());

                    s.add_layer(Dialog::info("New file!"));
                })
                // ... and of sub-trees, which open up when selected.
                .subtree(
                    "Recent",
                    // The `.with()` method can help when running loops
                    // within builder patterns.
                    Tree::new().with(|tree| {
                        for i in 1..100 {
                            // We don't actually do anything here,
                            // but you could!
                            tree.add_leaf(format!("Item {}", i), |_| ())
                        }
                    }),
                )
                // Delimiter are simple lines between items,
                // and cannot be selected.
                .delimiter()
                .with(|tree| {
                    for i in 1..10 {
                        tree.add_leaf(format!("Option {}", i), |_| ());
                    }
                }),
        )
        .add_subtree(
            "Help",
            Tree::new()
                .subtree(
                    "Help",
                    Tree::new().leaf("General", |s| s.add_layer(Dialog::info("Help message!"))).leaf("Online", |s| {
                        let text = "Google it yourself!\n\
                                        Kids, these days...";
                        s.add_layer(Dialog::info(text))
                    }),
                )
                .leaf("About", |s| s.add_layer(Dialog::info("Cursive v0.0.0"))),
        )
        .add_delimiter()
        .add_leaf("Quit", |s| s.quit());

    // When `autohide` is on (default), the menu only appears when active.
    // Turning it off will leave the menu always visible.
    // Try uncommenting this line!

    if alwaysVisible {
        siv.set_autohide_menu(false)
    };
    if showMenu {
        siv.select_menubar()
    }
    siv.add_global_callback(Key::Esc, switch_panel);
    siv.add_global_callback(Key::F10, quit);
    siv.add_global_callback(Key::F4, cpy);
    //  siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));

    //siv.run();
}
fn switch_panel(s: &mut cursive::Cursive) {
    if let Some(mut dialog) = s.find_name::<Dialog>("DLG") {
        for but in dialog.buttons_mut() {}
    }
    //siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));
}
// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExplorerColumn {
    Name,
    Size,
    LastModifyTime,
}

impl ExplorerColumn {
    fn as_str(&self) -> &str {
        match *self {
            ExplorerColumn::Name => "Name",
            ExplorerColumn::Size => "Size",
            ExplorerColumn::LastModifyTime => "Last Modify Time",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExplorerColumnData {
    name: String,
    size: u64,
    last_modify_time: SystemTime,
}

impl TableViewItem<ExplorerColumn> for ExplorerColumnData {
    fn to_column(&self, column: ExplorerColumn) -> String {
        match column {
            ExplorerColumn::Name => self.name.clone(),
            ExplorerColumn::Size => format!("{}", self.size),
            ExplorerColumn::LastModifyTime => {
                let datetime: DateTime<Utc> = self.last_modify_time.into();
                format!("{}", datetime.format("%d/%m/%Y %T"))
            }
        }
    }

    fn cmp(&self, other: &Self, column: ExplorerColumn) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            ExplorerColumn::Name => {
                if self.name == format!("..") {
                    return std::cmp::Ordering::Equal;
                } else {
                    return self.name.cmp(&other.name);
                }
            }
            ExplorerColumn::Size => self.size.cmp(&other.size),
            ExplorerColumn::LastModifyTime => self.last_modify_time.cmp(&other.last_modify_time),
        }
    }
}
type tableViewType = TableView<ExplorerColumnData, ExplorerColumn>;
pub fn create_basic_table_core(siv: &mut Cursive, a_name: &'static str, initial_path: &str) -> NamedView<tableViewType> {
    let mut table = tableViewType::new()
        .column(ExplorerColumn::Name, "Name", |c| c.width_percent(60))
        .column(ExplorerColumn::Size, "Size", |c| c.align(cursive::align::HAlign::Center))
        .column(ExplorerColumn::LastModifyTime, "LastModifyTime", |c| {
            c.ordering(std::cmp::Ordering::Greater).align(HAlign::Right).width_percent(20)
        });
    /*
    let v = GLOBAL_FileManager.get();
    let tmp = v.lock().unwrap();
    let mut fm_manager = tmp.borrow_mut();
      fm_manager.start_dir_watcher_thread(&a_name, &initial_path,&mut table);*/
    /*=============BEGIN DIR WATCHER=================*/
    let (tx, rx) = channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(initial_path.clone(), RecursiveMode::NonRecursive).unwrap();

    start_dir_watcher_thread(siv, String::from(a_name), String::from(initial_path), rx);
    let watcher = Arc::new(Mutex::new(watcher));
    /*=============END DIR WATCHER=================*/
    fill_table_with_items(&mut table, PathBuf::from(initial_path));
    table.set_on_sort(|siv: &mut Cursive, column: ExplorerColumn, order: std::cmp::Ordering| {
        siv.add_layer(
            Dialog::around(TextView::new(format!("{} / {:?}", column.as_str(), order)))
                .title("Sorted by")
                .button("Close", |s| {
                    s.pop_layer();
                }),
        );
    });
    table.set_on_select(move |siv: &mut Cursive, row: usize, index: usize| {
        let current_item = siv
            .call_on_name(a_name, move |a_table: &mut tableViewType| {
                /*                let v = GLOBAL_FileManager.get();
                let tmp = v.lock().unwrap();
                let mut v = tmp.borrow_mut();*/
                a_table.borrow_item(index).unwrap().clone()
            })
            .unwrap();
        let _value = siv
            .call_on_name(&(String::from(a_name) + &String::from("InfoItem")), move |a_dlg: &mut TextView| {
                //                a_dlg.set_title(current_item.name.clone());
                a_dlg.set_content(current_item.name.clone());
            })
            .unwrap();

        /*        siv.add_layer(
            Dialog::around(TextView::new(value))
                .title(format!("Removing row # {}", row))
                .button("Close", move |s| {
                    s.call_on_name(a_name, |a_table: &mut tableViewType| {
                        a_table.remove_item(index);
                    });
                    s.pop_layer();
                }),
        );*/
    });
    table.set_selected_row(0);
    table.set_on_submit(move |siv: &mut Cursive, row: usize, index: usize| {
        let current_dir = siv
            .call_on_name(&(String::from(a_name) + &String::from("Dlg")), move |a_dlg: &mut Atomic_Dialog| {
                //format!("{:?}", a_table.borrow_item(index).unwrap())
                a_dlg.get_title()
            })
            .unwrap();
        let path_to_stop_watching = current_dir.clone();
        let new_path = siv
            .call_on_name(a_name, move |a_table: &mut tableViewType| {
                let selected_item = a_table.borrow_item(index).unwrap().name.clone();
                let whole_path = match selected_item.as_str() {
                    ".." => match PathBuf::from(current_dir).parent() {
                        Some(parent) => PathBuf::from(parent),
                        None => PathBuf::from("NO_PARENT"),
                    },
                    _ => {
                        if PathBuf::from(current_dir.clone() + &selected_item.clone()).is_dir() {
                            let mut removed_first_slash: String = selected_item.clone();
                            removed_first_slash.remove(0);
                            let mut full_path = PathBuf::from(current_dir);
                            full_path.push(&removed_first_slash);
                            full_path
                        } else {
                            PathBuf::from("FILE_SELECTED")
                        }
                    }
                };
                whole_path
            })
            .unwrap();
        if new_path.is_dir() {
            watcher.lock().unwrap().unwatch(path_to_stop_watching);
            watcher.lock().unwrap().watch(new_path.clone(), RecursiveMode::NonRecursive).unwrap();
            /*            let new_path_clone = new_path.clone();
            let a_table_name_clone = a_name.clone();*/

            /*            if let Some(wtchr) = a_file_mngr.watchers.remove_entry(a_table_name_clone) {}
            a_file_mngr.watchers.insert(
                String::from(a_table_name_clone),

                std::thread::spawn(move || watch_dir(new_path_clone, a_table_name_clone)),
            );*/
            fill_table_with_items_wrapper(siv, String::from(a_name), new_path);
            /*
            let mut res = Option::<std::io::Error>::default();
            siv.call_on_name(a_name, |a_table: &mut tableViewType| {
                res = fill_table_with_items(a_table, new_path.clone()).err();
            });
            match res {
                Some(e) => {
                    siv.add_layer(Dialog::around(TextView::new(e.to_string())).dismiss_button("Ok"));
                }
                None => {
                    let _value = siv
                        .call_on_name(&(String::from(a_name) + &String::from("Dlg")), |a_dlg: &mut Atomic_Dialog| {
                            a_dlg.set_title(new_path.clone().to_str().unwrap());
                        })
                        .unwrap();
                }
            }*/
        }
        /*        siv.add_layer(
            Dialog::around(TextView::new(value))
                .title(format!("Removing row # {}", row))
                .button("Close", move |s| {
                    s.call_on_name(a_name, |a_table: &mut tableViewType| {
                        a_table.remove_item(index);
                    });
                    s.pop_layer();
                }),
        );*/
    });
    let named_view_table = table.with_name(a_name);

    named_view_table
}
fn get_selected_path(siv: &mut Cursive, a_name: &str) -> Option<String> {
    let mut item_from_inx = usize::MAX;
    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
        if let Some(inx) = a_table.item() {
            item_from_inx = inx;
        };
    });
    let selected_path = get_selected_path_from_inx(siv, a_name, item_from_inx);
    selected_path
}

fn get_current_dir(siv: &mut Cursive, a_name: &str) -> String {
    let current_dir = siv
        .call_on_name(&(String::from(a_name) + &String::from("Dlg")), move |a_dlg: &mut Atomic_Dialog| {
            a_dlg.get_title()
        })
        .unwrap();
    current_dir
}
fn get_selected_path_from_inx(siv: &mut Cursive, a_name: &str, index: usize) -> Option<String> {
    /*Todo repeat*/
    let current_dir = get_current_dir(siv, a_name);
    let new_path = siv
        .call_on_name(a_name, move |a_table: &mut tableViewType| {
            let mut selected_item = a_table.borrow_item(index).unwrap().name.clone();
            let whole_path = match selected_item.as_str() {
                ".." => None,
                _ => {
                    if selected_item.chars().nth(0).unwrap() != std::path::MAIN_SEPARATOR {
                        selected_item.insert(0, std::path::MAIN_SEPARATOR);
                    }
                    Some(current_dir + &selected_item)
                }
            };
            whole_path
        })
        .unwrap();
    new_path
}
// Function to simulate a long process.
fn copying_error(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer(); //trouble
    s.add_layer(
        Dialog::new()
            .title("Copying error")
            .content(TextView::new("Copying ERROR").center())
            .dismiss_button("OK"),
    );
}
fn copying_already_exists(s: &mut Cursive, path_from: Rc<PathBuf>, path_to: Rc<PathBuf>, is_overwrite: bool, is_recursive: bool) {
    let theme = s.current_theme().clone().with(|theme| {
        theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Red);
        theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
        theme.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Yellow);
        theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Black);
    });
    s.set_autorefresh(false); //todo repeat
    if let Some(_) = s.find_name::<Dialog>("ProgressDlg") {
        s.pop_layer();
    }
    let name_from = path_from.to_str().unwrap();
    let size_from = path_from.metadata().unwrap().size();
    let date_from = {
        let datetime: DateTime<Utc> = path_from.metadata().unwrap().modified().unwrap().into();
        format!("{}", datetime.format("%d/%m/%Y %T"))
    };
    let right_file_name = path_from.file_name().unwrap();
    let path_to_joined = path_to.join(right_file_name);
    let name_to = path_to_joined.to_str().unwrap();
    let size_to = path_to_joined.metadata().unwrap().size();
    let date_to = {
        let datetime: DateTime<Utc> = path_to_joined.metadata().unwrap().modified().unwrap().into();
        format!("{}", datetime.format("%d/%m/%Y %T"))
    };

    let new_from_layout = LinearLayout::horizontal().child(TextView::new(format!(
        "New     : {name}\nSize: {size}\t Date: {date}",
        name = name_from,
        size = size_from,
        date = date_from
    )));
    let new_to_layout = LinearLayout::horizontal().child(TextView::new(format!(
        "Existing: {name}\nSize: {size}\t Date: {date}",
        name = name_to,
        size = size_to,
        date = date_to
    )));
    let file_exist_dlg = Dialog::around(
        LinearLayout::vertical()
            .child(new_from_layout)
            .child(DummyView)
            .child(new_to_layout)
            .child(DummyView)
            .child(Delimiter::default())
            .child(DummyView)
            .child(
                LinearLayout::vertical()
                    .child(LinearLayout::horizontal().child(Checkbox::new()).child(TextView::new(" Apply to all")))
                    .child(LinearLayout::horizontal().child(Checkbox::new()).child(TextView::new(" Don't overwrite with zero length file"))),
            )
            .child(DummyView)
    )
    .title("File Exists")
    .button("Overwrite", move |s| {
        s.pop_layer();
        ok_cpy_callback(s, path_from.clone(), path_to.clone(), is_recursive, true)
    })
    .button("Older", |s| {})
    .button("Smaller", |s| {})
    .button("Different size", |s| {})
    .button("Append", |s| {})
    .button("Skip", |s| {})
    .button("Abort", |s| {});

    s.add_layer(views::ThemedView::new(theme, Layer::new(file_exist_dlg)));
}
fn copying_finished_success(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer(); //trouble
    s.add_layer(
        Dialog::new()
            .title("Copying finished")
            .content(TextView::new("Copying finished successfully").center())
            .dismiss_button("OK"),
    );
}
fn fill_table_with_items_wrapper(siv: &mut Cursive, a_name: String, new_path: PathBuf) {
    let mut res = Option::<std::io::Error>::default();
    siv.call_on_name(&a_name, |a_table: &mut tableViewType| {
        res = fill_table_with_items(a_table, new_path.clone()).err();
    });
    match res {
        Some(e) => {
            siv.add_layer(Dialog::around(TextView::new(e.to_string())).dismiss_button("Ok"));
        }
        None => {
            let _value = siv
                .call_on_name(&(String::from(a_name) + &String::from("Dlg")), |a_dlg: &mut Atomic_Dialog| {
                    a_dlg.set_title(new_path.clone().to_str().unwrap());
                })
                .unwrap();
        }
    }
}

fn update_table(siv: &mut Cursive, a_name: String, a_path: String) {
    let new_path = PathBuf::from(a_path);
    fill_table_with_items_wrapper(siv, a_name, new_path);
    //println!("Command received");
}
fn copying_cancelled(s: &mut Cursive) {
    s.set_autorefresh(false);
    /*    if let Some(_) = s.find_name::<Dialog>("ProgressDlg") {
        s.pop_layer(); //trouble
    }*/
    s.add_layer(
        Dialog::new()
            .title("User request Cancell")
            .content(TextView::new("Copying cancelled").center())
            .dismiss_button("OK"),
    );
}
/*let v = GLOBAL_FileManager.get();
let mut v = v.borrow_mut();
v.id = 1;*/
fn copy_engine(siv: &mut Cursive, path_from: Rc<PathBuf>, path_to: Rc<PathBuf>, is_recursive: bool, is_overwrite: bool) {
    // This is the callback channel
    let selected_path_from = (*path_from).clone();
    let selected_path_to = (*path_to).clone();
    let cb = siv.cb_sink().clone();
    siv.add_layer(
        Dialog::around(
            ProgressBar::new()
                // We need to know how many ticks represent a full bar.
                .range(0, selected_path_from.metadata(/*panic if dir*/).unwrap().size() as usize)
                .with_task(move |counter| {
                    let mut options = fs_extra::dir::CopyOptions::new();
                    options.overwrite = is_overwrite;
                    options.copy_inside = is_recursive;
                    // This closure will be called in a separate thread.
                    let handle = |process_info: fs_extra::TransitProcess| {
                        let v = GLOBAL_FileManager.get();
                        match v.lock().unwrap().borrow().tx_rx.1.try_recv() {
                            Ok(val) => {
                                if val as usize == fs_extra::dir::TransitProcessResult::Abort as usize {
                                    cb.send(Box::new(copying_cancelled)).unwrap();
                                    return fs_extra::dir::TransitProcessResult::Abort;
                                }
                            }
                            _ => { /*Do nothing, we are only interested in handling Abort*/ }
                        }
                        let percent = (process_info.file_bytes_copied as f64 / process_info.file_total_bytes as f64) * 100_000_f64;
                        counter.tick(percent as usize);
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
                    };

                    /*pub struct Error {
                        /// Type error
                        pub kind: ErrorKind,
                        message: String,
                    }

                    pub enum ErrorKind {
                        /// An entity was not found.
                        NotFound,
                        /// The operation lacked the necessary privileges to complete.
                        PermissionDenied,
                        /// An entity already exists.
                        AlreadyExists,
                        /// This operation was interrupted.
                        Interrupted,
                        /// Path does not a directory.
                        InvalidFolder,
                        /// Path does not a file.
                        InvalidFile,
                        /// Invalid file name.
                        InvalidFileName,
                        /// Invalid path.
                        InvalidPath,
                        /// Any I/O error.
                        Io(IoError),
                        /// Any StripPrefix error.
                        StripPrefix(StripPrefixError),
                        /// Any OsString error.
                        OsString(OsString),
                        /// Any fs_extra error not part of this list.
                        Other,
                    }
                    */
                    match fs_extra::copy_items_with_progress(&vec![selected_path_from.clone()], &selected_path_to, &options, handle) {
                        Ok(_) => {
                            // When we're done, send a callback through the channel
                            cb.send(Box::new(copying_finished_success)).unwrap()
                        }
                        Err(e) => match e.kind {
                            fs_extra::error::ErrorKind::NotFound => {}
                            fs_extra::error::ErrorKind::PermissionDenied => {}
                            fs_extra::error::ErrorKind::AlreadyExists => cb
                                .send(Box::new(move |s| {
                                    copying_already_exists(s, Rc::new(selected_path_from), Rc::new(selected_path_to), is_overwrite, is_recursive)
                                }))
                                .unwrap(),
                            fs_extra::error::ErrorKind::Interrupted => {}
                            fs_extra::error::ErrorKind::InvalidFolder => {}
                            fs_extra::error::ErrorKind::InvalidFile => {}
                            fs_extra::error::ErrorKind::InvalidFileName => {}
                            fs_extra::error::ErrorKind::InvalidPath => {}
                            fs_extra::error::ErrorKind::Io(IoError) => {}
                            fs_extra::error::ErrorKind::StripPrefix(StripPrefixError) => {}
                            fs_extra::error::ErrorKind::OsString(OsString) => {}
                            fs_extra::error::ErrorKind::Other => {}
                        },
                    }
                })
                .min_width(50)
                .max_width(50),
        )
        .button("Cancel", |s| {
            s.pop_layer();
            cancel_operation(s)
        })
        .with_name("ProgressDlg"),
    );
    siv.set_autorefresh(true);
}

fn ok_cpy_callback(siv: &mut Cursive, selected_path_from: Rc<PathBuf>, selected_path_to: Rc<PathBuf>, is_recursive: bool, is_overwrite: bool) {
    copy_engine(siv, selected_path_from, selected_path_to, is_recursive, is_overwrite);
}

fn create_cpy_dialog(path_from: String, path_to: String) -> NamedView<Dialog> {
    let mut cpy_dialog = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("Copy from:"))
            .child(EditView::new().content(path_from).with_name("cpy_from_edit_view").min_width(100))
            .child(DummyView)
            .child(TextView::new("Copy to:"))
            .child(EditView::new().content(path_to).with_name("cpy_to_edit_view").min_width(100))
            .child(DummyView)
            .child(
                LinearLayout::horizontal()
                    .child(
                        LinearLayout::horizontal()
                            .child(Checkbox::new().with_name("recursive_chck_bx"))
                            .child(TextView::new(" Recursive")),
                    )
                    .child(DummyView.min_width(5))
                    .child(
                        LinearLayout::horizontal()
                            .child(Checkbox::new().with_name("overwrite_chck_bx"))
                            .child(TextView::new(" Overwrite")),
                    )
                    .child(DummyView.min_width(5))
                    .child(
                        LinearLayout::horizontal()
                            .child(Checkbox::new().with_name("preserve_attribs_chck_bx"))
                            .child(TextView::new(" Preserve attributes")),
                    ),
            )
            .child(DummyView),
    )
    .title("Copy")
    .button("[ OK ]", |s| {
        let selected_path_from: Rc<String> = s
            .call_on_name("cpy_from_edit_view", move |an_edit_view: &mut EditView| an_edit_view.get_content())
            .unwrap();

        let selected_path_to: Rc<String> = s
            .call_on_name("cpy_to_edit_view", move |an_edit_view: &mut EditView| an_edit_view.get_content())
            .unwrap();
        let is_recursive = s
            .call_on_name("recursive_chck_bx", move |an_chck_bx: &mut Checkbox| an_chck_bx.is_checked())
            .unwrap();
        let is_overwrite = s
            .call_on_name("overwrite_chck_bx", move |an_chck_bx: &mut Checkbox| an_chck_bx.is_checked())
            .unwrap();
        /*Close our dialog*/
        s.pop_layer();

        ok_cpy_callback(
            s,
            Rc::new(PathBuf::from((*selected_path_from).clone())),
            Rc::new(PathBuf::from((*selected_path_to).clone())),
            is_recursive,
            is_overwrite,
        )
    })
    .button("[ Background ]", quit)
    .button("[ Cancel ]", |s| {
        s.pop_layer();
    });

    cpy_dialog.set_focus(DialogFocus::Button(0));

    cpy_dialog.with_name("DLG")
}
fn help(siv: &mut cursive::Cursive) {}
fn cancel_operation(siv: &mut cursive::Cursive) {
    let v = GLOBAL_FileManager.get();
    let tmp = v.lock().unwrap();
    let mut v = tmp.borrow_mut();
    v.tx_rx.0.send(fs_extra::dir::TransitProcessResult::Abort).unwrap();
    v.cancel_current_operation = true;
}
fn menu(siv: &mut cursive::Cursive) {}
fn view(siv: &mut cursive::Cursive) {}
fn edit(siv: &mut cursive::Cursive) {}
fn cpy(siv: &mut cursive::Cursive) {
    let left_panel_last_focus_time = siv
        .call_on_name("LeftPanel", move |a_table: &mut tableViewType| a_table.last_focus_time)
        .unwrap();

    let right_panel_last_focus_time = siv
        .call_on_name("RightPanel", move |a_table: &mut tableViewType| a_table.last_focus_time)
        .unwrap();
    let (from, to) = if left_panel_last_focus_time > right_panel_last_focus_time {
        ("LeftPanel", "RightPanel")
    } else {
        ("RightPanel", "LeftPanel")
    };
    match get_selected_path(siv, from) {
        Some(selected_path_from) => {
            let selected_path_to = get_current_dir(siv, to);
            siv.add_layer(create_cpy_dialog(selected_path_from, selected_path_to));
        }
        None => siv.add_layer(Atomic_Dialog::around(TextView::new("Please select item to copy")).dismiss_button("[ OK ]")),
    }
}
fn ren_mov(siv: &mut cursive::Cursive) {}
fn mkdir(siv: &mut cursive::Cursive) {}
fn del(siv: &mut cursive::Cursive) {}
fn pull_dn(siv: &mut cursive::Cursive) {}
fn quit(siv: &mut cursive::Cursive) {
    siv.quit();
}
fn start_dir_watcher_thread(siv: &mut Cursive, a_table_name: String, a_path: String, rx: Receiver<notify::DebouncedEvent>) {
    let cb_panel_update = siv.cb_sink().clone();
    let cb_panel_update_clone = cb_panel_update.clone();

    /*let (tx, rx) = channel();
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(5)).unwrap();
    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(a_path.clone(), RecursiveMode::NonRecursive).unwrap();
    let watcher = Arc::new(Mutex::new(watcher));*/
    std::thread::spawn(move || {
        // let v = GLOBAL_FileManager.get();
        // let tmp = v.lock().unwrap();
        //  let mut a_file_mngr = tmp.borrow_mut();
        loop {
            match rx.recv() {
                Ok(event) => {
                    let name = a_table_name.clone();
                    let path = a_path.clone(); //todo optimize
                                               //println!("{:?}", event);
                    cb_panel_update_clone.send(Box::new(|s| update_table(s, name, path))).unwrap();
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}
fn create_main_layout(siv: &mut cursive::CursiveRunnable, fm_config: &FileMangerConfig) {
    /*
                let v = GLOBAL_FileManager.get();
                let tmp = v.lock().unwrap();
                let mut a_file_mngr = tmp.borrow_mut();
    //            a_file_mngr.start_dir_watcher_thread(a_name,initial_path.clone());*/
    //let left_cb_sink = start_dir_watcher_thread(siv,String::from("LeftPanel"),String::from(&fm_config.left_panel_initial_path));
    let mut left_table = create_basic_table_core(siv, "LeftPanel", &fm_config.left_panel_initial_path);
    let left_info_item = TextView::new("Hello Dialog!").with_name("LeftPanelInfoItem");
    let left_layout = Atomic_Dialog::around(
        LinearLayout::vertical()
            .child(left_table.full_screen())
            .child(Delimiter::new("Title 1"))
            .child(left_info_item),
    )
    .title(fm_config.left_panel_initial_path.clone())
    .padding_lrtb(0, 0, 0, 0)
    .with_name("LeftPanelDlg");

    let mut right_table = create_basic_table_core(siv, "RightPanel", &fm_config.right_panel_initial_path);
    let right_info_item = TextView::new("Hello Dialog!").with_name("RightPanelInfoItem");
    let right_layout = Atomic_Dialog::around(
        LinearLayout::vertical()
            .child(right_table.full_screen())
            .child(Delimiter::new("Title 2"))
            .child(right_info_item),
    )
    .title(fm_config.right_panel_initial_path.clone()) //todo get name from table
    .padding_lrtb(0, 0, 0, 0)
    .with_name("RightPanelDlg");

    let button_help = OnEventView::new(TextView::new("[ Help ]"))
        .on_event('w', |s| s.quit())
        .on_event(event::Key::Tab, |s| s.quit());
    //    button_help.disable();
    //button_help.align
    let help_layout = LinearLayout::horizontal().child(TextView::new("1")).child(button_help);
    let button_menu = Button::new_raw("[ Menu ]", menu);
    let menu_layout = LinearLayout::horizontal().child(TextView::new("2")).child(button_menu);
    let button_view = Button::new_raw("[ View ]", view);
    let view_layout = LinearLayout::horizontal().child(TextView::new("3")).child(button_view);
    let button_edit = Button::new_raw("[ Edit ]", edit);
    let edit_layout = LinearLayout::horizontal().child(TextView::new("4")).child(button_edit);
    let button_cpy = Button::new_raw("[ Copy ]", cpy);
    let cpy_layout = LinearLayout::horizontal().child(TextView::new("5")).child(button_cpy);
    let button_RenMov = Button::new_raw("[ RenMov ]", ren_mov);
    let mut tv = TextView::new("6");
    tv.set_style(theme::ColorStyle::title_primary());
    let ren_mov_layout = LinearLayout::horizontal().child(tv).child(button_RenMov);
    let button_MkDir = Button::new_raw("[ MkDir ]", mkdir);
    let mkdir_layout = LinearLayout::horizontal().child(TextView::new("7")).child(button_MkDir);
    let button_Del = Button::new_raw("[ Del ]", del);
    let del_layout = LinearLayout::horizontal().child(TextView::new("8")).child(button_Del);
    let button_PullDn = Button::new_raw("[ PullDn ]", pull_dn);
    let pulldn_layout = LinearLayout::horizontal().child(TextView::new("9")).child(button_PullDn);
    let button_quit = Button::new_raw("[ Quit ]", quit);
    let quit_layout = LinearLayout::horizontal().child(TextView::new("10")).child(button_quit);
    let buttons_layout = LinearLayout::horizontal()
        .child(help_layout.full_width())
        .child(menu_layout.full_width())
        .child(view_layout.full_width())
        .child(edit_layout.full_width())
        .child(cpy_layout.full_width())
        .child(ren_mov_layout.full_width())
        .child(mkdir_layout.full_width())
        .child(del_layout.full_width())
        .child(pulldn_layout.full_width())
        .child(quit_layout);
    let left_right_layout = CircularFocus::new(LinearLayout::horizontal().child(left_layout).child(right_layout), true, true);
    let whole_layout = LinearLayout::vertical().child(left_right_layout).child(buttons_layout);
    siv.add_fullscreen_layer(whole_layout);
    //    siv.run();
}
fn fill_table_with_items(a_table: &mut tableViewType, a_dir: PathBuf) -> Result<(), std::io::Error> {
    let is = crate::internals::utils::read_directory(&a_dir)?;
    let mut items = Vec::new();
    if a_dir.parent().is_some() {
        items.push(ExplorerColumnData {
            name: format!(".."),
            size: 0,
            last_modify_time: SystemTime::now(),
        });
    }
    for a_path_buf in is {
        let is_dir = a_path_buf.metadata().unwrap().is_dir();
        let path_last_part = if is_dir {
            String::from(std::path::MAIN_SEPARATOR) + a_path_buf.file_name().unwrap().to_str().unwrap()
        } else {
            String::from(a_path_buf.file_name().unwrap().to_str().unwrap())
        };
        items.push(ExplorerColumnData {
            name: format!("{}", path_last_part),
            size: a_path_buf.metadata().unwrap().len(),
            last_modify_time: a_path_buf.metadata().unwrap().modified().unwrap(),
        });
    }
    let _ = a_table.take_items(); //clear before you put new, panic! otherwise will occur
    a_table.set_items(items);
    Ok(())
}
