//! Assistant — a 1:1 port of examples/assistant.c from the C repo. A chat
//! app with reasoning, tool calls, a plan, sources and a composer. Sending a
//! message plays a short scripted answer. Dark by default.

use cui::{
    Align, App, AppDesc, Button, ButtonVariant, Card, Color, ControlSize, Justify, MenuItem,
    Model, Nav, Placement, PlanStep, Row, Size, TaskStatus, TextAlign, Tone, Ui, VizItem,
};

#[derive(Default)]
struct AppState {
    chat: i32,
    draft: String,
    asked: String,
    model: i32,
    vote: i32,
    started: f64, // when the current answer began, < 0 idle
    asked_once: bool,
    files: Vec<&'static str>,
    removed: i32,
    narrow: bool,  // a phone-sized window
    drawer: bool,  // the chat list on a narrow window
}

fn models() -> [Model<'static>; 3] {
    [
        Model { name: "Atlas", detail: Some("Longer answers, reads files and code"), badge: Some("New"), icon: Some(cui::ICON_LAYERS) },
        Model { name: "Atlas Fast", detail: Some("Short answers in about a second"), badge: None, icon: Some(cui::ICON_ZAP) },
        Model { name: "Atlas Mini", detail: Some("Runs on your own machine"), badge: None, icon: Some(cui::ICON_BOX) },
    ]
}

fn sources() -> [cui::Source<'static>; 3] {
    [
        cui::Source {
            title: Some("Immediate mode GUIs and why they keep coming back"),
            url: "https://fieldnotes.dev/immediate-mode",
            snippet: Some("Retained and immediate mode toolkits and the trade offs between them."),
        },
        cui::Source {
            title: Some("Notes on subpixel text positioning"),
            url: "https://glyphlab.io/notes/subpixel",
            snippet: Some("Shaping, hinting and subpixel positioning explained with examples."),
        },
        cui::Source {
            title: Some("Spring physics for interface motion"),
            url: "https://motionbook.dev/springs",
            snippet: Some("Why spring physics feel more natural than cubic beziers for interface motion."),
        },
    ]
}

const ANSWER: &str = "Mostly because they do very little per frame. The interface is rebuilt from your data each frame, laid out in one pass and drawn in a handful of batches, so there is no tree of widget objects to keep in sync.\n\nText is the expensive part, which is why glyphs are rasterized once into an atlas and reused. And motion feels quick when it is driven by springs that respond to input at once, instead of fixed length curves.";

