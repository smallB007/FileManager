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
    reexports::log::warn,
};
use cursive::{Cursive, CursiveExt};
//std
use chrono::offset::Utc;
use chrono::DateTime;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::{fmt::Display, path::PathBuf};
use std::{
    io::{Error, ErrorKind},
    os::unix::prelude::MetadataExt,
};
//fs_extra
use fs_extra::dir::{copy, TransitProcessResult};
//FileManager crate
use crate::internals::file_explorer_utils::{
    create_themed_view, get_active_panel, get_current_dir, get_error_theme, get_panel_id_from_table_id,
    get_selected_paths, remove_view, tableViewType, unselect_inx, PathInfoT, ProgressDlgT,
};
use crate::internals::file_manager::GLOBAL_FileManager;
use crate::internals::literals::copy_dlg;
use crate::internals::literals::copy_progress_dlg;
use crate::internals::literals::file_exists_dlg;
use crate::internals::literals::main_ui;
use crate::internals::{self, atomic_dialog::Atomic_Dialog, literals};

#[derive(Copy, Clone)]
pub enum AtomicFileTransitFlags {
    Abort,
    Skip,
}
#[derive(Clone)]
pub struct CpyData {
    pub cond_var_suspend: Arc<(Mutex<bool>, Condvar)>,
    //    cond_var_skip: Arc<(Mutex<bool>, Condvar)>,
    pub files_total: usize,
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
fn assign_action_and_notify(
    cond_var: &Arc<(
        /*lock flag*/ Mutex<bool>,
        Condvar,
        Mutex<FileExistsActionWithOptions>,
    )>,
    action: FileExistsAction,
    apply_to_all: bool,
    dont_overwrite_with_zero: bool,
) {
    let (lock_flag, cond_var, file_action) = &**cond_var; //todo pretty certain, no mutex is needed here
    *lock_flag.lock().unwrap() = false;
    let action_with_options = &mut *file_action.lock().unwrap();
    action_with_options.action = action;
    action_with_options.apply_to_all = apply_to_all;
    action_with_options.dont_overwrite_with_zero = dont_overwrite_with_zero;
    cond_var.notify_all();
}
#[derive(Copy, Clone)]
pub enum OverrideCase {
    JustDoIt,
    Older,
    Newer,
    Larger,
    Smaller,
    DifferentSize,
    Append,
}
#[derive(Copy, Clone)]
pub enum FileExistsAction {
    Abort,
    Override(OverrideCase),
    Skip,
}
pub struct FileExistsActionWithOptions {
    pub action: FileExistsAction,
    pub apply_to_all: bool,
    pub dont_overwrite_with_zero: bool,
}
macro_rules! with_clones {
    ($($capture:ident),+; $e:expr) => {
        {
        $(
            let $capture = $capture.clone();
        )+
            $e
        }
    }
}
fn clone_and_notify(
    arc: &Arc<(
        /*lock flag*/ Mutex<bool>,
        Condvar,
        Mutex<FileExistsActionWithOptions>,
    )>,
    action: FileExistsAction,
) -> impl Fn(&mut Cursive) {
    let cloned = arc.clone();
    move |s| {
        let is_apply_to_all = s
            .call_on_name(
                file_exists_dlg::widget_names::apply_to_all_chckbx,
                |a_checkbox: &mut Checkbox| a_checkbox.is_checked(),
            )
            .unwrap();
        let is_dont_overwrite_with_zero = s
            .call_on_name(
                file_exists_dlg::widget_names::dont_overwrite_with_zero_chckbx,
                |a_checkbox: &mut Checkbox| a_checkbox.is_checked(),
            )
            .unwrap();
        s.pop_layer();
        assign_action_and_notify(&cloned, action, is_apply_to_all, is_dont_overwrite_with_zero)
    }
}
pub fn copying_already_exists(
    siv: &mut Cursive,
    path_from: PathBuf,
    path_to: PathBuf,
    is_overwrite: bool,
    is_recursive: bool,
    cond_var_skip: Arc<(Mutex<bool>, Condvar, Mutex<FileExistsActionWithOptions>)>,
) {
    let theme_error = get_error_theme(siv);
    siv.set_autorefresh(false); //todo repeat
                                /*todo!("Dialog type changed");
                                if let Some(_) = siv.find_name::<ProgressDlgT>(copy_progress_dlg::labels::dialog_name) {
                                    siv.pop_layer();
                                }*/
    let name_from = path_from.to_str().unwrap();
    let size_from = path_from.metadata().unwrap().size();
    let date_from = {
        let datetime: DateTime<Utc> = path_from.metadata().unwrap().modified().unwrap().into();
        format!("{}", datetime.format("%d/%m/%Y %T"))
    };
    let name_to = path_to.to_str().unwrap(); //todo repeat
    let size_to = path_to.metadata().unwrap().size();
    let date_to = {
        let datetime: DateTime<Utc> = path_to.metadata().unwrap().modified().unwrap().into();
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
        //todo refactor
        LinearLayout::vertical()
            .child(new_from_layout)
            .child(DummyView)
            .child(new_to_layout)
            .child(DummyView)
            //.child(Delimiter::default())
            .child(DummyView)
            .child(
                LinearLayout::vertical()
                    .child(
                        LinearLayout::horizontal()
                            .child(Checkbox::new().with_name(file_exists_dlg::widget_names::apply_to_all_chckbx))
                            .child(TextView::new(" Apply to all")),
                    )
                    .child(
                        LinearLayout::horizontal()
                            .child(
                                Checkbox::new()
                                    .with_name(file_exists_dlg::widget_names::dont_overwrite_with_zero_chckbx),
                            )
                            .child(TextView::new(" Don't overwrite with zero length file")),
                    ),
            )
            .child(DummyView),
    )
    .title("File Exists")
    .button(
        "Overwrite",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::JustDoIt)),
    )
    .button(
        "Older",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::Older)),
    )
    .button(
        "Newer",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::Newer)),
    )
    .button(
        "Larger",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::Larger)),
    )
    .button(
        "Smaller",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::Smaller)),
    )
    .button(
        "Different size",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::DifferentSize)),
    )
    .button(
        "Append",
        clone_and_notify(&cond_var_skip, FileExistsAction::Override(OverrideCase::Append)),
    )
    .button("Skip", clone_and_notify(&cond_var_skip, FileExistsAction::Skip))
    .button("Abort", clone_and_notify(&cond_var_skip, FileExistsAction::Abort));
    /*let buttons = LinearLayout::horizontal().child(Button::new("A bu",|s|{}));
    let dlg_and_buttons = LinearLayout::vertical().child(file_exist_dlg).child(buttons);*/
    siv.add_layer(views::ThemedView::new(theme_error, Layer::new(file_exist_dlg)));
}

