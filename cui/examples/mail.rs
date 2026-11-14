//! Mail — a 1:1 port of examples/mail.c from the C repo. A three pane inbox
//! with folders, search, a reading pane and a compose dialog. Narrower
//! windows drop to two panes with the folders in a drawer, and a phone shows
//! one pane at a time.

use cui::{
    Align, App, AppDesc, Avatar, Badge, Button, ButtonVariant, CanvasResult, Color, ControlSize,
    Corners, Dialog, EmptyState, Input, Justify, MenuItem, Nav, Rect, Row, Size, Tone, Token, Ui,
};

struct Message {
    from: &'static str,
    email: &'static str,
    subject: &'static str,
    preview: &'static str,
    time: &'static str,
    body: &'static str,
    folder: i32, // 0 inbox, 1 starred, 2 sent, 3 archive
    unread: bool,
    starred: bool,
    label: i32, // -1 none, else tone
}


#[derive(Default)]
#[allow(dead_code)] // dark stays for parity with the C example
struct MailApp {
    messages: Vec<Message>,
    folder: i32,
    selected: i32,
    query: String,
    split1: f32,
    split2: f32,
    compose: bool,
    to: String,
    subject: String,
    body: String,
    dark: bool,
    panes: i32,   // 3, 2 or 1, from the window width
    reading: bool, // one pane: the message is open
    drawer: bool,  // folders in a drawer below three panes
}

fn build_messages() -> Vec<Message> {
    vec![
        Message { from: "Jonas Berg", email: "jonas@northlight.io", subject: "Review of the onboarding flow", preview: "I went through all six screens on a 13 inch laptop. Step three scrolls before the button is visible.", time: "9:41", body: "Hi Maya,\n\nI went through all six screens on a 13 inch laptop. Step three scrolls before the button is visible, and the skip link reads like an error.\n\nCould we look at it together on Thursday at 10?\n\nJonas", folder: 0, unread: true, starred: true, label: Tone::Info as i32 },
        Message { from: "Priya Nair", email: "priya@harbor.dev", subject: "Invoice 2024-118", preview: "The October invoice is attached. Payment terms are 30 days as usual.", time: "8:15", body: "Hi,\n\nThe October invoice is attached. Payment terms are 30 days as usual. Let me know if the purchase order number needs to change.\n\nPriya", folder: 0, unread: true, starred: false, label: -1 },
        Message { from: "Tomas Rivera", email: "tomas@kiln.app", subject: "Export is slow for large projects", preview: "A 900 file project takes about four minutes to export on my machine.", time: "Yesterday", body: "A 900 file project takes about four minutes to export on my machine, most of it in the thumbnail step. I attached a trace.\n\nTomas", folder: 0, unread: false, starred: true, label: Tone::Warning as i32 },
        Message { from: "Lena Fischer", email: "lena@fischer.dev", subject: "Re: Release notes draft", preview: "Looks good. I tightened the second paragraph and fixed two links.", time: "Yesterday", body: "Looks good. I tightened the second paragraph and fixed two links that pointed at the old docs.\n\nLena", folder: 0, unread: false, starred: false, label: Tone::Success as i32 },
        Message { from: "Aiko Tanaka", email: "aiko@tanaka.design", subject: "Icons for the settings page", preview: "Here are the twelve icons at 16 and 20 px, outline only.", time: "Mon", body: "Here are the twelve icons at 16 and 20 px, outline only, on the same 1.5 px stroke as the rest of the set.\n\nAiko", folder: 0, unread: false, starred: false, label: Tone::Violet as i32 },
        Message { from: "Sam Whitfield", email: "sam@whitfield.co", subject: "Office closed on Friday", preview: "The building is closed for maintenance. Work from home if you can.", time: "Mon", body: "The building is closed for maintenance on Friday. Work from home if you can; the server room stays reachable over the VPN.\n\nSam", folder: 3, unread: false, starred: true, label: -1 },
    ]
}

fn matches(m: &Message, q: &str) -> bool {
    q.is_empty() || format!("{} {} {}", m.from, m.subject, m.preview).to_lowercase().contains(&q.to_lowercase())
}

