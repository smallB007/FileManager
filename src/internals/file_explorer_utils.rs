#![forbid(unreachable_patterns)]
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
    literals,
};
use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::path::PathBuf;

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

use crate::internals::file_manager::GLOBAL_FileManager;
use crate::internals::file_manager_config::FileMangerConfig;
use crate::internals::literals::main_ui;
use crate::internals::ops::f5_cpy::{
    copying_already_exists, cpy, AtomicFileTransitFlags, CpyData, FileExistsAction, FileExistsActionWithOptions,
    OverrideCase,
};
use crate::internals::ops::f6_ren_mv::ren_mv;
use crate::internals::ops::f8_del::del;
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
    siv.add_global_callback(Key::F5, cpy);
    siv.add_global_callback(Key::F8, del);
    siv.add_global_callback(Key::F4, ren_mv);
    //    siv.add_global_callback(Key::F7, show_hide_cpy);
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
            .call_on_name(
                &(String::from(a_name) + &String::from("InfoItem")),
                move |a_dlg: &mut TextView| {
                    //                a_dlg.set_title(current_item.name.clone());
                    a_dlg.set_content(current_item.name.clone());
                },
            )
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

        let current_dir = get_current_dir(siv, a_name);
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
            watcher
                .lock()
                .unwrap()
                .watch(new_path.clone(), RecursiveMode::NonRecursive)
                .unwrap();
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

pub fn get_current_dir(siv: &mut Cursive, a_panel_id: &str) -> String {
    let current_dir = siv
        .call_on_name(a_panel_id, move |a_dlg: &mut Atomic_Dialog| a_dlg.get_title())
        .unwrap();
    current_dir
}
fn get_panel_id_from_table_id(table_id: &str) -> &str {
    if table_id == literals::main_ui::widget_names::LEFT_PANEL_TABLE_ID {
        literals::main_ui::widget_names::LEFT_PANEL_ID
    } else if table_id == literals::main_ui::widget_names::RIGHT_PANEL_TABLE_ID {
        literals::main_ui::widget_names::RIGHT_PANEL_ID
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
                .call_on_name(
                    &(String::from(a_name) + &String::from("Dlg")),
                    |a_dlg: &mut Atomic_Dialog| {
                        a_dlg.set_title(new_path.clone().to_str().unwrap());
                    },
                )
                .unwrap();
        }
    }
}

fn update_table(siv: &mut Cursive, a_name: String, a_path: String) {
    let new_path = PathBuf::from(a_path);
    fill_table_with_items_wrapper(siv, a_name, new_path);
    //println!("Command received");
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
/*fn show_hide_cpy(siv: &mut cursive::Cursive) {
    if let Some(_) = siv.find_name::<ProgressDlgT>("Pcopy_progress_dlg::labels::dialog_namerogressDlg") {
        siv.pop_layer(); //trouble
    } else {
        let g_file_manager = GLOBAL_FileManager.get();
        match &g_file_manager.lock().unwrap().borrow().cpy_data {
            Some(cpy_data) => {
                let cpy_progress_dlg = create_cpy_progress_dialog(cpy_data.files_total, cpy_data.cond_var_suspend.clone());
                siv.add_layer(cpy_progress_dlg);
                siv.set_autorefresh(true);
            }
            None => {}
        }
    }
}*/
pub fn create_themed_view<T>(siv: &mut Cursive, view: T) -> ThemedView<Layer<T>>
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

fn mkdir(siv: &mut cursive::Cursive) {}

fn pull_dn(siv: &mut cursive::Cursive) {}
fn quit(siv: &mut cursive::Cursive) {
    /* */
    siv.quit();
}
fn start_dir_watcher_thread(
    siv: &mut Cursive,
    a_table_name: String,
    a_path: String,
    rx: Receiver<notify::DebouncedEvent>,
) {
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
                    cb_panel_update_clone
                        .send(Box::new(|siv| update_table(siv, name, path)))
                        .unwrap();
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
    let left_info_item = TextView::new("Hello Dialog!").with_name("LeftPanelInfoItem");
    let left_layout = Atomic_Dialog::around(
        LinearLayout::vertical()
            .child(left_table.full_screen())
            .child(Delimiter::new("Title 1"))
            .child(left_info_item),
    )
    .title(fm_config.left_panel_initial_path.clone())
    .padding_lrtb(0, 0, 0, 0)
    .with_name(main_ui::widget_names::LEFT_PANEL_ID);

    let right_table = create_basic_table_core(
        siv,
        main_ui::widget_names::RIGHT_PANEL_TABLE_ID,
        &fm_config.right_panel_initial_path,
    );
    let right_info_item = TextView::new("Hello Dialog!").with_name("RightPanelInfoItem");
    let right_layout = Atomic_Dialog::around(
        LinearLayout::vertical()
            .child(right_table.full_screen())
            .child(Delimiter::new("Title 2"))
            .child(right_info_item),
    )
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
    let button_view = Button::new_raw("[ View ]", view);
    let view_layout = LinearLayout::horizontal().child(TextView::new("3")).child(button_view);
    let button_edit = Button::new_raw("[ Edit ]", edit);
    let edit_layout = LinearLayout::horizontal().child(TextView::new("4")).child(button_edit);
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
        .child(TextView::new("6"))
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
    let left_right_layout = CircularFocus::new(
        LinearLayout::horizontal().child(left_layout).child(right_layout),
        true,
        true,
    );
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
