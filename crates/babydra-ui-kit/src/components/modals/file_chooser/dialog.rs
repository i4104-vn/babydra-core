//! Modern BabyDra File Chooser / Picker Dialog
//! Replaces the default Linux / GTK file chooser with a tailored BabyDra experience.

use std::boxed::Box as StdBox;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use gtk4::gdk::Key;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, DropDown, Entry, EventControllerKey, GestureClick, Label, ListBox,
    ListBoxRow, Orientation, ScrolledWindow, StringList, Window,
};

use babydra_core::models::explore::file_entry::{FileEntry, FileType};
use babydra_core::services::file_dialog::{
    get_default_places, matches_filter, FileDialogMode, FileDialogOptions,
};

use crate::ui::icon::get_icon;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    List,
    Grid,
}

struct DialogState {
    current_path: PathBuf,
    history_back: Vec<PathBuf>,
    history_forward: Vec<PathBuf>,
    selected_paths: Vec<PathBuf>,
    options: FileDialogOptions,
    active_filter_idx: Option<usize>,
    search_query: String,
    show_hidden: bool,
    _view_mode: ViewMode,
    all_entries: Vec<FileEntry>,
}

/// Creates and presents the BabyDra File Chooser Dialog.
pub fn show_file_chooser_dialog(
    parent: Option<&Window>,
    options: FileDialogOptions,
    on_selected: impl Fn(Option<Vec<PathBuf>>) + 'static,
) {
    let initial_path = options
        .initial_folder
        .clone()
        .filter(|p| p.is_dir())
        .or_else(|| Some(glib::home_dir()))
        .unwrap_or_else(|| PathBuf::from("/"));

    let state = Rc::new(RefCell::new(DialogState {
        current_path: initial_path,
        history_back: Vec::new(),
        history_forward: Vec::new(),
        selected_paths: Vec::new(),
        active_filter_idx: options.default_filter_index,
        show_hidden: options.show_hidden,
        _view_mode: ViewMode::List,
        all_entries: Vec::new(),
        options,
        search_query: String::new(),
    }));

    let on_selected_cb: Rc<RefCell<Option<StdBox<dyn Fn(Option<Vec<PathBuf>>)>>>> =
        Rc::new(RefCell::new(Some(StdBox::new(on_selected))));

    let window = Window::new();
    window.set_title(Some(&state.borrow().options.title));
    window.set_default_size(880, 560);
    window.add_css_class("file-chooser-window");
    window.add_css_class("explore-dialog");

    if let Some(p) = parent {
        window.set_transient_for(Some(p));
        window.set_modal(true);
    }

    let root_vbox = Box::new(Orientation::Vertical, 0);
    root_vbox.add_css_class("file-chooser-root");
    window.set_child(Some(&root_vbox));

    // ── 1. Top Header Bar ───────────────────────────────────────────
    let header_bar = Box::new(Orientation::Horizontal, 12);
    header_bar.add_css_class("file-chooser-header");
    header_bar.set_margin_start(16);
    header_bar.set_margin_end(16);
    header_bar.set_margin_top(12);
    header_bar.set_margin_bottom(8);

    let title_lbl = Label::new(Some(&state.borrow().options.title));
    title_lbl.add_css_class("file-chooser-title");
    title_lbl.set_halign(Align::Start);
    title_lbl.set_hexpand(true);
    header_bar.append(&title_lbl);

    let btn_cancel_top = Button::with_label("Cancel");
    btn_cancel_top.add_css_class("modern-dialog-cancel-btn");
    btn_cancel_top.set_cursor_from_name(Some("pointer"));
    header_bar.append(&btn_cancel_top);

    let confirm_label = state
        .borrow()
        .options
        .accept_label
        .clone()
        .unwrap_or_else(|| match state.borrow().options.mode {
            FileDialogMode::OpenFile | FileDialogMode::OpenMultipleFiles => "Open".to_string(),
            FileDialogMode::OpenFolder => "Select".to_string(),
            FileDialogMode::SaveFile => "Save".to_string(),
        });

    let btn_confirm_top = Button::with_label(&confirm_label);
    btn_confirm_top.add_css_class("modern-dialog-primary-btn");
    btn_confirm_top.set_cursor_from_name(Some("pointer"));
    header_bar.append(&btn_confirm_top);

    root_vbox.append(&header_bar);

    // ── 2. Navigation / Toolbar Row ─────────────────────────────────
    let nav_row = Box::new(Orientation::Horizontal, 6);
    nav_row.add_css_class("file-chooser-nav-row");
    nav_row.set_margin_start(16);
    nav_row.set_margin_end(16);
    nav_row.set_margin_bottom(8);

    let btn_back = Button::new();
    btn_back.set_child(Some(&get_icon("arrow-left", 16)));
    btn_back.add_css_class("flat");
    btn_back.add_css_class("nav-btn");
    btn_back.set_tooltip_text(Some("Back"));

    let btn_forward = Button::new();
    btn_forward.set_child(Some(&get_icon("arrow-right", 16)));
    btn_forward.add_css_class("flat");
    btn_forward.add_css_class("nav-btn");
    btn_forward.set_tooltip_text(Some("Forward"));

    let btn_up = Button::new();
    btn_up.set_child(Some(&get_icon("arrow-up", 16)));
    btn_up.add_css_class("flat");
    btn_up.add_css_class("nav-btn");
    btn_up.set_tooltip_text(Some("Parent Directory"));

    nav_row.append(&btn_back);
    nav_row.append(&btn_forward);
    nav_row.append(&btn_up);

    // Breadcrumbs container
    let breadcrumb_box = Box::new(Orientation::Horizontal, 2);
    breadcrumb_box.add_css_class("file-chooser-breadcrumbs");
    breadcrumb_box.set_hexpand(true);
    breadcrumb_box.set_halign(Align::Fill);
    nav_row.append(&breadcrumb_box);

    // Search input
    let search_entry = Entry::new();
    search_entry.set_placeholder_text(Some("Search files..."));
    search_entry.add_css_class("file-chooser-search");
    search_entry.set_width_request(160);
    nav_row.append(&search_entry);

    // Hidden files toggle
    let btn_hidden = Button::new();
    btn_hidden.set_child(Some(&get_icon("eye", 16)));
    btn_hidden.add_css_class("flat");
    btn_hidden.add_css_class("nav-btn");
    btn_hidden.set_tooltip_text(Some("Toggle hidden files (Ctrl+H)"));
    nav_row.append(&btn_hidden);

    // New Folder button
    let btn_new_folder = Button::new();
    btn_new_folder.set_child(Some(&get_icon("folder-new", 16)));
    btn_new_folder.add_css_class("flat");
    btn_new_folder.add_css_class("nav-btn");
    btn_new_folder.set_tooltip_text(Some("New Folder"));
    nav_row.append(&btn_new_folder);

    root_vbox.append(&nav_row);

    // ── 3. Main Center Split (Sidebar Places + File List View) ───────
    let split_box = Box::new(Orientation::Horizontal, 0);
    split_box.set_vexpand(true);
    split_box.set_hexpand(true);
    root_vbox.append(&split_box);

    // Sidebar: Places Bookmarks
    let sidebar_scroll = ScrolledWindow::new();
    sidebar_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    sidebar_scroll.add_css_class("file-chooser-sidebar-scroll");
    sidebar_scroll.set_width_request(180);

    let sidebar_box = Box::new(Orientation::Vertical, 2);
    sidebar_box.add_css_class("file-chooser-sidebar");
    sidebar_box.set_margin_start(12);
    sidebar_box.set_margin_end(8);
    sidebar_box.set_margin_top(4);
    sidebar_box.set_margin_bottom(4);
    sidebar_scroll.set_child(Some(&sidebar_box));
    split_box.append(&sidebar_scroll);

    // Content: File list view
    let content_scroll = ScrolledWindow::new();
    content_scroll.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    content_scroll.add_css_class("file-chooser-content-scroll");
    content_scroll.set_hexpand(true);
    content_scroll.set_vexpand(true);

    let list_box = ListBox::new();
    list_box.add_css_class("file-chooser-list");
    list_box.set_selection_mode(match state.borrow().options.mode {
        FileDialogMode::OpenMultipleFiles => gtk4::SelectionMode::Multiple,
        _ => gtk4::SelectionMode::Single,
    });
    content_scroll.set_child(Some(&list_box));
    split_box.append(&content_scroll);

    // ── 4. Bottom Controls Bar ──────────────────────────────────────
    let bottom_bar = Box::new(Orientation::Horizontal, 10);
    bottom_bar.add_css_class("file-chooser-bottom-bar");
    bottom_bar.set_margin_start(16);
    bottom_bar.set_margin_end(16);
    bottom_bar.set_margin_top(10);
    bottom_bar.set_margin_bottom(12);

    let file_lbl = Label::new(Some(if state.borrow().options.mode == FileDialogMode::OpenFolder {
        "Folder:"
    } else {
        "File name:"
    }));
    file_lbl.add_css_class("file-chooser-bottom-lbl");
    bottom_bar.append(&file_lbl);

    let file_name_entry = Entry::new();
    file_name_entry.set_hexpand(true);
    file_name_entry.add_css_class("file-chooser-name-entry");
    if let Some(ref name) = state.borrow().options.initial_file {
        file_name_entry.set_text(name);
    }
    bottom_bar.append(&file_name_entry);

    // Filter dropdown if filters available
    let filter_list = StringList::new(&[]);
    for f in &state.borrow().options.filters {
        filter_list.append(&f.name);
    }
    let filter_dropdown = DropDown::new(Some(filter_list), None::<gtk4::Expression>);
    filter_dropdown.add_css_class("file-chooser-filter-dropdown");
    if let Some(idx) = state.borrow().active_filter_idx {
        filter_dropdown.set_selected(idx as u32);
    }
    if !state.borrow().options.filters.is_empty() {
        bottom_bar.append(&filter_dropdown);
    }

    let btn_cancel_bottom = Button::with_label("Cancel");
    btn_cancel_bottom.add_css_class("modern-dialog-cancel-btn");
    btn_cancel_bottom.set_cursor_from_name(Some("pointer"));
    bottom_bar.append(&btn_cancel_bottom);

    let btn_confirm_bottom = Button::with_label(&confirm_label);
    btn_confirm_bottom.add_css_class("modern-dialog-primary-btn");
    btn_confirm_bottom.set_cursor_from_name(Some("pointer"));
    bottom_bar.append(&btn_confirm_bottom);

    root_vbox.append(&bottom_bar);

    // ── 5. Logic & Navigation Callbacks ─────────────────────────────
    let places = get_default_places();
    let sidebar_buttons: Rc<RefCell<Vec<(PathBuf, Button)>>> = Rc::new(RefCell::new(Vec::new()));

    let places_lbl = Label::new(Some("Places"));
    places_lbl.add_css_class("file-chooser-sidebar-header");
    places_lbl.set_halign(Align::Start);
    places_lbl.set_margin_top(4);
    places_lbl.set_margin_bottom(4);
    sidebar_box.append(&places_lbl);

    for place in &places {
        let btn = Button::new();
        btn.add_css_class("file-chooser-sidebar-btn");
        btn.add_css_class("flat");
        btn.set_cursor_from_name(Some("pointer"));

        let row = Box::new(Orientation::Horizontal, 8);
        row.set_halign(Align::Start);
        let icon = get_icon(&place.icon_name, 18);
        row.append(&icon);

        let lbl = Label::new(Some(&place.name));
        lbl.add_css_class("file-chooser-sidebar-label");
        row.append(&lbl);

        btn.set_child(Some(&row));
        sidebar_box.append(&btn);
        sidebar_buttons.borrow_mut().push((place.path.clone(), btn));
    }

    let render_list: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    let navigate_to: Rc<dyn Fn(PathBuf, bool)> = {
        let state = state.clone();
        let render_list = render_list.clone();
        let sidebar_buttons = sidebar_buttons.clone();
        let btn_back = btn_back.clone();
        let btn_forward = btn_forward.clone();
        let breadcrumb_box = breadcrumb_box.clone();
        let file_name_entry = file_name_entry.clone();

        Rc::new(move |target_path: PathBuf, record_history: bool| {
            if !target_path.is_dir() {
                return;
            }

            if record_history {
                let prev = state.borrow().current_path.clone();
                if prev != target_path {
                    state.borrow_mut().history_back.push(prev);
                    state.borrow_mut().history_forward.clear();
                }
            }

            state.borrow_mut().current_path = target_path.clone();
            state.borrow_mut().selected_paths.clear();

            if state.borrow().options.mode == FileDialogMode::OpenFolder {
                file_name_entry.set_text(&target_path.to_string_lossy());
            }

            btn_back.set_sensitive(!state.borrow().history_back.is_empty());
            btn_forward.set_sensitive(!state.borrow().history_forward.is_empty());

            for (p, b) in sidebar_buttons.borrow().iter() {
                if p == &target_path {
                    b.add_css_class("active-nav");
                } else {
                    b.remove_css_class("active-nav");
                }
            }

            while let Some(ch) = breadcrumb_box.first_child() {
                breadcrumb_box.remove(&ch);
            }

            let mut segments = Vec::new();
            let mut curr: Option<&Path> = Some(&target_path);
            while let Some(p) = curr {
                if let Some(name) = p.file_name().and_then(|f| f.to_str()) {
                    segments.push((name.to_string(), p.to_path_buf()));
                } else if p == Path::new("/") {
                    segments.push(("/".to_string(), PathBuf::from("/")));
                }
                curr = p.parent();
            }
            segments.reverse();

            for (idx, (name, path_val)) in segments.iter().enumerate() {
                if idx > 0 {
                    let sep = Label::new(Some("›"));
                    sep.add_css_class("file-chooser-breadcrumb-sep");
                    breadcrumb_box.append(&sep);
                }

                let btn_seg = Button::with_label(name);
                btn_seg.add_css_class("file-chooser-breadcrumb-btn");
                btn_seg.add_css_class("flat");
                btn_seg.set_cursor_from_name(Some("pointer"));

                let target_click = path_val.clone();
                btn_seg.connect_clicked({
                    let state = state.clone();
                    let render_list = render_list.clone();
                    let btn_back = btn_back.clone();
                    let btn_forward = btn_forward.clone();
                    move |_| {
                        let prev = state.borrow().current_path.clone();
                        if prev != target_click {
                            state.borrow_mut().history_back.push(prev);
                            state.borrow_mut().history_forward.clear();
                            state.borrow_mut().current_path = target_click.clone();
                            btn_back.set_sensitive(true);
                            btn_forward.set_sensitive(false);
                            if let Some(ref r) = *render_list.borrow() {
                                r();
                            }
                        }
                    }
                });
                breadcrumb_box.append(&btn_seg);
            }

            if let Ok(entries) = babydra_core::services::explore::load_directory_sync(target_path.clone(), true) {
                state.borrow_mut().all_entries = entries;
            } else {
                state.borrow_mut().all_entries.clear();
            }

            if let Some(ref render) = *render_list.borrow() {
                render();
            }
        })
    };

    for (place_path, btn) in sidebar_buttons.borrow().iter() {
        let path_clone = place_path.clone();
        let nav = navigate_to.clone();
        btn.connect_clicked(move |_| {
            nav(path_clone.clone(), true);
        });
    }

    // Render list contents
    {
        let state = state.clone();
        let list_box = list_box.clone();
        let navigate_to = navigate_to.clone();
        let window = window.clone();
        let on_selected_cb = on_selected_cb.clone();

        let render_fn: Rc<dyn Fn()> = Rc::new(move || {
            while let Some(ch) = list_box.first_child() {
                list_box.remove(&ch);
            }

            let st = state.borrow();
            let show_hidden = st.show_hidden;
            let query = st.search_query.to_lowercase();
            let active_filter = st
                .active_filter_idx
                .and_then(|idx| st.options.filters.get(idx));

            let mut filtered_entries: Vec<FileEntry> = st
                .all_entries
                .iter()
                .filter(|e| {
                    if !show_hidden && e.is_hidden {
                        return false;
                    }
                    if !query.is_empty() && !e.display_name.to_lowercase().contains(&query) {
                        return false;
                    }
                    if e.file_type != FileType::Directory {
                        if st.options.mode == FileDialogMode::OpenFolder {
                            return false;
                        }
                        if let Some(filter) = active_filter {
                            if !matches_filter(&e.path, filter) {
                                return false;
                            }
                        }
                    }
                    true
                })
                .cloned()
                .collect();

            filtered_entries.sort_by(|a, b| {
                let a_dir = a.file_type == FileType::Directory;
                let b_dir = b.file_type == FileType::Directory;
                if a_dir && !b_dir {
                    std::cmp::Ordering::Less
                } else if !a_dir && b_dir {
                    std::cmp::Ordering::Greater
                } else {
                    a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase())
                }
            });

            for entry in filtered_entries {
                let row = ListBoxRow::new();
                row.add_css_class("file-chooser-row");

                let hbox = Box::new(Orientation::Horizontal, 12);
                hbox.set_margin_start(10);
                hbox.set_margin_end(10);
                hbox.set_margin_top(6);
                hbox.set_margin_bottom(6);

                let is_dir = entry.file_type == FileType::Directory;
                let icon_name = if is_dir {
                    "folder"
                } else {
                    &entry.icon_name
                };

                let icon = get_icon(icon_name, 22);
                hbox.append(&icon);

                let name_lbl = Label::new(Some(&entry.display_name));
                name_lbl.add_css_class("file-chooser-filename");
                name_lbl.set_halign(Align::Start);
                name_lbl.set_hexpand(true);
                hbox.append(&name_lbl);

                let size_text = if is_dir {
                    String::new()
                } else {
                    format_file_size(entry.size)
                };
                let size_lbl = Label::new(Some(&size_text));
                size_lbl.add_css_class("file-chooser-size");
                size_lbl.set_width_request(80);
                size_lbl.set_halign(Align::End);
                hbox.append(&size_lbl);

                let date_text = entry
                    .modified
                    .map(format_system_time)
                    .unwrap_or_default();
                let date_lbl = Label::new(Some(&date_text));
                date_lbl.add_css_class("file-chooser-date");
                date_lbl.set_width_request(130);
                date_lbl.set_halign(Align::End);
                hbox.append(&date_lbl);

                row.set_child(Some(&hbox));
                list_box.append(&row);

                let click_gesture = GestureClick::new();
                let entry_path = entry.path.clone();
                let nav = navigate_to.clone();
                let is_open_file = state.borrow().options.mode == FileDialogMode::OpenFile;
                let win = window.clone();
                let on_sel = on_selected_cb.clone();

                click_gesture.connect_pressed(move |_, n_press, _, _| {
                    if n_press == 2 {
                        if is_dir {
                            nav(entry_path.clone(), true);
                        } else if is_open_file {
                            if let Some(cb) = on_sel.borrow_mut().take() {
                                cb(Some(vec![entry_path.clone()]));
                            }
                            win.close();
                        }
                    }
                });
                row.add_controller(click_gesture);
            }
        });

        *render_list.borrow_mut() = Some(render_fn);
    }

    // List selection changed -> update filename entry and state
    {
        let state = state.clone();
        let file_name_entry = file_name_entry.clone();

        list_box.connect_row_selected(move |_, row| {
            if let Some(r) = row {
                let idx = r.index() as usize;
                let st = state.borrow();
                let show_hidden = st.show_hidden;
                let query = st.search_query.to_lowercase();
                let active_filter = st
                    .active_filter_idx
                    .and_then(|i| st.options.filters.get(i));

                let filtered: Vec<&FileEntry> = st
                    .all_entries
                    .iter()
                    .filter(|e| {
                        if !show_hidden && e.is_hidden {
                            return false;
                        }
                        if !query.is_empty() && !e.display_name.to_lowercase().contains(&query) {
                            return false;
                        }
                        if e.file_type != FileType::Directory {
                            if st.options.mode == FileDialogMode::OpenFolder {
                                return false;
                            }
                            if let Some(filter) = active_filter {
                                if !matches_filter(&e.path, filter) {
                                    return false;
                                }
                            }
                        }
                        true
                    })
                    .collect();

                if let Some(entry) = filtered.get(idx) {
                    state.borrow_mut().selected_paths = vec![entry.path.clone()];
                    file_name_entry.set_text(&entry.display_name);
                }
            }
        });
    }

    // Back button
    btn_back.connect_clicked({
        let state = state.clone();
        let nav = navigate_to.clone();
        move |_| {
            if let Some(prev) = state.borrow_mut().history_back.pop() {
                let cur = state.borrow().current_path.clone();
                state.borrow_mut().history_forward.push(cur);
                nav(prev, false);
            }
        }
    });

    // Forward button
    btn_forward.connect_clicked({
        let state = state.clone();
        let nav = navigate_to.clone();
        move |_| {
            if let Some(next) = state.borrow_mut().history_forward.pop() {
                let cur = state.borrow().current_path.clone();
                state.borrow_mut().history_back.push(cur);
                nav(next, false);
            }
        }
    });

    // Up button
    btn_up.connect_clicked({
        let state = state.clone();
        let nav = navigate_to.clone();
        move |_| {
            let cur = state.borrow().current_path.clone();
            if let Some(parent) = cur.parent() {
                nav(parent.to_path_buf(), true);
            }
        }
    });

    // Search input change
    search_entry.connect_changed({
        let state = state.clone();
        let render_list = render_list.clone();
        move |entry| {
            state.borrow_mut().search_query = entry.text().to_string();
            if let Some(ref r) = *render_list.borrow() {
                r();
            }
        }
    });

    // Hidden files toggle
    btn_hidden.connect_clicked({
        let state = state.clone();
        let render_list = render_list.clone();
        move |_| {
            let cur = state.borrow().show_hidden;
            state.borrow_mut().show_hidden = !cur;
            if let Some(ref r) = *render_list.borrow() {
                r();
            }
        }
    });

    // New Folder button
    btn_new_folder.connect_clicked({
        let state = state.clone();
        let nav = navigate_to.clone();
        move |_| {
            let cur_dir = state.borrow().current_path.clone();
            let mut name = "New Folder".to_string();
            let mut count = 1;
            while cur_dir.join(&name).exists() {
                count += 1;
                name = format!("New Folder {}", count);
            }
            let target = cur_dir.join(&name);
            if std::fs::create_dir(&target).is_ok() {
                nav(target, true);
            }
        }
    });

    // Filter dropdown change
    filter_dropdown.connect_selected_notify({
        let state = state.clone();
        let render_list = render_list.clone();
        move |dd| {
            state.borrow_mut().active_filter_idx = Some(dd.selected() as usize);
            if let Some(ref r) = *render_list.borrow() {
                r();
            }
        }
    });

    // Confirm execution
    let do_confirm: Rc<dyn Fn()> = {
        let state = state.clone();
        let file_name_entry = file_name_entry.clone();
        let on_selected_cb = on_selected_cb.clone();
        let window = window.clone();

        Rc::new(move || {
            let st = state.borrow();
            let cur_path = st.current_path.clone();
            let mode = st.options.mode;
            let typed_name = file_name_entry.text().trim().to_string();

            let final_paths = match mode {
                FileDialogMode::OpenFolder => {
                    if !st.selected_paths.is_empty() && st.selected_paths[0].is_dir() {
                        st.selected_paths.clone()
                    } else {
                        vec![cur_path]
                    }
                }
                FileDialogMode::SaveFile => {
                    if typed_name.is_empty() {
                        return;
                    }
                    vec![cur_path.join(typed_name)]
                }
                FileDialogMode::OpenFile => {
                    if !st.selected_paths.is_empty() {
                        st.selected_paths.clone()
                    } else if !typed_name.is_empty() {
                        let candidate = cur_path.join(&typed_name);
                        if candidate.exists() {
                            vec![candidate]
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                }
                FileDialogMode::OpenMultipleFiles => {
                    if !st.selected_paths.is_empty() {
                        st.selected_paths.clone()
                    } else {
                        return;
                    }
                }
            };

            if let Some(cb) = on_selected_cb.borrow_mut().take() {
                cb(Some(final_paths));
            }
            window.close();
        })
    };

    // Cancel execution
    let do_cancel: Rc<dyn Fn()> = {
        let on_selected_cb = on_selected_cb.clone();
        let window = window.clone();
        Rc::new(move || {
            if let Some(cb) = on_selected_cb.borrow_mut().take() {
                cb(None);
            }
            window.close();
        })
    };

    let confirm_c1 = do_confirm.clone();
    btn_confirm_top.connect_clicked(move |_| confirm_c1());

    let confirm_c2 = do_confirm.clone();
    btn_confirm_bottom.connect_clicked(move |_| confirm_c2());

    let cancel_c1 = do_cancel.clone();
    btn_cancel_top.connect_clicked(move |_| cancel_c1());

    let cancel_c2 = do_cancel.clone();
    btn_cancel_bottom.connect_clicked(move |_| cancel_c2());

    file_name_entry.connect_activate({
        let confirm_c = do_confirm.clone();
        move |_| confirm_c()
    });

    let key_controller = EventControllerKey::new();
    {
        let do_cancel = do_cancel.clone();
        let state = state.clone();
        let render_list = render_list.clone();
        key_controller.connect_key_pressed(move |_, key, _, modifier| {
            if key == Key::Escape {
                do_cancel();
                return gtk4::glib::Propagation::Stop;
            }
            if modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK)
                && (key == Key::h || key == Key::H)
            {
                let cur = state.borrow().show_hidden;
                state.borrow_mut().show_hidden = !cur;
                if let Some(ref r) = *render_list.borrow() {
                    r();
                }
                return gtk4::glib::Propagation::Stop;
            }
            gtk4::glib::Propagation::Proceed
        });
    }
    window.add_controller(key_controller);

    navigate_to(state.borrow().current_path.clone(), false);

    window.present();
}

fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_system_time(time: SystemTime) -> String {
    let dt: DateTime<Local> = time.into();
    dt.format("%d %b %Y, %H:%M").to_string()
}