fn in_folder(m: &Message, folder: i32) -> bool {
    if folder == 1 {
        m.starred
    } else {
        m.folder == folder
    }
}

fn folders(ui: &mut Ui, a: &mut MailApp, drawer: bool) {
    ui.column(
        &cui::Column {
            width: Size::FILL,
            height: Size::FILL,
            bg: if drawer { Color::TRANSPARENT } else { Color::SURFACE_MUTED },
            pad: if drawer { 0. } else { 12. },
            gap: 2.,
            ..Default::default()
        },
        |ui| {
            ui.row(&Row { pad_x: 6., pad_bottom: 12., gap: 10., ..Default::default() }, |ui| {
                ui.avatar_with("Maya Okafor", &Avatar { size: 28., ..Default::default() });
                ui.column(&cui::Column { gap: 0., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.text_with("Maya Okafor", &cui::Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                    ui.text_with(
                        "maya@fieldwork.studio",
                        &cui::Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() },
                    );
                });
            });
            if ui.button_with(
                "Compose",
                &Button { variant: ButtonVariant::Primary, icon: Some(cui::ICON_SQUARE_PEN), full_width: true, ..Default::default() },
            ) {
                a.compose = true;
                a.drawer = false;
            }
            ui.gap(10.);
            const NAMES: [&str; 4] = ["Inbox", "Starred", "Sent", "Archive"];
            const ICONS: [&str; 4] = [cui::ICON_INBOX, cui::ICON_STAR, cui::ICON_SEND, cui::ICON_ARCHIVE];
            for (f, name) in NAMES.iter().enumerate() {
                let unread = a.messages.iter().filter(|m| in_folder(m, f as i32) && m.unread).count();
                if ui.nav_item_with(
                    name,
                    &Nav {
                        icon: Some(ICONS[f]),
                        selected: a.folder == f as i32,
                        badge: (unread > 0).then(|| unread.to_string()).as_deref(),
                        ..Default::default()
                    },
                ) {
                    a.folder = f as i32;
                    a.selected = -1;
                    a.drawer = false;
                    a.reading = false;
                }
            }
            ui.spacer();
            ui.nav_section("Labels");
            const LABELS: [&str; 3] = ["Work", "Personal", "Urgent"];
            const TONES: [Tone; 3] = [Tone::Info, Tone::Violet, Tone::Warning];
            for (l, label) in LABELS.iter().enumerate() {
                ui.row(&Row { gap: 10., pad_x: 10., pad_y: 6., ..Default::default() }, |ui| {
                    ui.badge_with(label, &Badge { tone: TONES[l], ..Default::default() });
                });
            }
        },
    );
}

/// list rows: a quiet fill when selected or hovered, a hairline between the others
#[derive(Clone, Copy)]
struct RowBg {
    hovered: bool,
    selected: bool,
    last: bool,
}

fn paint_row(ui: &mut cui::CanvasUi, r: Rect, b: &RowBg) {
    let active = ui.token_color(Token::Active);
    let hover = ui.token_color(Token::Hover);
    let border = ui.token_color(Token::BorderSubtle);
    if b.selected {
        ui.draw_rect(r, Corners::all(10.), active);
    } else if b.hovered {
        ui.draw_rect(r, Corners::all(10.), hover);
    } else if !b.last {
        ui.draw_rect(
            Rect { x: r.x + 26., y: r.y + r.h - 1., w: r.w - 38., h: 1. },
            Corners::all(0.),
            border,
        );
    }
}

