#![forbid(unreachable_patterns)]
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
use theme::BaseColor;
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
use std::sync::{Arc, Condvar, Mutex};
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
use crate::internals::literals::copy_progress_dlg;
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
#[derive(Copy, Clone)]
enum AtomicFileTransitFlags {
    Abort,
    Skip,
}
struct CpyData {
    cond_var: Arc<(Mutex<bool>, Condvar)>,
    files_total: usize,
}
pub struct FileManager {
    id: i64,
    active_table: String, //change to &str
    tx_rx: (Sender<AtomicFileTransitFlags>, Receiver<AtomicFileTransitFlags>),
    cpy_data: Option<CpyData>,
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
                .leaf("New", move |siv| {
                    // Here we use the counter to add an entry
                    // in the list of "Recent" items.
                    let i = counter.fetch_add(1, Ordering::Relaxed);
                    let filename = format!("New {}", i);
                    siv.menubar()
                        .find_subtree("File")
                        .unwrap()
                        .find_subtree("Recent")
                        .unwrap()
                        .insert_leaf(0, filename, |_| ());

                    siv.add_layer(Dialog::info("New file!"));
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
                    Tree::new()
                        .leaf("General", |siv| siv.add_layer(Dialog::info("Help message!")))
                        .leaf("Online", |siv| {
                            let text = "Google it yourself!\n\
                                        Kids, these days...";
                            siv.add_layer(Dialog::info(text))
                        }),
                )
                .leaf("About", |siv| siv.add_layer(Dialog::info("Cursive v0.0.0"))),
        )
        .add_delimiter()
        .add_leaf("Quit", |siv| siv.quit());

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
    siv.add_global_callback(Key::F7, show_hide_cpy);
    //  siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));

    //siv.run();
}
fn switch_panel(siv: &mut cursive::Cursive) {
    if let Some(mut dialog) = siv.find_name::<Dialog>("DLG") {
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
                .button("Close", |siv| {
                    siv.pop_layer();
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
                .button("Close", move |siv| {
                    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
                        a_table.remove_item(index);
                    });
                    siv.pop_layer();
                }),
        );*/
    });
    table.set_selected_row(0);
    table.set_on_submit(move |siv: &mut Cursive, row: usize, index: usize| {
        siv.call_on_name(a_name, |a_table: &mut tableViewType| {
            a_table.clear_selected_items();
        });

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
                .button("Close", move |siv| {
                    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
                        a_table.remove_item(index);
                    });
                    siv.pop_layer();
                }),
        );*/
    });
    let named_view_table = table.with_name(a_name);

    named_view_table
}
type TableNameT = String;
type PathT = String;
type IndexT = usize;
type CopyPathInfoT = Vec<(TableNameT, PathT, IndexT)>;
fn get_selected_path(siv: &mut Cursive, a_name: &str) -> Option<CopyPathInfoT> {
    let mut selected_items_inx = std::collections::BTreeSet::<usize>::new();
    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
        selected_items_inx = a_table.get_selected_items();
    });

    if selected_items_inx.len() != 0 {
        let mut selected_paths = CopyPathInfoT::new();
        for selected_inx in selected_items_inx {
            selected_paths.push(get_selected_path_from_inx(siv, a_name, selected_inx).unwrap());
        }

        Some(selected_paths)
    } else {
        None
    }
}

