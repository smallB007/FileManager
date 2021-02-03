use cursive::event::*;
use cursive::menu::MenuTree;
use cursive::traits::*;
use cursive::view::Boxable;
use cursive::views::{Button, LinearLayout, ProgressBar, TextArea};
use cursive::views::{CircularFocus, Dialog, NamedView, OnEventView, TextView};
use cursive::views::{DummyView, Panel};
use cursive::{Cursive, CursiveExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    io::{Error, ErrorKind},
    os::unix::prelude::MetadataExt,
};
// STD Dependencies -----------------------------------------------------------
// ----------------------------------------------------------------------------
//use std::cmp::Ordering;
// External Dependencies ------------------------------------------------------
// ----------------------------------------------------------------------------
use crate::internals::atomic_button::Atomic_Button;
use crate::internals::atomic_dialog::Atomic_Dialog;
use crate::internals::atomic_dialog_try::AtomicDialog;
use crate::internals::atomic_text_view::AtomicTextView;
use cursive::align::{HAlign, VAlign};
use cursive::traits::*;
use cursive::*;
// This examples shows how to configure and use a menubar at the top of the
// application.

pub fn create_main_menu(siv: &mut cursive::CursiveRunnable, showMenu: bool, alwaysVisible: bool) {
    //    let mut siv = cursive::default();

    // We'll use a counter to name new files.
    let counter = AtomicUsize::new(1);

    // The menubar is a list of (label, menu tree) pairs.
    siv.menubar()
        // We add a new "File" tree
        .add_subtree(
            "File",
            MenuTree::new()
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
                    MenuTree::new().with(|tree| {
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
            MenuTree::new()
                .subtree(
                    "Help",
                    MenuTree::new()
                        .leaf("General", |s| s.add_layer(Dialog::info("Help message!")))
                        .leaf("Online", |s| {
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
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.add_global_callback(Key::Tab, switch_panel); //todo not working
    siv.add_global_callback(Key::F10, quit);
    siv.add_global_callback(Key::F4, cpy);
    //  siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));

    //siv.run();
}
fn switch_panel(siv: &mut cursive::Cursive) {
    siv.add_layer(Dialog::text("Hit <Esc> to show the menu!"));
}
// Modules --------------------------------------------------------------------
// ----------------------------------------------------------------------------
use super::cursive_table_view::{TableView, TableViewItem};
use chrono::offset::Utc;
use chrono::DateTime;
use std::time::SystemTime;
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
pub fn create_basic_table_core(a_name: &'static str, initial_path: &PathBuf) -> NamedView<tableViewType> {
    let mut table = tableViewType::new()
        .column(ExplorerColumn::Name, "Name", |c| c.width_percent(60))
        .column(ExplorerColumn::Size, "Size", |c| c.align(cursive::align::HAlign::Center))
        .column(ExplorerColumn::LastModifyTime, "LastModifyTime", |c| {
            c.ordering(std::cmp::Ordering::Greater).align(HAlign::Right).width_percent(20)
        });

    fill_table_with_items(&mut table, &std::path::PathBuf::from(initial_path));
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
            .call_on_name(&(String::from(a_name) + &String::from("InfoItem")), move |a_dlg: &mut Atomic_Dialog| {
                a_dlg.set_title(current_item.name.clone());
                a_dlg.set_content(TextView::new(current_item.name.clone()));
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
        let current_path = siv
            .call_on_name(&(String::from(a_name) + &String::from("Dlg")), move |a_dlg: &mut Atomic_Dialog| {
                //format!("{:?}", a_table.borrow_item(index).unwrap())
                a_dlg.get_title()
            })
            .unwrap();
        let new_path = siv
            .call_on_name(a_name, move |a_table: &mut tableViewType| {
                let selected_item = a_table.borrow_item(index).unwrap().name.clone();
                let whole_path = match selected_item.as_str() {
                    ".." => match PathBuf::from(current_path).parent() {
                        Some(parent) => PathBuf::from(parent),
                        None => PathBuf::from("NO_PARENT"),
                    },
                    _ => {
                        if PathBuf::from(current_path.clone() + &selected_item.clone()).is_dir() {
                            let mut removed_first_slash: String = selected_item.clone();
                            removed_first_slash.remove(0);
                            let mut full_path = PathBuf::from(current_path);
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
            let mut res = Option::<std::io::Error>::default();
            siv.call_on_name(a_name, |a_table: &mut tableViewType| {
                res = fill_table_with_items(a_table, &new_path.clone()).err();
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
fn get_selected_path(siv: &mut Cursive, a_name: &str) -> PathBuf {
    let mut item_from_inx = usize::MAX;
    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
        if let Some(inx) = a_table.item() {
            item_from_inx = inx;
        };
    });
    let selected_path = get_selected_path_from_inx(siv, a_name, item_from_inx);
    selected_path
}
fn get_current_dir(siv: &mut Cursive, a_name: &str) -> PathBuf {
    let current_path = siv
        .call_on_name(&(String::from(a_name) + &String::from("Dlg")), move |a_dlg: &mut Atomic_Dialog| {
            a_dlg.get_title()
        })
        .unwrap();
    PathBuf::from(current_path)
}
fn get_selected_path_from_inx(siv: &mut Cursive, a_name: &str, index: usize) -> PathBuf {
    /*Todo repeat*/
    let current_path = get_current_dir(siv, a_name);
    let new_path = siv
        .call_on_name(a_name, move |a_table: &mut tableViewType| {
            let selected_item = a_table.borrow_item(index).unwrap().name.clone();
            let whole_path = match selected_item.as_str() {
                ".." => current_path,
                _ => current_path.clone().join(PathBuf::from(&selected_item)),
            };
            whole_path
        })
        .unwrap();
    new_path
}
// Function to simulate a long process.
use cursive::utils::Counter;
use std::thread;
use std::time::Duration;
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
fn copying_cancelled(s: &mut Cursive) {
    s.set_autorefresh(false);
    s.pop_layer(); //trouble
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
use fs_extra::dir::copy;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
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
    let selected_path_from = get_selected_path(siv, from);
    let selected_path_to = get_current_dir(siv, to);
    // This is the callback channel
    let cb = siv.cb_sink().clone();
    siv.add_layer(
        Dialog::around(
            ProgressBar::new()
                // We need to know how many ticks represent a full bar.
                .range(0, selected_path_from.metadata().unwrap().size() as usize)
                .with_task(move |counter| {
                    let options = fs_extra::dir::CopyOptions::new();
                    // This closure will be called in a separate thread.
                    let handle = |process_info: fs_extra::TransitProcess| {
                        let v = GLOBAL_FileManager.get();
                        match v.lock().unwrap().borrow().tx_rx.1.try_recv() {
                            Ok(val) => {
                                if val as usize == fs_extra::dir::TransitProcessResult::Abort as usize {
                                    cb.send(Box::new(copying_cancelled)).unwrap();
                                    panic!("User cancelled copying. Thread terminated");
                                }
                            }
                            _ => {/*Do nothing, we are only interested in handling Abort*/}
                        }
                        let percent = (process_info.file_bytes_copied as f64 / process_info.file_total_bytes as f64) * 100_000_f64;
                        counter.tick(percent as usize);
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
                    };
                    fs_extra::copy_items_with_progress(&vec![selected_path_from], &selected_path_to, &options, handle).unwrap();
                    // When we're done, send a callback through the channel
                    cb.send(Box::new(copying_finished_success)).unwrap();
                })
                .min_width(50)
                .max_width(50),
        )
        .button("Cancel", cancel_operation),
    );
    siv.set_autorefresh(true);
}
fn ren_mov(siv: &mut cursive::Cursive) {}
fn mkdir(siv: &mut cursive::Cursive) {}
fn del(siv: &mut cursive::Cursive) {}
fn pull_dn(siv: &mut cursive::Cursive) {}
fn quit(siv: &mut cursive::Cursive) {
    siv.quit();
}
pub fn create_main_layout(siv: &mut cursive::CursiveRunnable) {
    let initial_path = String::from("/home/artie/Desktop/Left");
    let mut left_table = create_basic_table_core("LeftPanel", &PathBuf::from(initial_path.clone()));
    /*   let mut items = Vec::new();
    items.push(ExplorerColumnData {
        name: format!(".."),
        size: 0,
        last_modify_time: SystemTime::now(),
    });
    left_table.get_mut().set_items(items);*/
    let right_table = create_basic_table_core("RightPanel", &PathBuf::from(initial_path.clone()));
    let left_main_panel_view = Atomic_Dialog::around(left_table.full_screen())
        .title(initial_path.clone())
        .with_name("LeftPanelDlg");
    let mut left_info_item = Atomic_Dialog::around(TextView::new("Hello Dialog!"))
        .title("Left")
        .with_name("LeftPanelInfoItem");
    left_info_item.get_mut().set_title_position(HAlign::Right);
    left_info_item.get_mut().set_title_position_vert(VAlign::Bottom);
    //    left_info_item.set_title_position(HAlign::Left);
    let left_layout = LinearLayout::vertical().child(left_main_panel_view).child(left_info_item);
    let right_main_panel_view = Atomic_Dialog::around(right_table.full_screen())
        .title(initial_path.clone())
        .with_name("RightPanelDlg");
    let mut right_info_item = Atomic_Dialog::around(TextView::new("Hello Dialog!"))
        .title("Right")
        .with_name("RightPanelInfoItem");
    //    right_info_item.set_title_position(HAlign::Left);
    let right_layout = LinearLayout::vertical().child(right_main_panel_view).child(right_info_item);
    //    let hm = HashMap::new();
    //let button_help = Button::new_raw("[ Help ]", help);
    //    let mut button_help = AtomicTextView::new("[ Help ]");
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
    let left_right_layout = LinearLayout::horizontal().child(left_layout).child(right_layout);
    let whole_layout = LinearLayout::vertical().child(left_right_layout).child(buttons_layout);
    siv.add_fullscreen_layer(whole_layout);
    //    siv.run();
}
fn read_config() {}
use config::Config;
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
use std::sync::Mutex;
static GLOBAL_FileManager: state::Storage<std::sync::Mutex<std::cell::RefCell<FileManager>>> = state::Storage::new();
//static GLOBAL_FileManager: state::LocalStorage<std::cell::RefCell<FileManager>> = state::LocalStorage::new();
impl FileManager {
    fn init(&self, mut siv: &mut cursive::CursiveRunnable) {
        read_config();
        create_main_menu(&mut siv, true, true);

        create_main_layout(&mut siv);
    }
    pub fn new(mut siv: &mut cursive::CursiveRunnable) {
        GLOBAL_FileManager.set(std::sync::Mutex::new(std::cell::RefCell::new(FileManager::default())));
        let v = GLOBAL_FileManager.get();
        let tmp = v.lock().unwrap();
        let fm = tmp.borrow_mut();
        //let fm = FileManager{id:1};
        fm.init(&mut siv);
    }
}
fn fill_table_with_items(a_table: &mut tableViewType, a_dir: &PathBuf) -> Result<(), std::io::Error> {
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