fn message_list(ui: &mut Ui, a: &mut MailApp) {
    const NAMES: [&str; 4] = ["Inbox", "Starred", "Sent", "Archive"];
    ui.column(
        &cui::Column { width: Size::FILL, height: Size::FILL, gap: 10., pad: 12., ..Default::default() },
        |ui| {
            ui.row(&Row { pad_x: 4., pad_top: 4., gap: 6., width: Size::FILL, ..Default::default() }, |ui| {
                if a.panes < 3 && ui.icon_button_with(cui::ICON_MENU, &Button { tooltip: Some("Folders"), ..Default::default() }) {
                    a.drawer = true;
                }
                ui.box_(&cui::BoxOpts { width: Size::FILL, ..Default::default() }, |ui| {
                    ui.heading(NAMES[a.folder as usize], 4);
                });
                if a.panes < 3 && ui.icon_button_with(cui::ICON_SQUARE_PEN, &Button { tooltip: Some("Compose"), ..Default::default() }) {
                    a.compose = true;
                }
            });
            ui.search_with(
                "search",
                &mut a.query,
                &Input { placeholder: Some("Search mail"), size: ControlSize::Sm, ..Default::default() },
            );
            ui.scroll(
                &cui::Scroll { id: Some("list"), gap: 0., fade_edges: true, ..Default::default() },
                |ui| {
                    let count = a.messages.len() as i32;
                    let mut shown = 0;
                    for i in 0..count {
                        let matches_msg = { matches(&a.messages[i as usize], &a.query) };
                        if !in_folder(&a.messages[i as usize], a.folder) || !matches_msg {
                            continue;
                        }
                        shown += 1;
                        let _g = ui.push_id_int(i);
                        let mut next = i + 1;
                        while next < count
                            && (!in_folder(&a.messages[next as usize], a.folder)
                                || !matches(&a.messages[next as usize], &a.query))
                        {
                            next += 1;
                        }
                        let bg_id = ui.make_id("bg");
                        let hover = ui.state::<bool>(bg_id);
                        let bg = RowBg { hovered: *hover, selected: a.selected == i, last: next >= count || a.selected == next };
                        let res: CanvasResult = ui.canvas("bg", Size::FILL, Size::FILL, paint_row, &bg);
                        *ui.state::<bool>(bg_id) = res.hovered;
                        if res.clicked {
                            a.selected = i;
                            a.messages[i as usize].unread = false;
                            a.reading = true;
                        }
                        // every row keeps the dot's gutter, so names line up read or unread
                        ui.row(
                            &Row { width: Size::FILL, pad_y: 12., pad_right: 12., align: Align::Start, ..Default::default() },
                            |ui| {
                                ui.box_(
                                    &cui::BoxOpts { width: Size::px(26.), pad_top: 6., align: Align::Center, ..Default::default() },
                                    |ui| {
                                        if a.messages[i as usize].unread {
                                            ui.box_(
                                                &cui::BoxOpts {
                                                    width: Size::px(7.),
                                                    height: Size::px(7.),
                                                    radius: 4.,
                                                    bg: Color::ACCENT,
                                                    ..Default::default()
                                                },
                                                |_| {},
                                            );
                                        }
                                    },
                                );
                                let m = &a.messages[i as usize];
                                ui.column(&cui::Column { gap: 3., width: Size::FILL, ..Default::default() }, |ui| {
                                    ui.row(&Row { gap: 8., width: Size::FILL, ..Default::default() }, |ui| {
                                        ui.text_with(
                                            m.from,
                                            &cui::Text {
                                                font: if m.unread { cui::font::MEDIUM } else { cui::font::BODY },
                                                nowrap: true,
                                                width: Size::FILL,
                                                ..Default::default()
                                            },
                                        );
                                        ui.text_with(
                                            m.time,
                                            &cui::Text { size: 12., color: Color::TEXT_MUTED, nowrap: true, ..Default::default() },
                                        );
                                    });
                                    ui.text_with(
                                        m.subject,
                                        &cui::Text { font: cui::font::MEDIUM, size: 12.5, nowrap: true, ..Default::default() },
                                    );
                                    ui.text_with(
                                        m.preview,
                                        &cui::Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() },
                                    );
                                });
                            },
                        );
                        let unread = a.messages[i as usize].unread;
                        let starred = a.messages[i as usize].starred;
                        ui.context_menu("row-menu", |ui| {
                            if ui.menu_item_with(
                                if unread { "Mark as read" } else { "Mark as unread" },
                                &MenuItem { icon: Some(cui::ICON_MAIL), ..Default::default() },
                            ) {
                                a.messages[i as usize].unread = !unread;
                            }
                            if ui.menu_item_with(
                                if starred { "Unstar" } else { "Star" },
                                &MenuItem { icon: Some(cui::ICON_STAR), ..Default::default() },
                            ) {
                                a.messages[i as usize].starred = !starred;
                            }
                            if ui.menu_item_with(
                                "Archive",
                                &MenuItem { icon: Some(cui::ICON_ARCHIVE), ..Default::default() },
                            ) {
                                a.messages[i as usize].folder = 3;
                            }
                        });
                    }
                    if shown == 0 {
                        ui.empty_state_with(
                            "No messages",
                            &EmptyState { icon: Some(cui::ICON_INBOX), description: Some("You are all caught up."), ..Default::default() },
                        );
                    }
                },
            );
        },
    );
}