fn get_current_dir(siv: &mut Cursive, a_name: &str) -> String {
    let current_dir = siv
        .call_on_name(&(String::from(a_name) + &String::from("Dlg")), move |a_dlg: &mut Atomic_Dialog| {
            a_dlg.get_title()
        })
        .unwrap();
    current_dir
}
fn get_selected_path_from_inx(siv: &mut Cursive, a_name: &str, index: usize) -> Option<(TableNameT, PathT, IndexT)> {
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
                    Some((a_name.to_owned(), current_dir + &selected_item, index))
                }
            };
            whole_path
        })
        .unwrap();
    new_path
}
// Function to simulate a long process.
fn copying_error(siv: &mut Cursive) {
    siv.set_autorefresh(false);
    siv.pop_layer(); //trouble
    siv.add_layer(
        Dialog::new()
            .title("Copying error")
            .content(TextView::new("Copying ERROR").center())
            .dismiss_button("OK"),
    );
}
fn copying_already_exists(siv: &mut Cursive, path_from: PathBuf, path_to: PathBuf, is_overwrite: bool, is_recursive: bool) {
    let theme = siv.current_theme().clone().with(|theme| {
        theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Red);
        theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
        theme.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Yellow);
        theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Black);
    });
    siv.set_autorefresh(false); //todo repeat
    todo!("Dialog type changed");
    if let Some(_) = siv.find_name::<ProgressDlgT>("ProgressDlg") {
        siv.pop_layer();
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
                    .child(
                        LinearLayout::horizontal()
                            .child(Checkbox::new())
                            .child(TextView::new(" Don't overwrite with zero length file")),
                    ),
            )
            .child(DummyView),
    )
    .title("File Exists")
    .button("Overwrite", move |siv| {
        siv.pop_layer();
        todo!();
        //        ok_cpy_callback(siv, vec![String::from(path_from.to_str().unwrap())], path_to.clone(), is_recursive, true)
    })
    .button("Older", |siv| {})
    .button("Smaller", |siv| {})
    .button("Different size", |siv| {})
    .button("Append", |siv| {})
    .button("Skip", |siv| {})
    .button("Abort", |siv| {});

    siv.add_layer(views::ThemedView::new(theme, Layer::new(file_exist_dlg)));
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
fn cannot_suspend_copy(siv: &mut Cursive) {
    siv.set_autorefresh(false);
    if let Some(_) = siv.find_name::<ResizedView<Dialog>>("ProgressDlg") {
        siv.pop_layer();
    }
    siv.add_layer(
        Dialog::new()
            .title("Cannot suspend")
            .content(TextView::new("Cannot suspend").center())
            .dismiss_button("OK"),
    );
}
fn end_copying_helper(siv: &mut Cursive, title: &str, text: &str) {
    let g_file_manager = GLOBAL_FileManager.get();
    g_file_manager.lock().unwrap().borrow_mut().cpy_data = None;
    show_progress_cpy(siv, 0, false);
    siv.set_autorefresh(false);
    if let Some(_) = siv.find_name::<ProgressDlgT>("ProgressDlg") {
        siv.pop_layer();
    }
    siv.add_layer(Dialog::new().title(title).content(TextView::new(text).center()).dismiss_button("OK"));
}
fn copying_finished_success(siv: &mut Cursive) {
    end_copying_helper(siv, "Copying finished", "Copying finished successfully");
}
fn copying_cancelled(siv: &mut Cursive) {
    end_copying_helper(siv, "User request cancel", "Copying cancelled");
}

