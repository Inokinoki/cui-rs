//! `todo.rs` rewritten with the `view!` macro: the same app, the same
//! behaviour, the layout described declaratively instead of as nested
//! closures. Compare it with `todo.rs`.

use cui::{view, Align, App, ButtonVariant, Color, ControlSize, Size, Ui};
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

        view! { ui,
            Column(width: Size::FILL, height: Size::FILL, align: Align::Center, bg: Color::BACKGROUND) {
                Column(width: Size::FILL, max_width: 560., height: Size::FILL, pad: 24., gap: 16.) {
                    Row(gap: 12., align: Align::End) {
                        Heading("To do", 2);
                        Spacer();
                        Text(format!("{} open, {} done", self.tasks.len() - done, done));
                    }

                    // add: Enter in the field or the button
                    Row(gap: 8.) {
                        let typed = Input("new-task" => self.draft, placeholder: Some("Add a task"), width: Size::FILL);
                        let empty = self.draft.is_empty();
                        let add = Button("Add", variant: ButtonVariant::Primary, icon: Some(cui::ICON_PLUS), disabled: empty);
                        if typed.submitted() || add {
                            self.changed |= self.add_task();
                        }
                    }

                    Row(gap: 8., align: Align::Center) {
                        Segmented("filter", &["All", "Open", "Done"] => self.filter);
                        Spacer();
                        Button("Clear done", variant: ButtonVariant::Ghost, disabled: done == 0) {
                            self.confirm_clear = true;
                        }
                    }

                    // the list: each row gets an id scope, so two tasks can have the same title
                    let mut remove: Option<usize> = None;
                    let mut shown = 0;
                    Scroll(id: Some("tasks"), width: Size::FILL, height: Size::FILL, gap: 2.) {
                        for i in 0..self.tasks.len() {
                            let done_i = self.tasks[i].done;
                            if (self.filter == 1 && done_i) || (self.filter == 2 && !done_i) {
                                continue;
                            }
                            shown += 1;
                            let _g = ui.push_id_int(i as i32);
                            let title = self.tasks[i].title.clone();
                            Row(gap: 8., align: Align::Center, width: Size::FILL) {
                                Checkbox(title => self.tasks[i].done) {
                                    self.changed = true;
                                }
                                Spacer();
                                IconButton(
                                    cui::ICON_X,
                                    tooltip: Some("Delete"),
                                    variant: ButtonVariant::Ghost,
                                    size: ControlSize::Sm,
                                ) {
                                    remove = Some(i);
                                }
                            }
                        }
                        if shown == 0 {
                            let (title, description) = if self.tasks.is_empty() {
                                ("No tasks yet", "Type one above and press Enter.")
                            } else {
                                ("Nothing here", "No task matches this filter.")
                            };
                            EmptyState(title, icon: Some(cui::ICON_LIST_CHECKS), description: Some(description));
                        }
                    }
                    // after the scroll scope, so the rows built this frame stay
                    // in step with the vector (same reason todo.c waits for the loop)
                    if let Some(i) = remove {
                        self.tasks.remove(i);
                        self.changed = true;
                    }
                }
            }

            Dialog(
                "Clear finished tasks?" => confirm_clear,
                description: Some("They are removed from the list and from the file."),
                width: 380.,
            ) {
                DialogFooter() {
                    Button("Cancel") { action = Confirm::Cancel; }
                    Button("Clear", variant: ButtonVariant::Danger) {
                        self.tasks.retain(|t| !t.done);
                        self.changed = true;
                        action = Confirm::Clear;
                        Toast("Finished tasks cleared");
                    }
                }
            }
        }

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