fn reading_pane(ui: &mut Ui, a: &mut MailApp) {
    ui.column(&cui::Column { width: Size::FILL, height: Size::FILL, ..Default::default() }, |ui| {
        if a.selected < 0 || a.selected as usize >= a.messages.len() {
            ui.column(
                &cui::Column { width: Size::FILL, height: Size::FILL, justify: Justify::Center, ..Default::default() },
                |ui| {
                    ui.empty_state_with(
                        "No message selected",
                        &EmptyState { icon: Some(cui::ICON_MAIL), description: Some("Choose a message to read it here."), ..Default::default() },
                    );
                },
            );
            return;
        }
        let idx = a.selected as usize;
        ui.row(
            &Row { gap: 4., pad_x: if a.panes == 1 { 12. } else { 20. }, pad_y: 10., ..Default::default() },
            |ui| {
                if a.panes == 1
                    && ui.icon_button_with(
                        cui::ICON_CHEVRON_LEFT,
                        &Button { tooltip: Some("Back to the list"), ..Default::default() },
                    )
                {
                    a.reading = false;
                }
                if ui.icon_button_with(
                    cui::ICON_ARCHIVE,
                    &Button { tooltip: Some("Archive"), shortcut: Some("E"), ..Default::default() },
                ) {
                    a.messages[idx].folder = 3;
                    a.selected = -1;
                    ui.toast_with("Archived", &cui::Toast { action: Some("Undo"), ..Default::default() });
                }
                ui.icon_button_with(cui::ICON_TRASH_2, &Button { tooltip: Some("Delete"), ..Default::default() });
                let starred = a.messages[idx].starred;
                if ui.icon_button_with(
                    cui::ICON_STAR,
                    &Button {
                        tooltip: Some(if starred { "Unstar" } else { "Star" }),
                        selected: starred,
                        color: if starred { Color::WARNING } else { Color::TRANSPARENT },
                        ..Default::default()
                    },
                ) {
                    a.messages[idx].starred = !starred;
                }
                ui.spacer();
                if ui.button_with("Reply", &Button { icon: Some(cui::ICON_UNDO_2), size: ControlSize::Sm, ..Default::default() }) {
                    a.compose = true;
                    a.to = a.messages[idx].email.to_string();
                    a.subject = format!("Re: {}", a.messages[idx].subject);
                }
            },
        );
        ui.separator();
        if a.selected < 0 {
            return;
        }
        let m = &a.messages[idx];
        ui.scroll(
            &cui::Scroll {
                id: Some("reader"),
                pad: if a.panes == 1 { 16. } else { 28. },
                gap: 18.,
                fade_edges: true,
                ..Default::default()
            },
            |ui| {
                ui.heading(m.subject, 3);
                ui.row(&Row { gap: 12., ..Default::default() }, |ui| {
                    ui.avatar_with(m.from, &Avatar { size: 36., ..Default::default() });
                    ui.column(&cui::Column { gap: 1., width: Size::FILL, ..Default::default() }, |ui| {
                        ui.text_with(m.from, &cui::Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                        ui.text_with(m.email, &cui::Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() });
                    });
                    if m.label >= 0 {
                        let label = match m.label {
                            l if l == Tone::Info as i32 => "Work",
                            l if l == Tone::Violet as i32 => "Personal",
                            l if l == Tone::Warning as i32 => "Urgent",
                            _ => "Merged",
                        };
                        ui.badge_with(label, &Badge { tone: Tone::from_index(m.label), ..Default::default() });
                    }
                    ui.text_with(
                        m.time,
                        &cui::Text { size: 12., color: Color::TEXT_MUTED, nowrap: true, ..Default::default() },
                    );
                });
                ui.text_with(m.body, &cui::Text { line_height: 1.6, ..Default::default() });
            },
        );
    });
}