fn update_cpy_dlg(siv: &mut Cursive, process_info: fs_extra::file::TransitProcess, file_name: String, current_inx: usize) {
    /*   siv.call_on_name("TextView_copying_x_of_n", |a_text_view: &mut TextView| {
        a_text_view.set_content(format!("Copying {} of {}", process_info.copied_bytes, process_info.total_bytes));
    })
    .unwrap();*/
    siv.call_on_name(copy_progress_dlg::widget_names::progress_bar_total, |a_progress_bar: &mut ProgressBar| {
        a_progress_bar.set_value(current_inx);
        a_progress_bar.set_label(|val, (_min, max)| format!("Copied {} of {}", val, max));
    });
    siv.call_on_name("hideable_cpy_prgrs_br", |hideable_view_total: &mut HideableView<ResizedView<ProgressBar>>| {
        hideable_view_total.get_inner_mut().get_inner_mut().set_value(current_inx);
    });
    siv.call_on_name("TextView_copying_x", |a_text_view: &mut TextView| {
        a_text_view.set_content(format!("Copying:\n {}", file_name));
    });
    siv.call_on_name(copy_progress_dlg::widget_names::progress_bar_current, |a_progress_bar: &mut ProgressBar| {
        let current_file_percent = ((process_info.copied_bytes as f64 / process_info.total_bytes as f64) * 100_f64) as usize;
        a_progress_bar.set_value(current_file_percent);
    });
}
#[cfg(ETA)]
struct file_transfer_context {
    eta_secs: u64,
    bps: u64,
    bps_time: u64,
}
fn unselect_inx(siv: &mut Cursive, a_table_name: Arc<String>, inx: Arc<usize>) {
    siv.call_on_name(a_table_name.as_str(), |a_table: &mut tableViewType| {
        a_table.clear_selected_item(*inx);
    });
}
fn cpy_task(selected_paths: CopyPathInfoT, path_to: String, cb: CbSink, cond_var: Arc<(Mutex<bool>, Condvar)>) {
    let start = std::time::Instant::now();
    for (current_inx, (table_name, current_file, inx)) in selected_paths.iter().enumerate() {
        let progres_handler = |process_info: fs_extra::file::TransitProcess| {
            let v = GLOBAL_FileManager.get();
            match v.lock().unwrap().borrow().tx_rx.1.try_recv() {
                Ok(ref val) => {
                    if *val as usize == AtomicFileTransitFlags::Abort as usize {
                        //                        std::thread::park();
                        cb.send(Box::new(copying_cancelled)).unwrap();
                        return fs_extra::dir::TransitProcessResult::Abort;
                    }
                }
                _ => { /*Do nothing, we are only interested in handling Abort*/ }
            }

            let (lock, cvar) = &*cond_var;
            match cvar.wait_while(lock.lock().unwrap(), |pending| *pending) {
                Err(err) => cb.send(Box::new(cannot_suspend_copy)).unwrap(),
                _ => {}
            }

            /*let tid = std::thread::current().id();
            let tid = std::thread::current().id();*/
            //println!("ThreadID inside handler: {:?}",tid);
            let current_file_clone = current_file.clone();
            cb.send(Box::new(move |siv| update_cpy_dlg(siv, process_info, current_file_clone, current_inx)));
            TransitProcessResult::ContinueOrAbort
        };

        //            println!("ThreadID outside handler: {:?}",tid);
        let options = fs_extra::file::CopyOptions::new(); //Initialize default values for CopyOptions
        let current_file_name = PathBuf::from(current_file.clone());
        let current_file_name = current_file_name.file_name().unwrap().to_str().unwrap();
        let full_path_to = path_to.clone() + "/" + current_file_name;
        match fs_extra::file::copy_with_progress(current_file, full_path_to, &options, progres_handler) {
            Ok(val) => {
                let inx_clone = Arc::new(*inx);
                let table_name_clone = Arc::new(table_name.clone());
                cb.send(Box::new(|s| unselect_inx(s, table_name_clone, inx_clone))).unwrap();
            }
            Err(err) => {
                println!("err: {}", err)
            }
        }
    }
    let duration = start.elapsed();
    println!("Copying finished:{}", duration.as_secs());
}
const a_const: i128 = 0;
fn suspend_cpy_thread(siv: &mut Cursive, cond_var: Arc<(Mutex<bool>, Condvar)>) {
    let mut resume_thread = cond_var.0.lock().unwrap();
    *resume_thread = if resume_thread.cmp(&true) == std::cmp::Ordering::Equal {
        siv.call_on_name("Suspend_Resume_Btn", move |a_button: &mut Button| a_button.set_label("Suspend"))
            .unwrap();
        false
    } else {
        siv.call_on_name("Suspend_Resume_Btn", move |a_button: &mut Button| a_button.set_label("Resume"));
        true
    };
    cond_var.1.notify_one();
}
type CopyProgressDlgT = NamedView<ResizedView<Dialog>>;
fn create_cpy_progress_dialog(files_total: usize, cond_var: Arc<(Mutex<bool>, Condvar)>) -> CopyProgressDlgT {
    let hideable_total = HideableView::new(
        LinearLayout::vertical()
            .child(TextView::new(copy_progress_dlg::labels::copying_progress_total).with_name(copy_progress_dlg::widget_names::text_view_copying_total))
            .child(ProgressBar::new()
                    .range(0, files_total)
                    .with_name(copy_progress_dlg::widget_names::progress_bar_total),
            )
            .child(DummyView),
    )
    .visible(files_total > 1);

    let suspend_button = Button::new("Suspend", move |siv| suspend_cpy_thread(siv, cond_var.clone())).with_name("Suspend_Resume_Btn");
    let background_button = Button::new("Background", move |siv| {
        show_progress_cpy(siv, files_total, true);
        siv.pop_layer();
    });
    let cancel_button = Button::new("Cancel", |siv| {
        siv.pop_layer(); //yes but make sure that update isn't proceeding ;)
        cancel_operation(siv)
    });
    let buttons = LinearLayout::horizontal()
        .child(suspend_button)
        .child(DummyView)
        .child(background_button)
        .child(DummyView)
        .child(cancel_button);

    let cpy_progress_dlg = Dialog::around(
        LinearLayout::vertical().child(hideable_total).child(
            LinearLayout::vertical()
                .child(TextView::new("").with_name("TextView_copying_x"))
                .child(
                    ProgressBar::new()
                        .range(0, 100)
                        .with_name(copy_progress_dlg::widget_names::progress_bar_current),
                )
                .child(DummyView)
                .child(buttons),
        ),
    )
    .fixed_width(80)
    .with_name("ProgressDlg");

    cpy_progress_dlg
}
fn copy_engine(siv: &mut Cursive, paths_from: CopyPathInfoT, path_to: PathBuf, is_recursive: bool, is_overwrite: bool, is_background_cpy: bool) {
    let cond_var = Arc::new((Mutex::new(false), Condvar::new()));
    let cond_var_clone = Arc::clone(&cond_var);
    let paths_from_clone = paths_from.clone();
    let path_to_clone: String = String::from(path_to.as_os_str().to_str().unwrap());

    let cb = siv.cb_sink().clone();
    #[cfg(feature = "serial_cpy")]
    let handle = std::thread::spawn(move || {
        cpy_task(paths_from_clone /*todo clone here?*/, path_to_clone, cb.clone(), cond_var_clone);
        cb.send(Box::new(|siv| copying_finished_success(siv)));
    });
    let g_file_manager = GLOBAL_FileManager.get();
    g_file_manager.lock().unwrap().borrow_mut().cpy_data = Some(CpyData {
        cond_var: cond_var.clone(),
        files_total: paths_from.len(),
    });

    if !is_background_cpy {
        let cpy_progress_dlg = create_cpy_progress_dialog(paths_from.len(), cond_var);
        let cpy_progress_dlg = create_themed_view(siv, cpy_progress_dlg);
        siv.add_layer(cpy_progress_dlg);
        siv.set_autorefresh(true);
    }
}

