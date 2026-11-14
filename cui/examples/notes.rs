//! Notes — a 1:1 port of examples/notes.c from the C repo. Split view,
//! search, create, edit, pin and delete notes, with keyboard shortcuts and a
//! command palette. On a phone the list and the editor take turns.

use cui::{
    App, AppDesc, Button, ButtonVariant, Color, Command, ControlSize, Dialog, Input, MenuItem,
    Row, Size, Tone, Ui,
};

const MAX_NOTES: usize = 64;
const TAGS: [&str; 3] = ["Personal", "Work", "Ideas"];

#[derive(Clone)]
struct Note {
    title: String,
    body: String,
    pinned: bool,
    tag: i32,
}

#[derive(Default)]
struct NotesApp {
    notes: Vec<Note>,
    selected: i32,
    query: String,
    split: f32,
    palette: bool,
    confirm_delete: bool,
    dark: bool,
    filter: i32,
    one_pane: bool, // a phone: the list or the open note
    editing: bool,
}

fn matches(n: &Note, q: &str) -> bool {
    q.is_empty() || format!("{} {}", n.title, n.body).to_lowercase().contains(&q.to_lowercase())
}

impl NotesApp {
    fn add_note(&mut self, title: &str, body: &str, tag: i32) {
        if self.notes.len() >= MAX_NOTES {
            return;
        }
        self.notes.push(Note { title: title.into(), body: body.into(), pinned: false, tag });
        self.selected = self.notes.len() as i32 - 1;
        self.editing = true;
    }

    fn delete_selected(&mut self) {
        if self.selected < 0 || self.selected as usize >= self.notes.len() {
            return;
        }
        self.notes.remove(self.selected as usize);
        if self.selected >= self.notes.len() as i32 {
            self.selected = self.notes.len() as i32 - 1;
        }
    }