fn cannot_suspend_copy(siv: &mut Cursive) {
    siv.set_autorefresh(false);
    if let Some(_) = siv.find_name::<ResizedView<Dialog>>(copy_progress_dlg::widget_names::dialog_name) {
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
    g_file_manager.lock().unwrap().borrow_mut().clear(); //todo think of safe clear for everything that needs to be

    show_progress_cpy(siv, 0, false);

    siv.set_autorefresh(false);

    remove_view(siv, copy_progress_dlg::widget_names::dialog_name);

    let success_dlg = Dialog::new()
        .title(title)
        .content(TextView::new(text).center())
        .dismiss_button("OK");
    let success_dlg = create_themed_view(siv, success_dlg);
    siv.add_layer(success_dlg);
}
fn copying_finished_success(siv: &mut Cursive) {
    end_copying_helper(siv, "Copying finished", "Copying finished successfully");
}
fn copying_cancelled(siv: &mut Cursive) {
    end_copying_helper(siv, "User request cancel", "Copying cancelled");
}

fn update_cpy_dlg(
    siv: &mut Cursive,
    process_info: fs_extra::TransitProcess,
    file_name: String,
    current_inx: usize,
    is_copy: bool,
) {
    /*   siv.call_on_name("TextView_copying_x_of_n", |a_text_view: &mut TextView| {
        a_text_view.set_content(format!("Copying {} of {}", process_info.copied_bytes, process_info.total_bytes));
    })
    .unwrap();*/
    siv.call_on_name(
        copy_progress_dlg::widget_names::progress_bar_total,
        |a_progress_bar: &mut ProgressBar| {
            a_progress_bar.set_value(current_inx);
            a_progress_bar.set_label(|val, (_min, max)| format!("Copied {} of {}", val, max));
        },
    );
    siv.call_on_name(
        "hideable_cpy_prgrs_br",
        |hideable_view_total: &mut HideableView<ResizedView<ProgressBar>>| {
            hideable_view_total
                .get_inner_mut()
                .get_inner_mut()
                .set_value(current_inx);
        },
    );
    siv.call_on_name("TextView_copying_x", |a_text_view: &mut TextView| {
        a_text_view.set_content(literals::copy_progress_dlg::labels::get_copying_n(is_copy, &file_name));
    });
    siv.call_on_name(
        copy_progress_dlg::widget_names::progress_bar_current,
        |a_progress_bar: &mut ProgressBar| {
            let current_path_percent =
                ((process_info.copied_bytes as f64 / process_info.total_bytes as f64) * 100_f64) as usize;
            a_progress_bar.set_value(current_path_percent);
        },
    );
}
#[cfg(ETA)]
struct file_transfer_context {
    eta_secs: u64,
    bps: u64,
    bps_time: u64,
}
/* let start = std::time::Instant::now();
let duration = start.elapsed();
println!("Copying finished:{}", duration.as_secs());*/
pub fn move_rename_items_with_progress<F>(
    selected_mask: &String, //++artie
    from_items: &[String],
    to: &String,
    options: &fs_extra::dir::CopyOptions,
    mut progress_handler: F,
) -> fs_extra::error::Result<u64>
where
    F: FnMut(fs_extra::TransitProcess) -> fs_extra::dir::TransitProcessResult,
{
    let rg = regex::RegexSet::new(&selected_mask.split_ascii_whitespace().collect::<Vec<_>>()); //++artie
    let rg_ok = rg.is_ok();

    for item_from in from_items {
        if rg_ok && !rg.as_ref().unwrap().is_match(&item_from) {
            continue;
        }

        let current_path_name = PathBuf::from(&item_from).file_name().unwrap().to_owned();
        let full_path_to = to.clone() + &std::path::MAIN_SEPARATOR.to_string() + current_path_name.to_str().unwrap();
        match std::fs::rename(item_from, full_path_to) {
            Ok(val) => {}
            Err(err) => {
                warn!("Cannot rename, reason: {}", err)
            }
        }
    }
    Ok(1)
}

fn cpy_or_mv<Cb>(
    cp: bool,
    selected_mask: &String,
    current_path: String,
    path_to: String,
    options: &fs_extra::dir::CopyOptions,
    progress_handler_path: Cb,
) -> std::result::Result<u64, fs_extra::error::Error>
where
    Cb: Fn(fs_extra::TransitProcess) -> TransitProcessResult,
{
    if cp {
        fs_extra::copy_items_with_progress(
            &selected_mask,
            &vec![current_path],
            &path_to,
            &options,
            progress_handler_path,
        )
    } else {
        move_rename_items_with_progress(
            &selected_mask,
            &vec![current_path],
            &path_to,
            &options,
            progress_handler_path,
        )
    }
}
fn cpy_task(
    selected_mask: String,
    selected_paths: PathInfoT,
    path_to: String,
    cb: CbSink,
    cond_var_suspend: Arc<(Mutex<bool>, Condvar)>,
    cond_var_skip: Arc<(Mutex<bool>, Condvar, Mutex<FileExistsActionWithOptions>)>, //todo not sure, most likely on demand only
    is_recursive: bool,
    is_overwrite: bool,
    is_append: bool,
    is_copy: bool,
    is_rename: bool,
) {
    if is_rename {
        match std::fs::rename(&selected_paths[0].1, &path_to) {
            Ok(_val) => {}
            Err(err) => {
                println!("Couldn't rename, reason: {}", err)
            }
        }
    } else {
        /*Find if mounting point is the same and if so, do rename (much quicker)*/
        let rg = regex::RegexSet::new(&selected_mask.split_ascii_whitespace().collect::<Vec<_>>());
        let rg_ok = rg.is_ok();
        let path_to_clone = path_to.clone();
        'main_for: for (current_inx, (table_name, current_path, inx)) in selected_paths.iter().enumerate() {
            if rg_ok
                && !rg.as_ref().unwrap().is_match(current_path)
                && !PathBuf::from(current_path).metadata().unwrap().is_dir()
            {
                /*we filter for files only here, content of a dir is being filtered in fs_extra */
                continue;
            }
            let progress_handler_path = |process_info: fs_extra::TransitProcess| {
                //Todo, could this be outside of loop?
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

                let (lock, cvar) = &*cond_var_suspend;
                match cvar.wait_while(lock.lock().unwrap(), |pending| *pending) {
                    Err(err) => cb.send(Box::new(cannot_suspend_copy)).unwrap(),
                    _ => {}
                }

                let current_path_clone = current_path.clone();
                cb.send(Box::new(move |siv| {
                    update_cpy_dlg(siv, process_info, current_path_clone, current_inx, is_copy)
                }));
                TransitProcessResult::ContinueOrAbort
            };

            let mut options = fs_extra::dir::CopyOptions::new(); //Initialize default values for CopyOptions
            options.overwrite = is_overwrite;
            options.append = is_append;
            options.copy_inside = is_recursive;
            if !is_recursive {
                options.depth = 1;
            }
            let current_path_name = PathBuf::from(current_path.clone());
            let current_path_name = current_path_name.file_name().unwrap().to_str().unwrap();
            let full_path_to = path_to_clone.clone() + &std::path::MAIN_SEPARATOR.to_string() + current_path_name;
            /*clones*/
            let current_path_clone = PathBuf::from(current_path.clone());
            let cond_var_skip_clone = cond_var_skip.clone();
            let cond_var_suspend_clone = cond_var_suspend.clone();
            let path_to_clone = path_to_clone.clone();
            let cb_clone = cb.clone();
            let table_name_clone = table_name.clone();

            /**/

            match cpy_or_mv(is_copy, &selected_mask,
                (*current_path).clone(),
                path_to.clone(),
                &options,
                progress_handler_path)
        /*match fs_extra::copy_items_with_progress(
            &selected_mask,
            &vec![current_path],
            &path_to,
            &options,
            progress_handler_path,
        ) */{
            Ok(val) => {
                let inx_clone = Arc::new(*inx);
                let table_name_clone = Arc::new(table_name.clone());
                cb.send(Box::new(|s| unselect_inx(s, table_name_clone, inx_clone)))
                    .unwrap();
            }
            Err(err) => match err.kind {
                fs_extra::error::ErrorKind::NotFound => {}
                fs_extra::error::ErrorKind::PermissionDenied => {}
                fs_extra::error::ErrorKind::AlreadyExists => {
                    let mut proceed_with_copy = false;
                    let mut is_overwrite = false;
                    let mut is_append = false;
                    let current_path_clone_internal = current_path_clone.clone();
                    let cond_var_skip_clone_internal = cond_var_skip_clone.clone();
                    let full_path_to_clone = full_path_to.clone();

                    let (lock, cvar, skip_file) = &*cond_var_skip; //todo repeat

                    if !skip_file.lock().unwrap().apply_to_all {
                        cb.send(Box::new(move |s| {
                            copying_already_exists(
                                s,
                                current_path_clone,
                                PathBuf::from(full_path_to_clone),
                                is_overwrite,
                                is_recursive,
                                cond_var_skip_clone,
                            )
                        }))
                        .unwrap();

                        match cvar.wait_while(lock.lock().unwrap(), |pending| *pending) {
                            Err(err) => cb.send(Box::new(cannot_suspend_copy)).unwrap(),
                            _ => {}
                        }
                        /*put the flag down ;)*/
                        *lock.lock().unwrap() = true;
                    }

                    if skip_file.lock().unwrap().dont_overwrite_with_zero
                        && current_path_clone_internal.metadata().unwrap().len() == 0
                    {
                        continue;
                    }
                    match skip_file.lock().unwrap().action {
                        FileExistsAction::Override(OverrideCase::JustDoIt) => {
                            proceed_with_copy = true;
                            is_overwrite = true;
                        }
                        FileExistsAction::Override(OverrideCase::DifferentSize) => {
                            let size_left = current_path_clone_internal.metadata().unwrap().len();
                            let size_right = PathBuf::from(full_path_to).metadata().unwrap().len();
                            if size_left != size_right {
                                proceed_with_copy = true;
                                is_overwrite = true;
                            }
                        }
                        FileExistsAction::Override(OverrideCase::Larger) => {
                            let size_left = current_path_clone_internal.metadata().unwrap().len();
                            let size_right = PathBuf::from(full_path_to).metadata().unwrap().len();
                            if size_left < size_right {
                                proceed_with_copy = true;
                                is_overwrite = true;
                            }
                        }
                        FileExistsAction::Override(OverrideCase::Smaller) => {
                            let size_left = current_path_clone_internal.metadata().unwrap().len();
                            let size_right = PathBuf::from(full_path_to).metadata().unwrap().len();
                            if size_left > size_right {
                                proceed_with_copy = true;
                                is_overwrite = true;
                            }
                        }
                        FileExistsAction::Override(OverrideCase::Older) => {
                            let date_left = current_path_clone_internal.metadata().unwrap().modified().unwrap();
                            let date_right = PathBuf::from(full_path_to).metadata().unwrap().modified().unwrap();
                            if date_right < date_left {
                                proceed_with_copy = true;
                                is_overwrite = true;
                            }
                        }
                        FileExistsAction::Override(OverrideCase::Newer) => {
                            let date_left = current_path_clone_internal.metadata().unwrap().modified().unwrap();
                            let date_right = PathBuf::from(full_path_to).metadata().unwrap().modified().unwrap();
                            if date_right > date_left {
                                proceed_with_copy = true;
                                is_overwrite = true;
                            }
                        }
                        FileExistsAction::Override(OverrideCase::Append) => {
                            proceed_with_copy = true;
                            is_overwrite = true;
                            is_append = true;
                        }

                        FileExistsAction::Abort => {
                            break 'main_for;
                        }
                        FileExistsAction::Skip => {}
                    }
                    if proceed_with_copy {
                        cpy_task(
                            selected_mask.clone(),
                            vec![(
                                table_name_clone,
                                String::from(current_path_clone_internal.to_str().unwrap()),
                                *inx,
                            )],
                            path_to_clone,
                            cb_clone,
                            cond_var_suspend_clone,
                            cond_var_skip_clone_internal,
                            is_recursive,
                            is_overwrite,
                            is_append,
                            is_copy,
                            is_rename,
                        );
                    }
                }
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
        } //file
        }
    }
}
const a_const: i128 = 0;
fn clone_and_get_copying_progress_total_background_text(
    is_copy: bool,
) -> impl Fn(&mut HideableView<ResizedView<ProgressBar>>) {
    let is_copy_clone = is_copy.clone();
    move |a_prgrss_bar: &mut HideableView<ResizedView<ProgressBar>>| {
        a_prgrss_bar
            .get_inner_mut()
            .get_inner_mut()
            .set_label(move |a, (b, c)| {
                literals::copy_progress_dlg::labels::get_copying_progress_total_background_text(is_copy_clone)
            })
    }
}
fn clone_and_get_copying_progress_total_suspended_background_text(
    is_copy: bool,
) -> impl Fn(&mut HideableView<ResizedView<ProgressBar>>) {
    let is_copy_clone = is_copy.clone();
    move |a_prgrss_bar: &mut HideableView<ResizedView<ProgressBar>>| {
        a_prgrss_bar
            .get_inner_mut()
            .get_inner_mut()
            .set_label(move |a, (b, c)| {
                literals::copy_progress_dlg::labels::get_copying_progress_total_suspended_background_text(is_copy_clone)
            })
    }
}

fn suspend_cpy_thread(siv: &mut Cursive, cond_var_suspend: Arc<(Mutex<bool>, Condvar)>, is_copy: bool) {
    let mut suspend_thread = cond_var_suspend.0.lock().unwrap();
    *suspend_thread = if suspend_thread.cmp(&true) == std::cmp::Ordering::Equal {
        siv.call_on_name(
            literals::copy_progress_dlg::widget_names::suspend_resume_btn,
            move |a_button: &mut Button| a_button.set_label("Suspend"),
        )
        .unwrap();
        siv.call_on_name(
            literals::copy_progress_dlg::widget_names::dialog_name,
            move |a_dlg: &mut CopyProgressDlgT| {
                a_dlg
                    .get_mut()
                    .get_inner_mut()
                    .set_title(literals::copy_progress_dlg::labels::get_copy_dialog_title_text(is_copy))
            },
        )
        .unwrap();
        siv.call_on_name(
            literals::copy_progress_dlg::widget_names::hideable_cpy_prgrs_br,
            clone_and_get_copying_progress_total_background_text(is_copy),
        )
        .unwrap();
        false
    } else {
        siv.call_on_name(
            literals::copy_progress_dlg::widget_names::suspend_resume_btn,
            move |a_button: &mut Button| a_button.set_label("Resume"),
        );
        siv.call_on_name(
            literals::copy_progress_dlg::widget_names::dialog_name,
            move |a_dlg: &mut CopyProgressDlgT| {
                a_dlg.get_mut().get_inner_mut().set_title(
                    literals::copy_progress_dlg::labels::get_copy_dialog_title_copying_suspended_text(is_copy),
                )
            },
        )
        .unwrap();
        siv.call_on_name(
            literals::copy_progress_dlg::widget_names::hideable_cpy_prgrs_br,
            clone_and_get_copying_progress_total_suspended_background_text(is_copy),
        )
        .unwrap();
        true
    };

    cond_var_suspend.1.notify_one();
}
type CopyProgressDlgT = NamedView<ResizedView<Dialog>>;
fn create_thmd_cpy_pgrss_dlg(
    siv: &mut Cursive,
    files_total: usize,
    cond_var_suspend: Arc<(Mutex<bool>, Condvar)>,
    is_copy: bool,
) -> ThemedView<Layer<CopyProgressDlgT>> {
    let cpy_progress_dlg = create_cpy_progress_dialog_priv(siv, files_total, cond_var_suspend, is_copy);
    let cpy_progress_dlg = create_themed_view(siv, cpy_progress_dlg);
    cpy_progress_dlg
}
fn create_cpy_progress_dialog_priv(
    siv: &mut Cursive,
    files_total: usize,
    cond_var_suspend: Arc<(Mutex<bool>, Condvar)>,
    is_copy: bool,
) -> CopyProgressDlgT {
    //let g_file_manager = GLOBAL_FileManager.get();
    //let g_file_manager = g_file_manager.lock().unwrap();
    //let is_copying_in_progress = g_file_manager.borrow().cpy_data.is_some();

    let hideable_total = HideableView::new(
        LinearLayout::vertical()
            .child(
                TextView::new(copy_progress_dlg::labels::get_copying_progress_total_text(is_copy))
                    .with_name(copy_progress_dlg::widget_names::text_view_copying_total),
            )
            .child(
                ProgressBar::new()
                    .range(0, files_total)
                    .with_name(copy_progress_dlg::widget_names::progress_bar_total),
            )
            .child(DummyView),
    )
    .visible(files_total > 1);
    let cond_var_suspend_clone = cond_var_suspend.clone();
    let suspend_button = Button::new("Suspend", move |siv| {
        suspend_cpy_thread(siv, cond_var_suspend.clone(), is_copy)
    })
    .with_name(literals::copy_progress_dlg::widget_names::suspend_resume_btn);
    let background_button = Button::new("Background", move |siv| {
        siv.pop_layer();
        show_progress_cpy(siv, files_total, true);
    });
    let cancel_button = Button::new("Cancel", move |siv| {
        siv.pop_layer(); //yes but make sure that update isn't proceeding ;)
        cancel_cpy_operation(cond_var_suspend_clone.clone());
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
                .child(TextView::new("").with_name("TextView_copying_x")) //todo <<
                .child(
                    ProgressBar::new()
                        .range(0, 100)
                        .with_name(copy_progress_dlg::widget_names::progress_bar_current),
                )
                .child(DummyView)
                .child(buttons),
        ),
    )
    .title(literals::copy_progress_dlg::labels::get_copy_dialog_title_text(is_copy))
    .fixed_width(80)
    .with_name(copy_progress_dlg::widget_names::dialog_name);

    cpy_progress_dlg
}

fn copy_engine(
    siv: &mut Cursive,
    selected_mask: Rc<String>,
    paths_from: &PathInfoT,
    path_to: PathBuf,
    is_recursive: bool,
    is_overwrite: bool,
    is_background_cpy: bool,
    is_copy: bool,
    is_rename: bool,
) {
    /*Todo, get rid of clones */
    let cond_var_suspend = Arc::new((Mutex::new(false), Condvar::new()));
    let cond_var_suspend_clone = Arc::clone(&cond_var_suspend);

    let cond_var_skip = Arc::new((
        Mutex::new(true),
        Condvar::new(),
        Mutex::new(FileExistsActionWithOptions {
            action: FileExistsAction::Abort,
            apply_to_all: false,
            dont_overwrite_with_zero: false,
        }),
    ));
    let cond_var_skip_clone = Arc::clone(&cond_var_skip);

    let paths_from_clone = paths_from.clone();
    let path_to_clone: String = String::from(path_to.as_os_str().to_str().unwrap());

    let cb = siv.cb_sink().clone();
    let selected_mask = (*selected_mask).clone();
    #[cfg(feature = "serial_cpy")]
    let handle = std::thread::spawn(move || {
        cpy_task(
            selected_mask,
            paths_from_clone, /*todo clone here?*/
            path_to_clone,
            cb.clone(),
            cond_var_suspend_clone,
            cond_var_skip_clone,
            is_recursive,
            is_overwrite,
            false, //append todo check
            is_copy,
            is_rename,
        );
        cb.send(Box::new(|siv| copying_finished_success(siv)));
    });
    let g_file_manager = GLOBAL_FileManager.get();
    g_file_manager.lock().unwrap().borrow_mut().set_cpy_data(Some(CpyData {
        cond_var_suspend: cond_var_suspend.clone(),
        //       cond_var_skip: cond_var_skip.clone(),
        files_total: paths_from.len(),
    }));

    if !is_background_cpy {
        let cpy_progress_dlg = create_thmd_cpy_pgrss_dlg(siv, paths_from.len(), cond_var_suspend, is_copy);
        siv.add_layer(cpy_progress_dlg);
        siv.set_autorefresh(true);
    }
}

fn cpy_callback(
    siv: &mut Cursive,
    selected_mask: Rc<String>,
    selected_paths_from: &PathInfoT,
    selected_path_to: PathBuf,
    is_recursive: bool,
    is_overwrite: bool,
    is_background_cpy: bool,
    is_copy: bool,
    is_rename: bool,
) {
    if is_background_cpy {
        show_progress_cpy(siv, selected_paths_from.len(), true);
    }
    copy_engine(
        siv,
        selected_mask,
        selected_paths_from,
        selected_path_to,
        is_recursive,
        is_overwrite,
        is_background_cpy,
        is_copy,
        is_rename,
    );
}

fn show_progress_cpy(siv: &mut Cursive, total_files: usize, show_progress_bar: bool) {
    siv.call_on_name(
        copy_progress_dlg::widget_names::hideable_cpy_button,
        |hideable_cpy_btn: &mut HideableView<Button>| {
            hideable_cpy_btn.set_visible(!show_progress_bar);
        },
    );
    siv.call_on_name(
        copy_progress_dlg::widget_names::hideable_cpy_prgrs_br_left_bracket,
        |hideable_bracket: &mut HideableView<TextView>| {
            hideable_bracket.set_visible(show_progress_bar);
        },
    );
    siv.call_on_name(
        copy_progress_dlg::widget_names::hideable_cpy_prgrs_br,
        |hideable_view_total: &mut HideableView<ResizedView<ProgressBar>>| {
            hideable_view_total.set_visible(show_progress_bar);
            hideable_view_total
                .get_inner_mut()
                .get_inner_mut()
                .set_range(0, total_files);
        },
    );
    siv.call_on_name(
        copy_progress_dlg::widget_names::hideable_cpy_prgrs_br_right_bracket,
        |hideable_bracket: &mut HideableView<TextView>| {
            hideable_bracket.set_visible(show_progress_bar);
        },
    );
}
fn get_cpy_dialog_content_cb(
    siv: &mut Cursive,
    selected_paths_from: &PathInfoT,
    is_background_cpy: bool,
    is_copy: bool,
) {
    let selected_mask_from = siv
        .call_on_name("cpy_from_edit_view", move |an_edit_view: &mut EditView| {
            an_edit_view.get_content()
        })
        .unwrap();

    let selected_path_from = siv
        .call_on_name(
            literals::copy_dlg::widget_names::copy_from_edit_view,
            move |an_edit_view: &mut EditView| an_edit_view.get_content(),
        )
        .unwrap();

    let selected_path_to = siv
        .call_on_name(
            literals::copy_dlg::widget_names::copy_to_edit_view,
            move |an_edit_view: &mut EditView| an_edit_view.get_content(),
        )
        .unwrap();

    let is_recursive = siv
        .call_on_name("recursive_chck_bx", move |an_chck_bx: &mut Checkbox| {
            an_chck_bx.is_checked()
        })
        .unwrap();

    let is_overwrite = siv
        .call_on_name("overwrite_chck_bx", move |an_chck_bx: &mut Checkbox| {
            an_chck_bx.is_checked()
        })
        .unwrap();
    if selected_path_to.is_some() && selected_mask_from.is_some() {
        let mut is_rename = false;
        let path_to = PathBuf::from((selected_path_to.unwrap().as_ref()).clone());
        if selected_paths_from.len() == 1 {
            if !path_to.exists() {
                is_rename = true;
            }
        }
        cpy_callback(
            siv,
            selected_mask_from.unwrap(),
            selected_paths_from,
            path_to,
            is_recursive,
            is_overwrite,
            is_background_cpy,
            is_copy,
            is_rename,
        )
    } else {
        siv.add_layer(Dialog::around(TextView::new(
            "Please fill appropriate fields and try again",
        )));
    }
}

fn get_cpy_dialog_content_clone_cb(
    paths_from: &PathInfoT,
    is_background: bool,
    is_copy: bool,
) -> impl Fn(&mut Cursive) {
    let clone = paths_from.clone();
    move |s| {
        get_cpy_dialog_content_cb(s, &clone, is_background, is_copy);
        match s.screen_mut().find_layer_from_name(copy_dlg::widget_names::dialog_name) {
            Some(layer_position) => {
                s.screen_mut().remove_layer(layer_position);
            }
            None => {}
        }
    }
}

fn create_cpy_dialog(paths_from: PathInfoT, path_to: String, is_copy: bool) -> Dialog {
    let paths_from_clone = paths_from.clone();
    let mut cpy_dialog = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new(
                literals::copy_progress_dlg::labels::get_copy_n_items_with_mask_text(is_copy, paths_from.len()),
            ))
            .child(
                EditView::new()
                    .content("*.*")
                    .with_name(literals::copy_dlg::widget_names::copy_from_edit_view)
                    .min_width(100),
            )
            .child(DummyView)
            .child(TextView::new(literals::copy_progress_dlg::labels::get_copy_to_text(
                is_copy,
            )))
            .child(
                EditView::new()
                    .content(path_to)
                    .with_name(literals::copy_dlg::widget_names::copy_to_edit_view)
                    .min_width(100),
            )
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
                    ),
            )
            .child(DummyView),
    )
    .title(literals::copy_progress_dlg::labels::get_copy_dialog_title(is_copy))
    .button(
        "[ OK ]",
        get_cpy_dialog_content_clone_cb(&paths_from_clone.clone(), false, is_copy), /*Close our dialog*/
                                                                                    //        s.pop_layer();
    )
    .button(
        "[ Background ]",
        get_cpy_dialog_content_clone_cb(&paths_from_clone.clone(), true, is_copy), /*Close our dialog*/
                                                                                   //s.pop_layer();
    )
    .button("[ Cancel ]", |s| {
        s.pop_layer();
    });

    cpy_dialog.set_focus(DialogFocus::Button(0));
    cpy_dialog
}
pub fn cpy_mv_helper(siv: &mut cursive::Cursive, is_copy: bool) //todo remove pub
{
    /*First, check if copying is in the progress:*/
    let cpy_data;
    {
        let g_file_manager = GLOBAL_FileManager.get().lock();
        match g_file_manager {
            Ok(file_manager) => {
                cpy_data = file_manager.borrow().get_cpy_data().clone();
            }
            Err(err) => {
                panic!("Cannot unwrap")
            }
        }
    }
    //if let Some(ref cpy_data) = GLOBAL_FileManager.get().lock().unwrap().borrow().get_cpy_data() {
    if cpy_data.is_some() {
        if let None = siv.find_name::<ProgressDlgT>(copy_progress_dlg::widget_names::dialog_name) {
            let cpy_data_unwrapped = cpy_data.unwrap();
            let cpy_progress_dlg = create_thmd_cpy_pgrss_dlg(
                siv,
                cpy_data_unwrapped.files_total,
                cpy_data_unwrapped.cond_var_suspend.clone(),
                is_copy,
            );
            siv.add_layer(cpy_progress_dlg);
            if true {
                //todo refactor
                let suspend_thread = cpy_data_unwrapped.cond_var_suspend.0.lock().unwrap();
                if suspend_thread.cmp(&true) == std::cmp::Ordering::Equal {
                    /*todo refactor*/
                    siv.call_on_name(
                        literals::copy_progress_dlg::widget_names::dialog_name,
                        move |a_dlg: &mut ProgressDlgT| {
                            a_dlg.get_inner_mut().set_title(
                                literals::copy_progress_dlg::labels::get_copy_dialog_title_copying_suspended_text(
                                    is_copy,
                                ),
                            )
                        },
                    )
                    .unwrap();
                    siv.call_on_name(
                        literals::copy_progress_dlg::widget_names::suspend_resume_btn,
                        move |a_button: &mut Button| a_button.set_label("Resume"),
                    )
                    .unwrap();
                }
            }
            siv.set_autorefresh(true);
        }
    } else {
        /*No copying, let'siv start it then but first: ;)*/
        /*Check if we already presenting CpyDlg and if not ...*/
        if let None = siv
            .screen_mut()
            .find_layer_from_name(copy_dlg::widget_names::dialog_name)
        {
            let active_panel = get_active_panel(siv);

            let (from, to) = if active_panel == main_ui::widget_names::LEFT_PANEL_TABLE_ID {
                (
                    main_ui::widget_names::LEFT_PANEL_TABLE_ID,
                    main_ui::widget_names::RIGHT_PANEL_TABLE_ID,
                )
            } else {
                (
                    main_ui::widget_names::RIGHT_PANEL_TABLE_ID,
                    main_ui::widget_names::LEFT_PANEL_TABLE_ID,
                )
            };

            match get_selected_paths(siv, from) {
                Some(selected_paths_from) => {
                    let selected_path_to = get_current_dir(siv, get_panel_id_from_table_id(to));
                    let cpy_dlg = create_cpy_dialog(selected_paths_from, selected_path_to, is_copy);
                    let cpy_dlg = create_themed_view(siv, cpy_dlg).with_name(copy_dlg::widget_names::dialog_name);
                    siv.add_layer(cpy_dlg);
                }
                None => {
                    let info_dlg = create_themed_view(
                        siv,
                        Atomic_Dialog::around(TextView::new(
                            literals::copy_progress_dlg::labels::get_select_item_to_copy_to_text(is_copy),
                        ))
                        .dismiss_button("[ OK ]"),
                    );
                    siv.add_layer(info_dlg);
                }
            }
        }
    }
}
pub fn cpy(siv: &mut cursive::Cursive) {
    cpy_mv_helper(siv, true);
}

pub fn cancel_cpy_operation(cond_var_suspend: Arc<(Mutex<bool>, Condvar)>) {
    /*Not happy about this below...*/
    /*Make sure thread is not suspended*/
    let mut suspend_thread = cond_var_suspend.0.lock().unwrap();
    *suspend_thread = false; //put the flag up
    cond_var_suspend.1.notify_one();

    let mutex_g_file_manager = GLOBAL_FileManager.get();
    let mutex_guard_g_file_manager = mutex_g_file_manager.lock().unwrap();
    let g_file_manager = mutex_guard_g_file_manager.borrow_mut();
    g_file_manager.tx_rx.0.send(AtomicFileTransitFlags::Abort).unwrap();
}