fn ok_cpy_callback(siv: &mut Cursive, selected_paths_from: CopyPathInfoT, selected_path_to: PathBuf, is_recursive: bool, is_overwrite: bool) {
    copy_engine(siv, selected_paths_from, selected_path_to, is_recursive, is_overwrite, false);
}
fn show_progress_cpy(siv: &mut Cursive, total_files: usize, show_progress_bar: bool) {
    siv.call_on_name("hideable_cpy_button", |hideable_cpy_btn: &mut HideableView<Button>| {
        hideable_cpy_btn.set_visible(!show_progress_bar);
    });
    siv.call_on_name("left_bracket_hideable", |hideable_bracket: &mut HideableView<TextView>| {
        hideable_bracket.set_visible(show_progress_bar);
    });
    siv.call_on_name("hideable_cpy_prgrs_br", |hideable_view_total: &mut HideableView<ResizedView<ProgressBar>>| {
        hideable_view_total.set_visible(show_progress_bar);
        hideable_view_total.get_inner_mut().get_inner_mut().set_range(0, total_files);
    });
    siv.call_on_name("right_bracket_hideable", |hideable_bracket: &mut HideableView<TextView>| {
        hideable_bracket.set_visible(show_progress_bar);
    });
}
fn background_cpy_callback(siv: &mut Cursive, selected_paths_from: CopyPathInfoT, selected_path_to: PathBuf, is_recursive: bool, is_overwrite: bool) {
    show_progress_cpy(siv, selected_paths_from.len(), true);
    copy_engine(siv, selected_paths_from, selected_path_to, is_recursive, is_overwrite, true);
}

