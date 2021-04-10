#![forbid(unreachable_patterns)]
use archive_readers::{ArchiveReader, ZipArchiveReader};
use cursive::event::*;
use cursive::menu::Tree;
use cursive::traits::*;
use cursive::traits::*;
use cursive::utils::Counter;
use cursive::view::Boxable;
use cursive::views::*;
use cursive::*;
use cursive::{
    align::{HAlign, VAlign},
    theme::Theme,
};
use cursive::{Cursive, CursiveExt};
use theme::BaseColor;
// STD Dependencies -----------------------------------------------------------
use super::{
    cursive_table_view::{ExplorerReady, TableView, TableViewItem},
    file_manager_config, literals,
};
use chrono::offset::Utc;
use chrono::DateTime;
use std::{borrow::BorrowMut, collections::HashMap, io::Write, path::MAIN_SEPARATOR};
use std::{fs::File, fs::OpenOptions, io::Read, path::PathBuf};

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
use crate::internals::file_manager::GLOBAL_FileManager;
use crate::internals::literals::copy_progress_dlg;

use crate::internals::file_manager_config::FileMangerConfig;
use crate::internals::literals::main_ui;
use crate::internals::ops::f2_menu::menu;
use crate::internals::ops::f3_preview::preview;
use crate::internals::ops::f4_open::open_externally;
use crate::internals::ops::f5_cpy::cpy;
use crate::internals::ops::f6_ren_mv::ren_mv;
use crate::internals::ops::f8_del::del;
use crate::internals::ops::ops_utils::archive_readers;
use crate::internals::ops::ops_utils::archive_types;
// ----------------------------------------------------------------------------
//use std::cmp::Ordering;
// External Dependencies ------------------------------------------------------
// ----------------------------------------------------------------------------