fn frame(ui: &mut Ui, a: &mut MailApp) {
    if !ui.wants_keyboard() && ui.key_pressed(cui::Key::C) {
        a.compose = true;
    }
    let vw = ui.viewport_size().x;
    a.panes = if vw < 720. { 1 } else if vw < 1100. { 2 } else { 3 };
    if a.panes == 3 {
        a.drawer = false;
    }
    ui.box_(
        &cui::BoxOpts { width: Size::FILL, height: Size::FILL, bg: Color::BACKGROUND, ..Default::default() },
        |ui| {
            if a.panes == 3 {
                let mut split1 = a.split1;
                let mut split2 = a.split2;
                ui.split(
                    "outer",
                    &mut split1,
                    &cui::Split { min_first: 200., min_second: 600., ..Default::default() },
                    |ui| {
                        folders(ui, a, false);
                        ui.split_next();
                        ui.split(
                            "inner",
                            &mut split2,
                            &cui::Split { min_first: 280., min_second: 320., ..Default::default() },
                            |ui| {
                                message_list(ui, a);
                                ui.split_next();
                                reading_pane(ui, a);
                            },
                        );
                    },
                );
                a.split1 = split1;
                a.split2 = split2;
            } else if a.panes == 2 {
                let mut split2 = a.split2;
                ui.split(
                    "inner",
                    &mut split2,
                    &cui::Split { min_first: 260., min_second: 320., ..Default::default() },
                    |ui| {
                        message_list(ui, a);
                        ui.split_next();
                        reading_pane(ui, a);
                    },
                );
                a.split2 = split2;
            } else if a.reading && a.selected >= 0 {
                reading_pane(ui, a);
            } else {
                message_list(ui, a);
            }
        },
    );
    let mut drawer = a.drawer;
    ui.sheet(
        "Mail",
        &mut drawer,
        &cui::Sheet { side: cui::Side::Left, size: 280., ..Default::default() },
        |ui| folders(ui, a, true),
    );
    a.drawer = drawer;
    #[derive(PartialEq)]
    enum Compose {
        None,
        Discard,
        Send,
    }
    let mut compose = a.compose;
    let mut action = Compose::None;
    ui.dialog_with("New message", &mut compose, &Dialog { width: 560., ..Default::default() }, |ui| {
        ui.input_with("to", &mut a.to, &Input { label: Some("To"), icon: Some(cui::ICON_AT_SIGN), autofocus: true, ..Default::default() });
        ui.input_with("subject", &mut a.subject, &Input { label: Some("Subject"), ..Default::default() });
        ui.textarea_with(
            "body",
            &mut a.body,
            &Input { height: 180., placeholder: Some("Write your message"), ..Default::default() },
        );
        ui.dialog_footer(|ui| {
            if ui.button_with("Discard", &Button { variant: ButtonVariant::Ghost, ..Default::default() }) {
                action = Compose::Discard;
            }
            if ui.button_with(
                "Send",
                &Button { variant: ButtonVariant::Primary, icon: Some(cui::ICON_SEND), disabled: a.to.is_empty(), ..Default::default() },
            ) {
                action = Compose::Send;
            }
        });
    });
    match action {
        Compose::Discard => {
            a.compose = false;
            a.to.clear();
            a.subject.clear();
            a.body.clear();
        }
        Compose::Send => {
            a.compose = false;
            ui.toast_with(
                "Message sent",
                &cui::Toast {
                    description: if a.subject.is_empty() { Some("No subject") } else { Some(a.subject.as_str()) },
                    tone: Tone::Success,
                    ..Default::default()
                },
            );
            a.to.clear();
            a.subject.clear();
            a.body.clear();
        }
        Compose::None => {}
    }
    a.compose = compose;
}

fn main() {
    let mut a = MailApp {
        messages: build_messages(),
        folder: 0,
        selected: 0,
        split1: 0.2,
        split2: 0.38,
        ..Default::default()
    };
    a.messages[0].unread = false;
    let code = App::run(
        &AppDesc { title: Some("Mail"), width: 1280, height: 800, accent: cui::Accent::Blue, ..Default::default() },
        |ui| frame(ui, &mut a),
    );
    std::process::exit(code);
}