    fn list_pane(&mut self, ui: &mut Ui) {
        ui.column(
            &cui::Column {
                width: Size::FILL,
                height: Size::FILL,
                bg: Color::SURFACE_MUTED,
                pad: 12.,
                gap: 10.,
                ..Default::default()
            },
            |ui| {
                ui.row(&Row { gap: 8., pad_x: 4., pad_top: 4., ..Default::default() }, |ui| {
                    ui.text_with(
                        "Notes",
                        &cui::Text {
                            font: cui::font::MEDIUM,
                            size: 16.,
                            width: Size::FILL,
                            nowrap: true,
                            ..Default::default()
                        },
                    );
                    if ui.icon_button_with(
                        cui::ICON_SQUARE_PEN,
                        &Button { tooltip: Some("New note"), shortcut: Some("Ctrl N"), ..Default::default() },
                    ) {
                        let tag = if self.filter > 0 { self.filter - 1 } else { 0 };
                        self.add_note("Untitled", "", tag);
                    }
                });
                ui.search_with(
                    "q",
                    &mut self.query,
                    &Input { placeholder: Some("Search notes"), size: ControlSize::Sm, ..Default::default() },
                );
                const FILTERS: [&str; 4] = ["All", "Personal", "Work", "Ideas"];
                ui.segmented_with(
                    "filter",
                    &FILTERS,
                    &mut self.filter,
                    &cui::Segmented { size: ControlSize::Sm, full_width: true, ..Default::default() },
                );
                ui.scroll(
                    &cui::Scroll { id: Some("list"), gap: 2., fade_edges: true, ..Default::default() },
                    |ui| {
                        let mut shown = 0;
                        for pass in 0..2 {
                            for i in 0..self.notes.len() {
                                let note = &self.notes[i];
                                if (pass == 0) != note.pinned {
                                    continue;
                                }
                                if self.filter > 0 && note.tag != self.filter - 1 {
                                    continue;
                                }
                                if !matches(note, &self.query) {
                                    continue;
                                }
                                // one line of the note: its lines joined with a dot, list markers dropped
                                let preview = if note.body.is_empty() {
                                    "No content".to_string()
                                } else {
                                    note.body
                                        .lines()
                                        .map(|l| l.trim_start_matches(['-', '*', ' ']))
                                        .filter(|l| !l.is_empty())
                                        .collect::<Vec<_>>()
                                        .join("  ·  ")
                                };
                                let _g = ui.push_id_int(i as i32);
                                let title = if note.title.is_empty() { "Untitled" } else { &note.title };
                                let icon = if note.pinned { cui::ICON_BOOKMARK } else { cui::ICON_FILE_TEXT };
                                let selected = self.selected == i as i32;
                                let mut action = ListAction::None;
                                if ui.list_item_with(
                                    title,
                                    &cui::ListItem {
                                        icon: Some(icon),
                                        subtitle: Some(preview.as_str()),
                                        selected,
                                        ..Default::default()
                                    },
                                ) {
                                    action = ListAction::Select(i);
                                }
                                ui.context_menu("note-menu", |ui| {
                                    let pin_label = if note.pinned { "Unpin" } else { "Pin to top" };
                                    let _pinned = note.pinned;
                                    if ui.menu_item_with(
                                        pin_label,
                                        &MenuItem { icon: Some(cui::ICON_BOOKMARK), ..Default::default() },
                                    ) {
                                        action = ListAction::TogglePin(i);
                                    }
                                    let (t, b, tag) = (note.title.clone(), note.body.clone(), note.tag);
                                    if ui.menu_item_with(
                                        "Duplicate",
                                        &MenuItem { icon: Some(cui::ICON_COPY), ..Default::default() },
                                    ) {
                                        action = ListAction::Duplicate(t, b, tag);
                                    }
                                    ui.menu_separator();
                                    if ui.menu_item_with(
                                        "Delete",
                                        &MenuItem { icon: Some(cui::ICON_TRASH_2), danger: true, ..Default::default() },
                                    ) {
                                        action = ListAction::Delete(i);
                                    }
                                });
                                drop(_g);
                                match action {
                                    ListAction::Select(k) => {
                                        self.selected = k as i32;
                                        self.editing = true;
                                    }
                                    ListAction::TogglePin(k) => self.notes[k].pinned = !self.notes[k].pinned,
                                    ListAction::Duplicate(ref t, ref b, tag) => self.add_note(t, b, tag),
                                    ListAction::Delete(k) => {
                                        self.selected = k as i32;
                                        self.confirm_delete = true;
                                    }
                                    ListAction::None => {}
                                }
                                shown += 1;
                            }
                        }
                        if shown == 0 {
                            ui.empty_state_with(
                                "No notes found",
                                &cui::EmptyState {
                                    icon: Some(cui::ICON_SEARCH),
                                    description: Some("Try another search or create a note."),
                                    ..Default::default()
                                },
                            );
                        }
                    },
                );
            },
        );
    }