use fs_extra::dir::{copy, TransitProcessResult};
use notify::{watcher, INotifyWatcher, RecursiveMode, Watcher};
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
                // ... and of sub-trees, which preview up when selected.
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
    siv.add_global_callback(Key::Esc, close_dlgs);
    siv.add_global_callback(Key::F2, menu);
    siv.add_global_callback(Key::F3, preview); //todo repeat
    siv.add_global_callback(Key::F4, open_externally);
    siv.add_global_callback(Key::F5, cpy);
    siv.add_global_callback(Key::F6, ren_mv);
    siv.add_global_callback(Key::F8, del);
    siv.add_global_callback(Key::F10, quit);
}
fn close_dlgs(siv: &mut cursive::Cursive) {
    while siv.screen_mut().len() > 1 {
        siv.pop_layer();
    }
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

impl ExplorerReady for ExplorerColumnData {
    fn has_parent(&self) -> bool {
        self.name == ".." //todo static &str
    }
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
fn init_new_path_machinery(
    siv: &mut Cursive,
    a_name: &str,
    new_path: PathBuf,
    watcher: Arc<Mutex<INotifyWatcher>>,
    archive_reader: Option<Box<dyn ArchiveReader>>,
) {
    let current_dir = get_current_dir(siv, get_panel_id_from_table_id(a_name));
    let path_to_stop_watching = current_dir.clone();
    match watcher.lock().unwrap().unwatch(path_to_stop_watching) {
        Ok(_) => {
            //println!("Unwatched");
        }
        Err(err) => {
            println!("Cannot unwatch: {}", err);
            panic!(); //todo remove
        }
    }
    watcher
        .lock()
        .unwrap()
        .watch(new_path.clone(), RecursiveMode::NonRecursive)
        .unwrap();
    match archive_reader {
        Some(archive_reader) => {
            fill_table_with_archive_content_wrapper(siv, a_name, archive_reader, new_path);
        }
        None => fill_table_with_items_wrapper(siv, a_name, new_path),
    }
}
pub type tableViewType = TableView<ExplorerColumnData, ExplorerColumn>;
pub fn create_basic_table_core(
    siv: &mut Cursive,
    a_name: &'static str,
    initial_path: &str,
) -> NamedView<tableViewType> {
    let mut table = tableViewType::new()
        .column(ExplorerColumn::Name, "Name", |c| c.width_percent(60))
        .column(ExplorerColumn::Size, "Size", |c| {
            c.align(cursive::align::HAlign::Center)
        })
        .column(ExplorerColumn::LastModifyTime, "LastModifyTime", |c| {
            c.ordering(std::cmp::Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
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
    watcher
        .watch(initial_path.clone(), RecursiveMode::NonRecursive)
        .unwrap();

    start_dir_watcher_thread(siv, a_name, rx);
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
                a_table.borrow_item(index).unwrap().clone()
            })
            .unwrap();
        let _value = siv
            .call_on_name(get_info_item_from_table_id(a_name), move |a_dlg: &mut TextView| {
                a_dlg.set_content(current_item.name.clone());
            })
            .unwrap();
    });
    table.set_selected_row(0);
    table.set_on_submit(move |siv: &mut Cursive, row: usize, index: usize| {
        siv.call_on_name(a_name, |a_table: &mut tableViewType| {
            a_table.clear_selected_items();
        });

        let new_path = get_selected_path_only_from_inx(siv, a_name, index);
        match new_path {
            Some(new_path) => {
                let new_path = PathBuf::from(new_path);
                if new_path.is_dir() {
                    init_new_path_machinery(siv, a_name, new_path, watcher.clone(), None);
                } else {
                    let result = tree_magic_mini::from_filepath(new_path.as_path());
                    match result {
                        Some(potential_archive) => {
                            if let Some(archive_type) = archive_types::ARCHIVES_TYPES.get_key(potential_archive) {
                                //println!("archve detected:{}", archive_type);
                                /*check if we can open it and if so open */
                                match archive_readers::ARCHIVES_READERS_TYPES.get(archive_type) {
                                    Some(reader_type) => {
                                        let reader = archive_readers::ArchiveReaderFactory::create_reader(reader_type);
                                        init_new_path_machinery(siv, a_name, new_path, watcher.clone(), Some(reader));
                                        //fill_table_with_archive_content_wrapper(siv, a_name, reader.read(&new_path));
                                    }
                                    None => {
                                        println!("reader NOT found");
                                    }
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {
                if index == 0
                //we must be on ".." or top level
                {
                    let current_dir = PathBuf::from(get_current_dir(siv, get_panel_id_from_table_id(a_name)));
                    match current_dir.parent() {
                        Some(parent_path) => {
                            init_new_path_machinery(siv, a_name, parent_path.into(), watcher.clone(), None);
                        }
                        None => {}
                    }
                }
            }
        }
    });
    let named_view_table = table.with_name(a_name);

    named_view_table
}
pub type TableNameT = String;
pub type PathT = String;
pub type IndexT = usize;
pub type PathInfoT = Vec<(TableNameT, PathT, IndexT)>;
pub fn get_selected_paths(siv: &mut Cursive, a_name: &str) -> Option<PathInfoT> {
    let mut selected_items_inx = std::collections::BTreeSet::<usize>::new();
    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
        selected_items_inx = a_table.get_selected_items();
    });

    if selected_items_inx.len() != 0 {
        let mut selected_paths = PathInfoT::new();
        for selected_inx in selected_items_inx {
            match get_selected_path_from_inx(siv, a_name, selected_inx) {
                Some(path) => {
                    selected_paths.push(path);
                }
                None => {}
            }
        }
        if selected_paths.len() != 0 {
            Some(selected_paths)
        } else {
            None
        }
    } else {
        None
    }
}
pub fn get_selected_paths_only(siv: &mut Cursive, a_name: &str) -> Option<Vec<PathT>> {
    let mut selected_items_inx = std::collections::BTreeSet::<usize>::new();
    siv.call_on_name(a_name, |a_table: &mut tableViewType| {
        selected_items_inx = a_table.get_selected_items();
    });

    if selected_items_inx.len() != 0 {
        let mut selected_paths = Vec::new();
        for selected_inx in selected_items_inx {
            match get_selected_path_only_from_inx(siv, a_name, selected_inx) {
                Some(path) => {
                    selected_paths.push(path);
                }
                None => {}
            }
        }
        if selected_paths.len() != 0 {
            Some(selected_paths)
        } else {
            None
        }
    } else {
        None
    }
}
pub struct PanelId<'a> {
    panel_id: &'a str,
}
impl<'a> Into<&'a str> for PanelId<'a> {
    fn into(self) -> &'a str {
        self.panel_id
    }
}

pub fn get_current_dir(siv: &mut Cursive, a_panel_id: PanelId) -> String {
    let current_dir = siv
        .call_on_name(a_panel_id.into(), move |a_dlg: &mut Atomic_Dialog| a_dlg.get_title())
        .unwrap();
    current_dir
}
pub fn get_panel_id_from_table_id(table_id: &str) -> PanelId {
    if table_id == literals::main_ui::widget_names::LEFT_PANEL_TABLE_ID {
        PanelId {
            panel_id: literals::main_ui::widget_names::LEFT_PANEL_ID,
        }
    } else if table_id == literals::main_ui::widget_names::RIGHT_PANEL_TABLE_ID {
        PanelId {
            panel_id: literals::main_ui::widget_names::RIGHT_PANEL_ID,
        }
    } else {
        panic!("Wrong table id provided");
    }
}

fn get_info_item_from_table_id(table_id: &str) -> &str {
    if table_id == literals::main_ui::widget_names::LEFT_PANEL_TABLE_ID {
        literals::main_ui::widget_names::LEFT_PANEL_INFO_ITEM_ID
    } else if table_id == literals::main_ui::widget_names::RIGHT_PANEL_TABLE_ID {
        literals::main_ui::widget_names::RIGHT_PANEL_INFO_ITEM_ID
    } else {
        panic!("Wrong table id provided");
    }
}

fn get_selected_path_from_inx(siv: &mut Cursive, a_name: &str, index: usize) -> Option<(TableNameT, PathT, IndexT)> {
    /*Todo repeat*/
    let current_dir = get_current_dir(siv, get_panel_id_from_table_id(a_name));
    let new_path = siv
        .call_on_name(a_name, move |a_table: &mut tableViewType| {
            let mut selected_item = a_table.borrow_item(index).unwrap().name.clone();
            let whole_path = match selected_item.as_str() {
                ".." => None,
                _ => {
                    let s = selected_item.chars().nth(0).unwrap();
                    if s != std::path::MAIN_SEPARATOR {
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

fn get_selected_path_only_from_inx(siv: &mut Cursive, a_name: &str, index: usize) -> Option<PathT> {
    /*Todo repeat*/
    let current_dir = get_current_dir(siv, get_panel_id_from_table_id(a_name));
    let new_path = siv
        .call_on_name(a_name, move |a_table: &mut tableViewType| {
            let mut selected_item = a_table.borrow_item(index).unwrap().name.clone();
            let whole_path = match selected_item.as_str() {
                ".." => None,
                _ => {
                    let s = selected_item.chars().nth(0).unwrap();
                    if s != std::path::MAIN_SEPARATOR {
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
fn set_panel_title(siv: &mut Cursive, a_name: &str, new_path: PathBuf) {
    siv.call_on_name(
        get_panel_id_from_table_id(a_name).into(),
        |a_dlg: &mut Atomic_Dialog| {
            a_dlg.set_title(new_path.clone().to_str().unwrap());
        },
    )
    .unwrap()
}
fn fill_table_with_items_wrapper(siv: &mut Cursive, a_name: &str /*todo &str */, new_path: PathBuf) {
    let mut res = Option::<std::io::Error>::default();
    siv.call_on_name(&a_name, |a_table: &mut tableViewType| {
        res = fill_table_with_items(a_table, new_path.clone()).err();
    });
    match res {
        Some(e) => {
            siv.add_layer(Dialog::around(TextView::new(e.to_string())).dismiss_button("Ok"));
        }
        None => {
            set_panel_title(siv, a_name, new_path);
        }
    }
}

fn update_table(siv: &mut Cursive, a_name: &str, a_path: PathBuf) {
    fill_table_with_items_wrapper(siv, a_name, a_path);
}

pub fn remove_view(siv: &mut Cursive, view_name: &str) {
    match siv.screen_mut().find_layer_from_name(view_name) {
        Some(layer_position) => {
            siv.screen_mut().remove_layer(layer_position);
        }
        None => {}
    }
}

pub fn unselect_inx(siv: &mut Cursive, a_table_name: Arc<String>, inx: Arc<usize>) {
    siv.call_on_name(a_table_name.as_str(), |a_table: &mut tableViewType| {
        a_table.clear_selected_item(*inx);
    });
}

fn help(siv: &mut cursive::Cursive) {}

pub type ProgressDlgT = ResizedView<Dialog>;
pub fn create_themed_view<T>(siv: &mut Cursive, view: T) -> ThemedView<Layer<T>>
where
    T: View,
{
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
    });
    views::ThemedView::new(theme, Layer::new(view))
}

fn mkdir(siv: &mut cursive::Cursive) {}

fn pull_dn(siv: &mut cursive::Cursive) {}

fn quit(siv: &mut cursive::Cursive) {
    /*Todo move to separate mod */
    let left_dir = get_current_dir(
        siv,
        PanelId {
            panel_id: literals::main_ui::widget_names::LEFT_PANEL_ID,
        },
    );
    let right_dir = get_current_dir(
        siv,
        PanelId {
            panel_id: literals::main_ui::widget_names::RIGHT_PANEL_ID,
        },
    );

    let mutex_guard = GLOBAL_FileManager.get().lock().unwrap();
    let mut ref_mut = (*mutex_guard).borrow_mut();

    let fm = &mut ref_mut.config;
    fm.left_panel_initial_path = left_dir;
    fm.right_panel_initial_path = right_dir;
    file_manager_config::write_config(&fm);

    siv.quit();
}

fn start_dir_watcher_thread(siv: &mut Cursive, a_table_name: &'static str, rx: Receiver<notify::DebouncedEvent>) {
    let cb_panel_update_clone = siv.cb_sink().clone();
    //let cb_panel_update_clone = cb_panel_update.clone();

    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(event) => {
                    //let path = a_path.clone(); //todo optimize
                    let path: Option<PathBuf> = match event {
                        notify::DebouncedEvent::Rescan => None,
                        notify::DebouncedEvent::Error(_, _) => None,

                        notify::DebouncedEvent::NoticeWrite(a_path) => {
                            let mut dir = a_path.clone();
                            dir.pop();
                            Some(dir)
                        }
                        notify::DebouncedEvent::NoticeRemove(a_path) => {
                            let mut dir = a_path.clone();
                            dir.pop();
                            Some(dir)
                        }
                        notify::DebouncedEvent::Create(a_path) => {
                            let mut dir = a_path.clone();
                            dir.pop();
                            Some(dir)
                        }
                        notify::DebouncedEvent::Write(a_path) => {
                            let mut dir = a_path.clone();
                            dir.pop();
                            Some(dir)
                        }
                        notify::DebouncedEvent::Chmod(a_path) => {
                            let mut dir = a_path.clone();
                            dir.pop();
                            Some(dir)
                        }
                        notify::DebouncedEvent::Remove(a_path) => {
                            let mut dir = a_path.clone();
                            dir.pop();
                            Some(dir)
                        }
                        notify::DebouncedEvent::Rename(a_path_from, a_path_to) => {
                            let mut dir = a_path_to.clone();
                            dir.pop();
                            Some(dir)
                        } //todo check if to
                    };
                    if path.is_some() {
                        cb_panel_update_clone
                            .send(Box::new(move |siv| update_table(siv, a_table_name, path.unwrap())))
                            .unwrap();
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}

pub fn create_main_layout(siv: &mut cursive::CursiveRunnable, fm_config: &FileMangerConfig) {
    let left_table = create_basic_table_core(
        siv,
        main_ui::widget_names::LEFT_PANEL_TABLE_ID,
        &fm_config.left_panel_initial_path,
    );
    let left_info_item =
        TextView::new("Hello Dialog!").with_name(literals::main_ui::widget_names::LEFT_PANEL_INFO_ITEM_ID);
    let main_left_layout = LinearLayout::vertical()
        .child(left_table.full_screen())
        .child(Delimiter::new("Title 1"))
        .child(left_info_item);
    let mut left_stack_view = StackView::new();
    //left_stack_view.add_fullscreen_layer(main_left_layout);
    let left_layout = Atomic_Dialog::around(/*left_stack_view*/ main_left_layout)
        .title(fm_config.left_panel_initial_path.clone())
        .padding_lrtb(0, 0, 0, 0)
        .with_name(main_ui::widget_names::LEFT_PANEL_ID);

    let right_table = create_basic_table_core(
        siv,
        main_ui::widget_names::RIGHT_PANEL_TABLE_ID,
        &fm_config.right_panel_initial_path,
    );
    let right_info_item =
        TextView::new("Hello Dialog!").with_name(literals::main_ui::widget_names::RIGHT_PANEL_INFO_ITEM_ID);
    let main_right_layout = LinearLayout::vertical()
        .child(right_table.full_screen())
        .child(Delimiter::new("Title 2"))
        .child(right_info_item);
    let mut right_stack_view = StackView::new();
    //right_stack_view.add_fullscreen_layer(main_right_layout);
    let right_layout = Atomic_Dialog::around(/*right_stack_view*/ main_right_layout)
        .title(fm_config.right_panel_initial_path.clone()) //todo get name from table
        .padding_lrtb(0, 0, 0, 0)
        .with_name(main_ui::widget_names::RIGHT_PANEL_ID);

    let button_help = OnEventView::new(TextView::new("[ Help ]"))
        .on_event('w', |siv| siv.quit())
        .on_event(event::Key::Tab, |siv| siv.quit());
    //    button_help.disable();
    //button_help.align
    let help_layout = LinearLayout::horizontal().child(TextView::new("1")).child(button_help);
    let button_menu = Button::new_raw("[ Menu ]", menu);
    let menu_layout = LinearLayout::horizontal().child(TextView::new("2")).child(button_menu);
    let button_view = Button::new_raw("[ Preview ]", preview);
    let view_layout = LinearLayout::horizontal()
        .child(TextView::new("3").style(theme::ColorStyle::title_primary()))
        .child(button_view);
    let button_edit = Button::new_raw("[ Open ]", open_externally);
    let edit_layout = LinearLayout::horizontal()
        .child(TextView::new("4").style(theme::ColorStyle::title_primary()))
        .child(button_edit);
    let mouse_event = event::Event::Mouse {
        offset: XY::new(0, 0),
        position: XY::new(1, 1),
        event: MouseEvent::Press(MouseButton::Left),
    };
    let ProgressBar_on_event_view = HideableView::new(
        //todo must it be on event?
        ProgressBar::new(),
    )
    .visible(false)
    .with_name(copy_progress_dlg::widget_names::hideable_cpy_prgrs_br);
    let left_bracket_hideable = HideableView::new(TextView::new("["))
        .visible(false)
        .with_name(copy_progress_dlg::widget_names::hideable_cpy_prgrs_br_left_bracket);
    let right_bracket_hideable = HideableView::new(TextView::new("]"))
        .visible(false)
        .with_name(copy_progress_dlg::widget_names::hideable_cpy_prgrs_br_right_bracket);
    let button_cpy = HideableView::new(Button::new_raw("[ Copy ]", cpy))
        .visible(true)
        .with_name(copy_progress_dlg::widget_names::hideable_cpy_button);
    let cpy_layout = LinearLayout::horizontal()
        .child(TextView::new("5").style(theme::ColorStyle::title_primary()))
        .child(button_cpy)
        .child(left_bracket_hideable)
        .child(ProgressBar_on_event_view)
        .child(right_bracket_hideable);
    let button_RenMov = Button::new_raw("[ RenMv ]", ren_mv);
    let ren_mov_layout = LinearLayout::horizontal()
        .child(TextView::new("6").style(theme::ColorStyle::title_primary()))
        .child(button_RenMov);
    let button_MkDir = Button::new_raw("[ MkDir ]", mkdir);
    let mkdir_layout = LinearLayout::horizontal().child(TextView::new("7")).child(button_MkDir);
    let button_Del = Button::new_raw("[ Del ]", del);
    let del_layout = LinearLayout::horizontal()
        .child(TextView::new("8").style(theme::ColorStyle::title_primary()))
        .child(button_Del);
    let button_PullDn = Button::new_raw("[ PullDn ]", pull_dn);
    let pulldn_layout = LinearLayout::horizontal()
        .child(TextView::new("9"))
        .child(button_PullDn);
    let button_quit = Button::new_raw("[ Quit ]", quit);
    let quit_layout = LinearLayout::horizontal()
        .child(TextView::new("10").style(theme::ColorStyle::title_primary()))
        .child(button_quit);
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
    let left_right_layout = CircularFocus::new(
        LinearLayout::horizontal().child(left_layout).child(right_layout),
        true,
        true,
    );
    let whole_layout = LinearLayout::vertical().child(left_right_layout).child(buttons_layout);
    siv.add_fullscreen_layer(whole_layout);
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
fn fill_table_with_archive_content_wrapper(
    siv: &mut Cursive,
    a_name: &str, /*todo &str */
    archive_reader: Box<dyn ArchiveReader>,
    new_path: PathBuf,
) {
    let zipped_content = archive_reader.read(&new_path);
    siv.call_on_name(&a_name, |a_table: &mut tableViewType| {
        fill_table_with_archive_content(a_table, zipped_content);
    });
    set_panel_title(siv, a_name, new_path);
}
fn fill_table_with_archive_content(a_table: &mut tableViewType, zipped_content: Vec<String>) {
    let mut items = Vec::new();

    items.push(ExplorerColumnData {
        name: format!(".."),
        size: 0,
        last_modify_time: SystemTime::now(),
    });

    for a_path_buf in zipped_content {
        items.push(ExplorerColumnData {
            name: format!("{}", a_path_buf),
            size: 0,
            last_modify_time: SystemTime::now(), /*todo */
        });
    }
    let _ = a_table.take_items(); //clear before you put new, panic! otherwise will occur
    a_table.set_items(items);
}

pub fn get_active_panel(siv: &mut Cursive) -> String {
    let left_panel_last_focus_time = siv
        .call_on_name(
            main_ui::widget_names::LEFT_PANEL_TABLE_ID,
            |a_table: &mut tableViewType| a_table.last_focus_time,
        )
        .unwrap();

    let right_panel_last_focus_time = siv
        .call_on_name(
            main_ui::widget_names::RIGHT_PANEL_TABLE_ID,
            |a_table: &mut tableViewType| a_table.last_focus_time,
        )
        .unwrap();

    let active_panel = if left_panel_last_focus_time > right_panel_last_focus_time {
        main_ui::widget_names::LEFT_PANEL_TABLE_ID
    } else {
        main_ui::widget_names::RIGHT_PANEL_TABLE_ID
    };

    active_panel.to_owned()
}

pub fn get_error_theme(siv: &mut Cursive) -> cursive::theme::Theme {
    siv.current_theme().clone().with(|theme| {
        theme.palette[theme::PaletteColor::View] = theme::Color::Dark(theme::BaseColor::Red);
        theme.palette[theme::PaletteColor::Primary] = theme::Color::Light(theme::BaseColor::White);
        theme.palette[theme::PaletteColor::TitlePrimary] = theme::Color::Light(theme::BaseColor::Yellow);
        theme.palette[theme::PaletteColor::Highlight] = theme::Color::Dark(theme::BaseColor::Black);
    })
}

pub fn get_file_content(input_file: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(input_file)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    Ok(buf)
}
