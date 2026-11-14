//! A to-do list — a 1:1 port of examples/todo.c from the C repo. It shows the
//! things most apps need: your own data as the state, a list that grows, a
//! filter, a confirmation dialog and saving to a file.

use cui::{App, Button, ButtonVariant, Column, Dialog, EmptyState, Input, Row, Ui};
use std::fs;
use std::io::{BufRead, Write as _};

const MAX_TASKS: usize = 200;

#[derive(Clone)]
struct Task {
    title: String,
    done: bool,
}

#[derive(Default)]
struct AppState {
    tasks: Vec<Task>,
    /// What is being typed into the field.
    draft: String,
    /// 0 all, 1 open, 2 done.
    filter: i32,
    /// The "Clear finished tasks?" dialog is open.
    confirm_clear: bool,
    /// The file the list is saved in; empty: it is not saved.
    path: String,
    /// Set when anything changed this frame, so it saves once at the end.
    changed: bool,
}

impl AppState {
    fn save(&self) {
        if self.path.is_empty() {
            return;
        }
        let Ok(mut f) = fs::File::create(&self.path) else { return };
        for t in &self.tasks {
            let _ = writeln!(f, "{} {}", if t.done { 'x' } else { '-' }, t.title);
        }
    }

    fn load(&mut self) {
        let Ok(f) = fs::File::open(&self.path) else { return };
        for line in std::io::BufReader::new(f).lines().map_while(Result::ok) {
            if self.tasks.len() >= MAX_TASKS || line.len() < 3 {
                continue;
            }
            self.tasks.push(Task { done: line.starts_with('x'), title: line[2..].to_string() });
        }
    }

    fn add_task(&mut self) -> bool {
        if self.draft.is_empty() || self.tasks.len() >= MAX_TASKS {
            return false;
        }
        self.tasks.push(Task { title: self.draft.clone(), done: false });
        self.draft.clear();
        true
    }

    fn frame(&mut self, ui: &mut Ui) {
        let done = self.tasks.iter().filter(|t| t.done).count();

        ui.column(
            &Column {
                width: cui::Size::FILL,
                height: cui::Size::FILL,
                align: cui::Align::Center,
                bg: cui::Color::BACKGROUND,
                ..Default::default()
            },
            |ui| {
                ui.column(
                    &Column {
                        width: cui::Size::FILL,
                        max_width: 560.,
                        height: cui::Size::FILL,
                        pad: 24.,
                        gap: 16.,
                        ..Default::default()
                    },
                    |ui| {
                        ui.row(&Row { gap: 12., align: cui::Align::End, ..Default::default() }, |ui| {
                            ui.heading("To do", 2);
                            ui.spacer();
                            ui.text(format!("{} open, {} done", self.tasks.len() - done, done));
                        });

                        // add: Enter in the field or the button
                        ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
                            let typed = ui.input_with(
                                "new-task",
                                &mut self.draft,
                                &Input { placeholder: Some("Add a task"), width: cui::Size::FILL, ..Default::default() },
                            );
                            let empty = self.draft.is_empty();
                            let add = ui.button_with(
                                "Add",
                                &Button {
                                    variant: ButtonVariant::Primary,
                                    icon: Some(cui::ICON_PLUS),
                                    disabled: empty,
                                    ..Default::default()
                                },
                            );
                            if typed.submitted() || add {
                                self.changed |= self.add_task();
                            }
                        });

                        ui.row(&Row { gap: 8., align: cui::Align::Center, ..Default::default() }, |ui| {
                            ui.segmented("filter", &["All", "Open", "Done"], &mut self.filter);
                            ui.spacer();
                            if ui.button_with(
                                "Clear done",
                                &Button { variant: ButtonVariant::Ghost, disabled: done == 0, ..Default::default() },
                            ) {
                                self.confirm_clear = true;
                            }
                        });

                        // the list: each row gets an id scope, so two tasks can have the same title
                        let mut remove: Option<usize> = None;
                        let mut shown = 0;
                        ui.scroll(
                            &cui::Scroll {
                                id: Some("tasks"),
                                width: cui::Size::FILL,
                                height: cui::Size::FILL,
                                gap: 2.,
                                ..Default::default()
                            },
                            |ui| {
                                for i in 0..self.tasks.len() {
                                    let done_i = self.tasks[i].done;
                                    if (self.filter == 1 && done_i) || (self.filter == 2 && !done_i) {
                                        continue;
                                    }
                                    shown += 1;
                                    let _g = ui.push_id_int(i as i32);
                                    let title = self.tasks[i].title.clone();
                                    ui.row(
                                        &Row { gap: 8., align: cui::Align::Center, width: cui::Size::FILL, ..Default::default() },
                                        |ui| {
                                            if ui.checkbox(title, &mut self.tasks[i].done) {
                                                self.changed = true;
                                            }
                                            ui.spacer();
                                            if ui.icon_button_with(
                                                cui::ICON_X,
                                                &Button {
                                                    tooltip: Some("Delete"),
                                                    variant: ButtonVariant::Ghost,
                                                    size: cui::ControlSize::Sm,
                                                    ..Default::default()
                                                },
                                            ) {
                                                remove = Some(i);
                                            }
                                        },
                                    );
                                }
                                if shown == 0 {
                                    let (title, description) = if self.tasks.is_empty() {
                                        ("No tasks yet", "Type one above and press Enter.")
                                    } else {
                                        ("Nothing here", "No task matches this filter.")
                                    };
                                    ui.empty_state_with(
                                        title,
                                        &EmptyState {
                                            icon: Some(cui::ICON_LIST_CHECKS),
                                            description: Some(description),
                                            ..Default::default()
                                        },
                                    );
                                }
                            },
                        );
                        // after the scroll scope, so the rows built this frame stay
                        // in step with the vector (same reason todo.c waits for the loop)
                        if let Some(i) = remove {
                            self.tasks.remove(i);
                            self.changed = true;
                        }
                    },
                );
            },
        );

        // A dialog is drawn while its bool is true; closing it (the x, Esc, a
        // click outside) sets the bool to false. Buttons inside report back
        // through a local action, applied after the call.
        #[derive(PartialEq)]
        enum Confirm {
            None,
            Cancel,
            Clear,
        }
        let mut confirm_clear = self.confirm_clear;
        let mut action = Confirm::None;
        ui.dialog_with(
            "Clear finished tasks?",
            &mut confirm_clear,
            &Dialog {
                description: Some("They are removed from the list and from the file."),
                width: 380.,
                ..Default::default()
            },
            |ui| {
                ui.dialog_footer(|ui| {
                    if ui.button("Cancel") {
                        action = Confirm::Cancel;
                    }
                    if ui.button_with("Clear", &Button { variant: ButtonVariant::Danger, ..Default::default() }) {
                        self.tasks.retain(|t| !t.done);
                        self.changed = true;
                        action = Confirm::Clear;
                        ui.toast("Finished tasks cleared");
                    }
                });
            },
        );
        if action != Confirm::None {
            self.confirm_clear = false;
        }
        self.confirm_clear = confirm_clear;

        if self.changed {
            self.save();
            self.changed = false;
        }
    }
}

fn main() {
    let mut state = AppState { path: "todo.txt".into(), ..Default::default() };
    state.load(); // beside where the app is started
    let code = App::run(
        &cui::AppDesc { title: Some("To do"), width: 520, height: 640, ..Default::default() },
        |ui| state.frame(ui),
    );
    std::process::exit(code);
}