fn create_cpy_dialog(siv: &mut Cursive, paths_from: CopyPathInfoT, path_to: String) -> NamedView<Dialog> {
    let paths_from_clone = paths_from.clone();
    let mut cpy_dialog = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(format!("Copy {} items with mask:", paths_from.len())))
            .child(EditView::new().content("*").with_name("cpy_from_edit_view").min_width(100))
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
    .button("[ OK ]", move |siv| {
        let selected_mask_from: Rc<String> = siv
            .call_on_name("cpy_from_edit_view", move |an_edit_view: &mut EditView| an_edit_view.get_content())
            .unwrap();

        let selected_path_to: Rc<String> = siv
            .call_on_name("cpy_to_edit_view", move |an_edit_view: &mut EditView| an_edit_view.get_content())
            .unwrap();
        let is_recursive = siv
            .call_on_name("recursive_chck_bx", move |an_chck_bx: &mut Checkbox| an_chck_bx.is_checked())
            .unwrap();
        let is_overwrite = siv
            .call_on_name("overwrite_chck_bx", move |an_chck_bx: &mut Checkbox| an_chck_bx.is_checked())
            .unwrap();
        /*Close our dialog*/
        siv.pop_layer();

        ok_cpy_callback(siv, paths_from.clone(), PathBuf::from((*selected_path_to).clone()), is_recursive, is_overwrite)
    })
    .button("[ Background ]", move |siv| {
        let selected_mask_from: Rc<String> = siv
            .call_on_name("cpy_from_edit_view", move |an_edit_view: &mut EditView| an_edit_view.get_content())
            .unwrap();

        let selected_path_to: Rc<String> = siv
            .call_on_name("cpy_to_edit_view", move |an_edit_view: &mut EditView| an_edit_view.get_content())
            .unwrap();
        let is_recursive = siv
            .call_on_name("recursive_chck_bx", move |an_chck_bx: &mut Checkbox| an_chck_bx.is_checked())
            .unwrap();
        let is_overwrite = siv
            .call_on_name("overwrite_chck_bx", move |an_chck_bx: &mut Checkbox| an_chck_bx.is_checked())
            .unwrap();
        /*Close our dialog*/
        siv.pop_layer();

        background_cpy_callback(
            siv,
            paths_from_clone.clone(),
            PathBuf::from((*selected_path_to).clone()),
            is_recursive,
            is_overwrite,
        )
    })
    .button("[ Cancel ]", |siv| {
        siv.pop_layer();
    });

    cpy_dialog.set_focus(DialogFocus::Button(0));

    cpy_dialog.with_name("DLG")
}
fn help(siv: &mut cursive::Cursive) {}
fn cancel_operation(siv: &mut cursive::Cursive) {
    let v = GLOBAL_FileManager.get();
    let tmp = v.lock().unwrap();
    let mut v = tmp.borrow_mut();
    v.tx_rx.0.send(AtomicFileTransitFlags::Abort).unwrap();
}
type ProgressDlgT = ResizedView<Dialog>;
fn show_hide_cpy(siv: &mut cursive::Cursive) {
    if let Some(_) = siv.find_name::<ProgressDlgT>("ProgressDlg") {
        siv.pop_layer(); //trouble
    } else {
        let g_file_manager = GLOBAL_FileManager.get();
        match &g_file_manager.lock().unwrap().borrow().cpy_data {
            Some(cpy_data) => {
                let cpy_progress_dlg = create_cpy_progress_dialog(cpy_data.files_total, cpy_data.cond_var.clone());
                siv.add_layer(cpy_progress_dlg);
                siv.set_autorefresh(true);
            }
            None => {}
        }
    }
}
fn create_themed_view<T>(siv: &mut Cursive, view: T) -> ThemedView<Layer<T>>
where
    T: View,
{
    /*theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
            theme.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Yellow);
            theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Black);
    *//*
     /// Color used for the application background.
        Background,
        /// Color used for View shadows.
        Shadow,
        /// Color used for View backgrounds.
        View,
        /// Primary color used for the text.
        Primary,
        /// Secondary color used for the text.
        Secondary,
        /// Tertiary color used for the text.
        Tertiary,
        /// Primary color used for title text.
        TitlePrimary,
        /// Secondary color used for title text.
        TitleSecondary,
        /// Color used for highlighting text.
        Highlight,
        /// Color used for highlighting inactive text.
        HighlightInactive,
        /// Color used for highlighted text
        HighlightText,*/
    let curr_theme = siv.current_theme().clone();
    let theme = siv.current_theme().clone().with(|theme| {
        let color_view = match curr_theme.palette[theme::PaletteColor::View] {
            theme::Color::Dark(BaseColor) => BaseColor.light(),
            theme::Color::Light(BaseColor) => BaseColor.dark(),
            theme::Color::Rgb(R, G, B) => theme::Color::Rgb(R + 1, G + 1, B + 1),
            theme::Color::RgbLowRes(R, G, B) => theme::Color::RgbLowRes(R + 1, G + 1, B + 1),
            TerminalDefault => theme::Color::Rgb(1, 1, 1),
        };
        let color_highlight_text = match curr_theme.palette[theme::PaletteColor::HighlightText] {
            theme::Color::Dark(BaseColor) => BaseColor.light(),
            theme::Color::Light(BaseColor) => BaseColor.dark(),
            theme::Color::Rgb(R, G, B) => theme::Color::Rgb(R + 1, G + 1, B + 1),
            theme::Color::RgbLowRes(R, G, B) => theme::Color::RgbLowRes(R + 1, G + 1, B + 1),
            TerminalDefault => theme::Color::Rgb(1, 1, 1),
        };
        theme.palette[theme::PaletteColor::View] = color_view;
        theme.palette[theme::PaletteColor::HighlightText] = color_highlight_text;
        /*theme.palette[theme::PaletteColor::Background] = color_primary;
                            theme.palette[theme::PaletteColor::Shadow] = color_primary;
                            theme.palette[theme::PaletteColor::Primary] = color_primary;
                            theme.palette[theme::PaletteColor::Secondary] = color_primary;
                            theme.palette[theme::PaletteColor::Tertiary] = color_primary;
                            theme.palette[theme::PaletteColor::TitlePrimary] = color_primary;
                            theme.palette[theme::PaletteColor::TitleSecondary] = color_primary;
        //                    theme.palette[theme::PaletteColor::Highlight] = color_primary;
                            theme.palette[theme::PaletteColor::HighlightInactive] = color_primary;
        //                    theme.palette[theme::PaletteColor::Highlight] = color_primary;*/
    });
    views::ThemedView::new(theme, Layer::new(view))
}
fn menu(siv: &mut cursive::Cursive) {}
fn view(siv: &mut cursive::Cursive) {}
fn edit(siv: &mut cursive::Cursive) {}
fn cpy(siv: &mut cursive::Cursive) {
    /*First, check if copying is in the progress:*/
    if let Some(ref cpy_data) = GLOBAL_FileManager.get().lock().unwrap().borrow().cpy_data {
        if let None = siv.find_name::<ProgressDlgT>("ProgressDlg") {
            let cpy_progress_dlg = create_cpy_progress_dialog(cpy_data.files_total, cpy_data.cond_var.clone());
            siv.add_layer(cpy_progress_dlg);
            siv.set_autorefresh(true);
        }
    } else {
        /*No copying, let'siv start it then ;)*/
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
            Some(selected_paths_from) => {
                let selected_path_to = get_current_dir(siv, to);
                let cpy_dlg = create_cpy_dialog(siv, selected_paths_from, selected_path_to);
                let themed_cpy_dlg = create_themed_view(siv, cpy_dlg);
                siv.add_layer(themed_cpy_dlg);
            }
            None => siv.add_layer(Atomic_Dialog::around(TextView::new("Please select item to copy")).dismiss_button("[ OK ]")),
        }
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
                    cb_panel_update_clone.send(Box::new(|siv| update_table(siv, name, path))).unwrap();
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
        .on_event('w', |siv| siv.quit())
        .on_event(event::Key::Tab, |siv| siv.quit());
    //    button_help.disable();
    //button_help.align
    let help_layout = LinearLayout::horizontal().child(TextView::new("1")).child(button_help);
    let button_menu = Button::new_raw("[ Menu ]", menu);
    let menu_layout = LinearLayout::horizontal().child(TextView::new("2")).child(button_menu);
    let button_view = Button::new_raw("[ View ]", view);
    let view_layout = LinearLayout::horizontal().child(TextView::new("3")).child(button_view);
    let button_edit = Button::new_raw("[ Edit ]", edit);
    let edit_layout = LinearLayout::horizontal().child(TextView::new("4")).child(button_edit);
    let mouse_event = event::Event::Mouse {
        offset: XY::new(0, 0),
        position: XY::new(1, 1),
        event: MouseEvent::Press(MouseButton::Left),
    };
    let fn_with_label = |val, (min, max)| copy_progress_dlg::labels::copying_progress_total_background.to_owned();
    let ProgressBar_on_event_view = HideableView::new(
        ProgressBar::new()
            .with_label(fn_with_label)
            .min_width(copy_progress_dlg::labels::copying_progress_total_background.len()),
    )
    .visible(false)
    .with_name("hideable_cpy_prgrs_br");
    let left_bracket_hideable = HideableView::new(TextView::new("[")).visible(false).with_name("left_bracket_hideable");
    let right_bracket_hideable = HideableView::new(TextView::new("]")).visible(false).with_name("right_bracket_hideable");
    let button_cpy = HideableView::new(Button::new_raw("[ Copy ]", cpy))
        .visible(true)
        .with_name("hideable_cpy_button");
    let cpy_layout = LinearLayout::horizontal()
        .child(TextView::new("5"))
        .child(button_cpy)
        .child(left_bracket_hideable)
        .child(ProgressBar_on_event_view)
        .child(right_bracket_hideable);
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
    /*    let paths_from = vec!["file.txt".to_owned(),"file.txt".to_owned()];
    siv.add_layer(create_cpy_progress_dialog(siv,paths_from,Rc::new(PathBuf::from("path_to")),false,false));*/
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