    fn editor_pane(&mut self, ui: &mut Ui) {
        ui.column(
            &cui::Column {
                width: Size::FILL,
                height: Size::FILL,
                pad_x: if self.one_pane { 16. } else { 40. },
                pad_y: if self.one_pane { 16. } else { 28. },
                gap: 16.,
                ..Default::default()
            },
            |ui| {
                if self.selected < 0 || self.selected as usize >= self.notes.len() {
                    ui.spacer();
                    let clicked = ui.empty_state_with(
                        "Nothing selected",
                        &cui::EmptyState {
                            icon: Some(cui::ICON_NOTEBOOK_PEN),
                            description: Some("Pick a note on the left or start a new one."),
                            action: Some("New note"),
                        },
                    );
                    if clicked {
                        self.add_note("Untitled", "", 0);
                    }
                    ui.spacer();
                    return;
                }
                let idx = self.selected as usize;
                ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
                    if self.one_pane
                        && ui.icon_button_with(
                            cui::ICON_CHEVRON_LEFT,
                            &Button { tooltip: Some("All notes"), ..Default::default() },
                        )
                    {
                        self.editing = false;
                    }
                    let tag = self.notes[idx].tag;
                    ui.badge_with(
                        TAGS[tag as usize],
                        &cui::Badge {
                            tone: match tag {
                                1 => Tone::Info,
                                2 => Tone::Violet,
                                _ => Tone::Neutral,
                            },
                            ..Default::default()
                        },
                    );
                    let pinned = self.notes[idx].pinned;
                    ui.text_with(
                        if pinned { "Pinned" } else { "" },
                        &cui::Text {
                            size: 12.,
                            color: Color::TEXT_MUTED,
                            nowrap: true,
                            width: Size::FILL,
                            ..Default::default()
                        },
                    );
                    if ui.icon_button_with(
                        cui::ICON_BOOKMARK,
                        &Button {
                            tooltip: Some(if pinned { "Unpin" } else { "Pin" }),
                            selected: pinned,
                            color: if pinned { Color::ACCENT } else { Color::TRANSPARENT },
                            ..Default::default()
                        },
                    ) {
                        self.notes[idx].pinned = !pinned;
                    }
                    enum MenuAction {
                        None,
                        Tag(i32),
                        Export,
                        Delete,
                    }
                    let current_tag = self.notes[idx].tag;
                    let mut menu_action = MenuAction::None;
                    ui.menu_with(
                        "Note actions",
                        &cui::Menu {
                            icon_only: true,
                            icon: Some(cui::ICON_ELLIPSIS),
                            placement: cui::Placement::BottomEnd,
                            ..Default::default()
                        },
                        |ui| {
                            for (t, tag_name) in TAGS.iter().enumerate() {
                                if ui.menu_item_with(
                                    tag_name,
                                    &MenuItem {
                                        icon: Some(cui::ICON_TAG),
                                        checked: current_tag == t as i32,
                                        ..Default::default()
                                    },
                                ) {
                                    menu_action = MenuAction::Tag(t as i32);
                                }
                            }
                            ui.menu_separator();
                            if ui.menu_item_with(
                                "Export as text",
                                &MenuItem { icon: Some(cui::ICON_DOWNLOAD), ..Default::default() },
                            ) {
                                menu_action = MenuAction::Export;
                            }
                            ui.menu_separator();
                            if ui.menu_item_with(
                                "Delete note",
                                &MenuItem { icon: Some(cui::ICON_TRASH_2), danger: true, shortcut: Some("Del"), ..Default::default() },
                            ) {
                                menu_action = MenuAction::Delete;
                            }
                        },
                    );
                    match menu_action {
                        MenuAction::Tag(t) => self.notes[idx].tag = t,
                        MenuAction::Export => {
                            let note = &self.notes[idx];
                            let name = format!("{}.txt", if note.title.is_empty() { "note" } else { &note.title });
                            if let Some(path) = cui::file_dialog(&cui::FileDialog {
                                kind: cui::FileDialogKind::Save,
                                title: Some("Export note"),
                                filter_name: Some("Text"),
                                filter: Some("txt"),
                                default_name: Some(&name),
                            }) {
                                match std::fs::write(&path, format!("{}\n\n{}\n", note.title, note.body)) {
                                    Ok(_) => {
                                        ui.toast_with(
                                            "Note exported",
                                            &cui::Toast { description: Some(&path), tone: Tone::Success, ..Default::default() },
                                        );
                                    }
                                    Err(_) => {
                                        ui.toast_with(
                                            "Could not write the file",
                                            &cui::Toast { tone: Tone::Danger, ..Default::default() },
                                        );
                                    }
                                }
                            }
                        }
                        MenuAction::Delete => self.confirm_delete = true,
                        MenuAction::None => {}
                    }
                });
                let _g = ui.push_id_int(self.selected);
                let note = &mut self.notes[idx];
                ui.input_with(
                    "title",
                    &mut note.title,
                    &Input { placeholder: Some("Title"), size: ControlSize::Lg, ..Default::default() },
                );
                ui.textarea_with(
                    "body",
                    &mut note.body,
                    &Input { placeholder: Some("Start writing"), height: -1., ..Default::default() },
                );
                let words = note.body.split_whitespace().count();
                let chars = note.body.len();
                ui.text(format!("{words} words  ·  {chars} characters"));
            },
        );
    }

    fn frame(&mut self, ui: &mut Ui) {
        if ui.shortcut(cui::Mods::PRIMARY, cui::Key::N) {
            self.add_note("Untitled", "", 0);
        }
        if ui.shortcut(cui::Mods::PRIMARY, cui::Key::K) {
            self.palette = true;
        }
        if !ui.wants_keyboard() && ui.key_pressed(cui::Key::Delete) && !self.notes.is_empty() {
            self.confirm_delete = true;
        }
        self.one_pane = ui.viewport_size().x < 640.;
        ui.box_(
            &cui::BoxOpts { width: Size::FILL, height: Size::FILL, bg: Color::BACKGROUND, ..Default::default() },
            |ui| {
                if !self.one_pane {
                    let mut split = self.split;
                    ui.split(
                        "split",
                        &mut split,
                        &cui::Split { min_first: 240., min_second: 360., ..Default::default() },
                        |ui| {
                            self.list_pane(ui);
                            ui.split_next();
                            self.editor_pane(ui);
                        },
                    );
                    self.split = split;
                } else if self.editing && self.selected >= 0 {
                    self.editor_pane(ui);
                } else {
                    self.list_pane(ui);
                }
            },
        );
        #[derive(PartialEq)]
        enum Action {
            None,
            Cancel,
            Delete,
        }
        let mut confirm = self.confirm_delete;
        let mut action = Action::None;
        ui.dialog_with(
            "Delete this note?",
            &mut confirm,
            &Dialog {
                description: Some("The note will be removed permanently."),
                width: 380.,
                ..Default::default()
            },
            |ui| {
                ui.dialog_footer(|ui| {
                    if ui.button("Cancel") {
                        action = Action::Cancel;
                    }
                    if ui.button_with(
                        "Delete note",
                        &Button { variant: ButtonVariant::Danger, ..Default::default() },
                    ) {
                        action = Action::Delete;
                    }
                });
            },
        );
        if action == Action::Delete {
            self.delete_selected();
            confirm = false;
            ui.toast("Note deleted");
        } else if action == Action::Cancel {
            confirm = false;
        }
        self.confirm_delete = confirm;
        const CMDS: [Command; 4] = [
            Command { title: "New note", icon: Some(cui::ICON_SQUARE_PEN), group: Some("Notes"), shortcut: Some("Ctrl N") },
            Command { title: "Pin or unpin note", icon: Some(cui::ICON_BOOKMARK), group: Some("Notes"), shortcut: None },
            Command { title: "Delete note", icon: Some(cui::ICON_TRASH_2), group: Some("Notes"), shortcut: Some("Del") },
            Command { title: "Toggle dark mode", icon: Some(cui::ICON_MOON), group: Some("View"), shortcut: None },
        ];
        match ui.command_palette(&mut self.palette, &CMDS) {
            0 => self.add_note("Untitled", "", 0),
            1 => {
                if self.selected >= 0 {
                    let idx = self.selected as usize;
                    self.notes[idx].pinned = !self.notes[idx].pinned;
                }
            }
            2 => self.confirm_delete = true,
            3 => {
                self.dark = ui.theme_mode() != cui::ThemeMode::Dark;
                ui.set_theme(if self.dark { cui::ThemeMode::Dark } else { cui::ThemeMode::Light });
            }
            _ => {}
        }
    }
}

enum ListAction {
    None,
    Select(usize),
    TogglePin(usize),
    Duplicate(String, String, i32),
    Delete(usize),
}

fn main() {
    let mut a = NotesApp { split: 0.32, ..Default::default() };
    a.add_note("Launch checklist", "Ship the 1.0 release.\n\n- Final pass on docs\n- Record the demo video\n- Write the announcement post\n- Celebrate", 1);
    a.add_note("Reading list", "The Design of Everyday Things\nA Pattern Language\nThe Art of Computer Programming", 0);
    a.add_note("App ideas", "A calm habit tracker with springy check marks. A tiny music player that lives in the menu bar.", 2);
    a.add_note("Meeting notes", "Discussed the roadmap for Q1. Focus on accessibility, keyboard support and docs.", 1);
    a.notes[0].pinned = true;
    a.selected = 0;
    a.editing = false; // a phone opens on the list
    let code = App::run(
        &AppDesc { title: Some("Notes"), width: 1100, height: 720, accent: cui::Accent::Amber, ..Default::default() },
        |ui| a.frame(ui),
    );
    std::process::exit(code);
}