fn sidebar(ui: &mut Ui, s: &mut AppState, drawer: bool) {
    ui.column(
        &cui::Column {
            width: if drawer { Size::FILL } else { Size::px(264.) },
            height: Size::FILL,
            bg: if drawer { Color::TRANSPARENT } else { Color::SURFACE_MUTED },
            pad: if drawer { 0. } else { 12. },
            gap: 2.,
            ..Default::default()
        },
        |ui| {
            ui.row(&Row { gap: 10., pad_x: 6., pad_top: 4., pad_bottom: 10., align: Align::Center, ..Default::default() }, |ui| {
                ui.box_(
                    &cui::BoxOpts {
                        width: Size::px(28.),
                        height: Size::px(28.),
                        radius: 9.,
                        bg: Color::ACCENT,
                        dir: cui::Dir::Stack,
                        align: Align::Center,
                        justify: Justify::Center,
                        ..Default::default()
                    },
                    |ui| {
                        ui.text_with(
                            "M",
                            &cui::Text {
                                font: cui::font::DISPLAY,
                                size: 16.,
                                color: Color::ACCENT_FOREGROUND,
                                line_height: 1.0,
                                ..Default::default()
                            },
                        );
                    },
                );
                ui.text_with("Muse", &cui::Text { font: cui::font::MEDIUM, size: 15., width: Size::FILL, nowrap: true, ..Default::default() });
                if ui.icon_button_with(cui::ICON_SQUARE_PEN, &Button { size: ControlSize::Sm, tooltip: Some("New chat"), ..Default::default() }) {
                    s.asked_once = false;
                    s.started = -1.;
                    s.draft.clear();
                }
            });
            ui.nav_section("Today");
            const TODAY: [&str; 3] = ["Why immediate mode feels fast", "Quarterly report summary", "Spring presets for menus"];
            for (i, chat) in TODAY.iter().enumerate() {
                if ui.nav_item_with(
                    chat,
                    &Nav { icon: Some(cui::ICON_MESSAGE_SQUARE), selected: s.chat == i as i32, group: Some("chats"), ..Default::default() },
                ) {
                    s.chat = i as i32;
                    s.drawer = false;
                }
            }
            ui.box_(&cui::BoxOpts { height: Size::px(10.), ..Default::default() }, |_| {});
            ui.nav_section("Previous 7 days");
            const WEEK: [&str; 4] = ["Gantt chart layout ideas", "Kanban drag and drop", "Color palette for charts", "Release notes draft"];
            for (i, chat) in WEEK.iter().enumerate() {
                if ui.nav_item_with(
                    chat,
                    &Nav { icon: Some(cui::ICON_MESSAGE_SQUARE), selected: s.chat == 3 + i as i32, group: Some("chats"), ..Default::default() },
                ) {
                    s.chat = 3 + i as i32;
                    s.drawer = false;
                }
            }
            ui.spacer();
            ui.card(&Card { pad: 14., gap: 8., ..Default::default() }, |ui| {
                ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                    ui.icon(cui::ICON_ZAP, 14., Color::ACCENT);
                    ui.text_with("Usage this month", &cui::Text { font: cui::font::MEDIUM, size: 13., width: Size::FILL, ..Default::default() });
                });
                ui.progress_with(0.42, &cui::Progress { height: 6., ..Default::default() });
                ui.text_with("4,210 of 10,000 messages", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.box_(&cui::BoxOpts { height: Size::px(6.), ..Default::default() }, |_| {});
            ui.user_menu_with(
                "Mara Quinn",
                &cui::UserMenu { detail: Some("mara@muse.dev"), status: Some("online"), ..Default::default() },
                |ui| {
                    ui.menu_item_with("Settings", &MenuItem { icon: Some(cui::ICON_SETTINGS), ..Default::default() });
                    ui.menu_item_with("Keyboard shortcuts", &MenuItem { icon: Some(cui::ICON_KEYBOARD), ..Default::default() });
                    ui.menu_separator();
                    ui.menu_item_with("Log out", &MenuItem { icon: Some(cui::ICON_LOG_OUT), ..Default::default() });
                },
            );
        },
    );
}

fn answer_block(ui: &mut Ui, s: &mut AppState, key: &str, t: f32, live: bool) {
    let _g = ui.push_id(key);
    let thinking = live && t < 2.6;
    ui.reasoning(
        "think",
        &cui::Reasoning {
            thinking,
            seconds: if live { if thinking { t } else { 2.6 } } else { 6. },
            ..Default::default()
        },
        |ui| {
            ui.text_with(
                "This is about perceived speed. Two parts: how little work a frame does, and how fast input reaches the screen.",
                &cui::Text { size: 13.5, color: Color::TEXT_SECONDARY, ..Default::default() },
            );
            ui.text_with(
                "Find sources on retained against immediate mode, text rendering cost and interface motion.",
                &cui::Text { size: 13.5, color: Color::TEXT_SECONDARY, ..Default::default() },
            );
        },
    );
    if !live || t > 1.2 {
        let st = if live && t < 3.6 { TaskStatus::Running } else { TaskStatus::Done };
        ui.tool_call(
            "search",
            "web_search",
            &cui::ToolCall {
                icon: Some(cui::ICON_GLOBE),
                detail: Some("immediate mode gui performance"),
                status: st,
                seconds: if st == TaskStatus::Done { 1.4 } else { t - 1.2 },
                input: Some("{\"query\": \"immediate mode gui performance\", \"limit\": 5}"),
                output: Some("{\"results\": 3, \"took_ms\": 1380}"),
            },
        );
    }
    if live && t > 2.6 && t < 4.2 {
        ui.agent_status(
            "Reading 3 sources",
            &cui::AgentStatus { status: TaskStatus::Running, seconds: t - 2.6, ..Default::default() },
        );
    }
    if !live || t > 3.6 {
        if live {
            ui.typewriter(
                "answer",
                ANSWER,
                &cui::TextFx { size: 15., speed: 140., caret: true, ..Default::default() },
            );
        } else {
            ui.text_with(ANSWER, &cui::Text { size: 15., ..Default::default() });
        }
        if !live || t > 6.5 {
            ui.row(&Row { gap: 6., align: Align::Center, ..Default::default() }, |ui| {
                let srcs = sources();
                for (k, source) in srcs.iter().enumerate() {
                    ui.citation(k as i32 + 1, source);
                }
            });
            ui.source_list("sources", &sources());
            let mut vote = s.vote;
            let act = ui.response_actions(
                "actions",
                &mut cui::ResponseActions { copy_text: Some(ANSWER), vote: Some(&mut vote), ..Default::default() },
            );
            s.vote = vote;
            if act.regenerated() {
                s.started = ui.time();
            }
            if act.shared() {
                ui.toast_with("Link copied", &cui::Toast { tone: Tone::Success, ..Default::default() });
            }
        }
    }
}

fn conversation(ui: &mut Ui, s: &mut AppState) {
    let now = ui.time();
    let t = if s.started >= 0. { (now - s.started) as f32 } else { 99. };
    ui.column(&cui::Column { width: Size::FILL, height: Size::FILL, ..Default::default() }, |ui| {
        ui.row(
            &Row {
                width: Size::FILL,
                pad_x: if s.narrow { 12. } else { 20. },
                pad_y: 12.,
                gap: if s.narrow { 6. } else { 10. },
                align: Align::Center,
                ..Default::default()
            },
            |ui| {
                if s.narrow && ui.icon_button_with(cui::ICON_MENU, &Button { tooltip: Some("Chats"), ..Default::default() }) {
                    s.drawer = true;
                }
                ui.model_picker(
                    "model",
                    &models(),
                    &mut s.model,
                    &cui::ModelPicker { ghost: true, placement: Placement::BottomStart },
                );
                ui.spacer();
                const PARTS: [(&str, f32); 3] = [("Instructions", 3200.), ("Files", 41800.), ("Messages", 68400.)];
                let parts: Vec<VizItem> = PARTS
                    .iter()
                    .map(|(label, value)| VizItem { label, value: *value, color: Color::TRANSPARENT })
                    .collect();
                ui.context_meter("ctx", 113400., 200000., &cui::ContextMeter { parts: &parts, ..Default::default() });
                if s.narrow {
                    ui.icon_button_with(cui::ICON_SHARE_2, &Button { tooltip: Some("Share"), ..Default::default() });
                } else {
                    ui.button_with("Share", &Button { icon: Some(cui::ICON_SHARE_2), variant: ButtonVariant::Outline, size: ControlSize::Sm, ..Default::default() });
                }
            },
        );
        ui.separator();
        ui.scroll(
            &cui::Scroll { id: Some("chat"), width: Size::FILL, height: Size::FILL, fade_edges: true, ..Default::default() },
            |ui| {
                ui.row(
                    &Row {
                        width: Size::FILL,
                        justify: Justify::Center,
                        pad_x: if s.narrow { 12. } else { 24. },
                        pad_y: if s.narrow { 16. } else { 24. },
                        ..Default::default()
                    },
                    |ui| {
                        ui.column(&cui::Column { width: Size::FILL, max_width: 760., gap: 14., ..Default::default() }, |ui| {
                            ui.chat_message_with(
                                "Why do immediate mode toolkits feel so fast? Cite a few sources.",
                                &cui::ChatMessage { mine: true, ..Default::default() },
                            );
                            answer_block(ui, s, "first", t, false);
                            if s.asked_once {
                                ui.gap(12.);
                                let asked = s.asked.clone();
                                ui.chat_message_with(&asked, &cui::ChatMessage { mine: true, ..Default::default() });
                                answer_block(ui, s, "second", t, true);
                            }
                        });
                    },
                );
            },
        );
        ui.row(
            &Row {
                width: Size::FILL,
                justify: Justify::Center,
                pad_x: if s.narrow { 12. } else { 24. },
                pad_bottom: if s.narrow { 12. } else { 20. },
                ..Default::default()
            },
            |ui| {
                ui.column(&cui::Column { width: Size::FILL, max_width: 760., gap: 8., ..Default::default() }, |ui| {
                    let busy = s.asked_once && t < 7.5;
                    let mut removed = s.removed;
                    let mut model = s.model;
                    let ev = ui.chat_composer(
                        "composer",
                        &mut s.draft,
                        &mut cui::Composer {
                            placeholder: Some("Ask a follow up"),
                            busy,
                            attachments: &s.files,
                            removed: Some(&mut removed),
                            models: &models(),
                            model: Some(&mut model),
                            ..Default::default()
                        },
                    );
                    s.removed = removed;
                    s.model = model;
                    if ev.sent() {
                        s.asked = s.draft.clone();
                        s.draft.clear();
                        s.asked_once = true;
                        s.started = ui.time();
                    }
                    if ev.stopped() {
                        s.started = ui.time() - 99.;
                    }
                    if ev.removed() && s.removed >= 0 && (s.removed as usize) < s.files.len() {
                        s.files.remove(s.removed as usize);
                    }
                    if ev.attach() {
                        ui.toast("Drop files on the window to attach them");
                    }
                    ui.text_with(
                        "Answers can be wrong. Each one links the sources it used.",
                        &cui::Text { size: 12., color: Color::TEXT_MUTED, align: TextAlign::Center, width: Size::FILL, ..Default::default() },
                    );
                });
            },
        );
    });
}

fn side_panel(ui: &mut Ui, s: &mut AppState) {
    let now = ui.time();
    let t = if s.asked_once && s.started >= 0. { (now - s.started) as f32 } else { 99. };
    ui.column(
        &cui::Column { width: Size::px(330.), height: Size::FILL, bg: Color::SURFACE, pad: 20., gap: 16., ..Default::default() },
        |ui| {
            ui.text_with("Workspace", &cui::Text { font: cui::font::MEDIUM, size: 15., ..Default::default() });
            let steps = [
                PlanStep {
                    title: "Search for sources",
                    detail: Some("3 results"),
                    status: if t > 3.6 { TaskStatus::Done } else { TaskStatus::Running },
                },
                PlanStep {
                    title: "Read the sources",
                    detail: Some(if t > 4.2 { "3 of 3" } else { "Reading" }),
                    status: if t > 4.2 {
                        TaskStatus::Done
                    } else if t > 2.6 {
                        TaskStatus::Running
                    } else {
                        TaskStatus::Pending
                    },
                },
                PlanStep {
                    title: "Write the answer",
                    detail: None,
                    status: if t > 7.5 { TaskStatus::Done } else if t > 3.6 { TaskStatus::Running } else { TaskStatus::Pending },
                },
                PlanStep {
                    title: "Check the citations",
                    detail: None,
                    status: if t > 7.5 { TaskStatus::Done } else { TaskStatus::Pending },
                },
            ];
            ui.agent_plan("plan", &steps, &cui::AgentPlan { title: Some("Plan") });
            ui.label("Files");
            ui.column(&cui::Column { gap: 6., width: Size::FILL, ..Default::default() }, |ui| {
                const NAMES: [&str; 3] = ["quarterly-report.pdf", "metrics.csv", "notes.md"];
                const SIZES: [&str; 3] = ["2.4 MB", "86 KB", "4 KB"];
                for (k, name) in NAMES.iter().enumerate() {
                    let _g = ui.push_id_int(k as i32);
                    ui.row(
                        &Row { gap: 10., align: Align::Center, width: Size::FILL, pad: 8., radius: 12., bg: Color::SURFACE_MUTED, ..Default::default() },
                        |ui| {
                            ui.icon(cui::ICON_FILE_TEXT, 16., Color::TEXT_SECONDARY);
                            ui.text_with(name, &cui::Text { size: 13., width: Size::FILL, nowrap: true, ..Default::default() });
                            ui.text_with(SIZES[k], &cui::Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
                        },
                    );
                }
            });
            ui.label("Try asking");
            const SUGG: [cui::PromptSuggestion; 2] = [
                cui::PromptSuggestion { icon: Some(cui::ICON_CHART_BAR), title: "Compare with last quarter", detail: Some("Revenue, churn and costs") },
                cui::PromptSuggestion { icon: Some(cui::ICON_LIST_CHECKS), title: "Turn this into a checklist", detail: None },
            ];
            let pick = ui.prompt_suggestions("sugg", &SUGG, &cui::PromptSuggestions { columns: 1 });
            if pick >= 0 {
                s.draft = SUGG[pick as usize].title.to_string();
            }
        },
    );
}

fn frame(ui: &mut Ui, s: &mut AppState) {
    let vw = ui.viewport_size().x;
    s.narrow = vw < 760.;
    if !s.narrow {
        s.drawer = false;
    }
    ui.row(
        &Row { width: Size::FILL, height: Size::FILL, align: Align::Stretch, bg: Color::BACKGROUND, ..Default::default() },
        |ui| {
            if !s.narrow {
                sidebar(ui, s, false);
                ui.separator();
            }
            conversation(ui, s);
            if vw >= 1180. {
                ui.separator();
                side_panel(ui, s);
            }
        },
    );
    let mut drawer = s.drawer;
    ui.sheet(
        "Muse",
        &mut drawer,
        &cui::Sheet { side: cui::Side::Left, size: 300., ..Default::default() },
        |ui| sidebar(ui, s, true),
    );
    s.drawer = drawer;
}

fn main() {
    let mut s = AppState {
        started: -1.,
        files: vec!["quarterly-report.pdf", "metrics.csv"],
        removed: -1,
        draft: "Now compare these numbers with last quarter.".into(),
        ..Default::default()
    };
    let code = App::run(
        &AppDesc {
            title: Some("Muse"),
            width: 1440,
            height: 900,
            theme: cui::ThemeMode::Dark,
            accent: cui::Accent::Neutral,
            ..Default::default()
        },
        |ui| frame(ui, &mut s),
    );
    std::process::exit(code);
}
