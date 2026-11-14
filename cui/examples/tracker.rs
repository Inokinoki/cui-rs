//! Issue tracker — a 1:1 port of examples/tracker.c from the C repo.
//! Grouped issue list, custom status and priority glyphs, a detail pane with
//! properties and activity. Dark by default.

use cui::{
    Align, App, AppDesc, Avatar, Badge, Button, ButtonVariant, Card, Color, Command, ControlSize,
    Corners, Input, Justify, Menu, MenuItem, Nav, Rect, Row, Size, TextAlign, Tone, Ui, Vec2,
};
use std::f32::consts::TAU;

const ST_BACKLOG: i32 = 0;
const ST_TODO: i32 = 1;
const ST_PROGRESS: i32 = 2;
const ST_REVIEW: i32 = 3;
const ST_DONE: i32 = 4;
const ST_COUNT: usize = 5;
const PRI_NONE: i32 = 0;
const PRI_LOW: i32 = 1;
const PRI_MEDIUM: i32 = 2;
const PRI_HIGH: i32 = 3;
const PRI_URGENT: i32 = 4;

struct Issue {
    key: &'static str,
    title: &'static str,
    assignee: &'static str,
    date: &'static str,
    status: i32,
    priority: i32,
    labels: [&'static str; 2],
    label_tones: [Tone; 2],
}

fn issues() -> &'static [Issue] {
    static ISSUES: std::sync::OnceLock<Vec<Issue>> = std::sync::OnceLock::new();
    ISSUES.get_or_init(|| {
        vec![
            Issue { key: "ENG-214", title: "Text input drops the last character on fast typing", assignee: "Maya Okafor", date: "Nov 6", status: ST_PROGRESS, priority: PRI_URGENT, labels: ["Bug", "Input"], label_tones: [Tone::Danger, Tone::Neutral] },
            Issue { key: "ENG-209", title: "Spring animations for popover placement flips", assignee: "Jonas Berg", date: "Nov 8", status: ST_PROGRESS, priority: PRI_HIGH, labels: ["Motion", ""], label_tones: [Tone::Violet, Tone::Neutral] },
            Issue { key: "ENG-205", title: "Software renderer: SIMD path for text quads", assignee: "Lena Fischer", date: "Nov 12", status: ST_PROGRESS, priority: PRI_MEDIUM, labels: ["Perf", ""], label_tones: [Tone::Teal, Tone::Neutral] },
            Issue { key: "ENG-198", title: "Keyboard navigation for the color picker", assignee: "Priya Nair", date: "Nov 7", status: ST_REVIEW, priority: PRI_HIGH, labels: ["A11y", "Input"], label_tones: [Tone::Info, Tone::Neutral] },
            Issue { key: "ENG-196", title: "Wayland backend through libdecor", assignee: "Tomas Rivera", date: "Nov 9", status: ST_REVIEW, priority: PRI_MEDIUM, labels: ["Platform", ""], label_tones: [Tone::Rose, Tone::Neutral] },
            Issue { key: "ENG-221", title: "Inline IME composition on macOS", assignee: "Aiko Tanaka", date: "Nov 14", status: ST_TODO, priority: PRI_HIGH, labels: ["Platform", "Input"], label_tones: [Tone::Rose, Tone::Neutral] },
            Issue { key: "ENG-219", title: "Virtual table with sticky header", assignee: "Sam Whitfield", date: "Nov 18", status: ST_TODO, priority: PRI_MEDIUM, labels: ["Data", ""], label_tones: [Tone::Teal, Tone::Neutral] },
            Issue { key: "ENG-217", title: "Docs: theming cookbook", assignee: "Maya Okafor", date: "Nov 20", status: ST_TODO, priority: PRI_LOW, labels: ["Docs", ""], label_tones: [Tone::Neutral, Tone::Neutral] },
            Issue { key: "ENG-188", title: "Ship 1.0 release notes", assignee: "Jonas Berg", date: "Nov 1", status: ST_DONE, priority: PRI_MEDIUM, labels: ["Docs", ""], label_tones: [Tone::Neutral, Tone::Neutral] },
            Issue { key: "ENG-185", title: "Gliding highlight in segmented controls", assignee: "Priya Nair", date: "Oct 30", status: ST_DONE, priority: PRI_LOW, labels: ["Motion", ""], label_tones: [Tone::Violet, Tone::Neutral] },
        ]
    })
}

const STATUS_NAMES: [&str; ST_COUNT] = ["Backlog", "Todo", "In progress", "In review", "Done"];
const PRIORITY_NAMES: [&str; 5] = ["No priority", "Low", "Medium", "High", "Urgent"];

#[derive(Default)]
#[allow(dead_code)] // parity with the C example, which keeps spare fields
struct AppState {
    nav: i32,
    tab: i32,
    selected: usize,
    collapsed: [bool; ST_COUNT],
    palette: bool,
    comment: String,
    subscribed: bool,
    panes: i32,           // 3, 2 or 1, from the window width
    drawer: bool,         // the sidebar when it does not fit
    sheet: bool,          // the detail when it does not fit
}

// --- custom glyphs --------------------------------------------------------

#[derive(Clone, Copy)]
struct StatusGlyph {
    status: i32,
}

fn paint_status(ui: &mut cui::CanvasUi, r: Rect, g: &StatusGlyph) {
    let c = Vec2 { x: r.x + r.w / 2., y: r.y + r.h / 2. };
    let rad = 6.5f32;
    let muted = ui.token_color(cui::Token::TextMuted);
    let warn = ui.token_color(cui::Token::Warning);
    let accent = ui.token_color(cui::Token::Accent);
    match g.status {
        ST_BACKLOG => {
            // dashed ring
            for i in 0..8 {
                ui.draw_arc(c, rad, TAU * i as f32 / 8. + 0.12, TAU * (i + 1) as f32 / 8. - 0.12, 1.5, muted);
            }
        }
        ST_TODO => {
            let secondary = ui.token_color(cui::Token::TextSecondary);
            ui.draw_arc(c, rad, 0., TAU, 1.5, secondary);
        }
        ST_PROGRESS | ST_REVIEW => {
            let col = if g.status == ST_PROGRESS { warn } else { accent };
            let frac = if g.status == ST_PROGRESS { 0.5 } else { 0.75 };
            ui.draw_arc(c, rad, 0., TAU, 1.5, col);
            let mut pts = vec![c];
            for i in 0..=32 {
                let a = -TAU / 4. + TAU * frac * i as f32 / 32.;
                pts.push(Vec2 { x: c.x + a.cos() * (rad - 3.), y: c.y + a.sin() * (rad - 3.) });
            }
            // the pie is not convex for frac > 0.5: draw it as two halves
            if frac <= 0.5 {
                ui.draw_polygon(&pts, col);
            } else {
                ui.draw_polygon(&pts[..18], col);
                let mut rest = [c; 18];
                rest[1..18].copy_from_slice(&pts[17..34]);
                ui.draw_polygon(&rest, col);
            }
        }
        _ => {
            // ST_DONE
            let accent_fg = ui.token_color(cui::Token::AccentForeground);
            ui.draw_circle(c, rad + 0.75, accent);
            ui.draw_icon(cui::ICON_CHECK, r, 10., accent_fg);
        }
    }
}

fn status_icon(ui: &mut Ui, id: &str, status: i32) {
    let g = StatusGlyph { status };
    ui.canvas(id, Size::px(16.), Size::px(16.), paint_status, &g);
}

#[derive(Clone, Copy)]
struct PriorityGlyph {
    priority: i32,
}

fn paint_priority(ui: &mut cui::CanvasUi, r: Rect, g: &PriorityGlyph) {
    let on = ui.token_color(cui::Token::TextSecondary);
    let off = on.with_alpha(0.3);
    if g.priority == PRI_URGENT {
        let warn = ui.token_color(cui::Token::Warning);
        let b = Rect { x: r.x + 1., y: r.y + 1., w: r.w - 2., h: r.h - 2. };
        ui.draw_rect(b, Corners::all(3.5), warn);
        ui.draw_rect(
            Rect { x: b.x + b.w / 2. - 0.9, y: b.y + 3., w: 1.8, h: 5.5 },
            Corners::all(0.9),
            Color::rgb(0x1a, 0x1a, 0x1a),
        );
        ui.draw_rect(
            Rect { x: b.x + b.w / 2. - 0.9, y: b.y + b.h - 4.6, w: 1.8, h: 1.8 },
            Corners::all(0.9),
            Color::rgb(0x1a, 0x1a, 0x1a),
        );
        return;
    }
    if g.priority == PRI_NONE {
        for i in 0..3 {
            ui.draw_rect(
                Rect { x: r.x + 2. + i as f32 * 4.5, y: r.y + r.h / 2. - 0.75, w: 3., h: 1.5 },
                Corners::all(0.75),
                off,
            );
        }
        return;
    }
    for i in 0..3 {
        let h = 4. + i as f32 * 3.5;
        ui.draw_rect(
            Rect { x: r.x + 2. + i as f32 * 4.5, y: r.y + r.h - 2. - h, w: 3., h },
            Corners::all(1.),
            if i < g.priority { on } else { off },
        );
    }
}

fn priority_icon(ui: &mut Ui, id: &str, priority: i32) {
    let g = PriorityGlyph { priority };
    ui.canvas(id, Size::px(16.), Size::px(16.), paint_priority, &g);
}

// A full-width hover/selection background behind a row of components.
#[derive(Clone, Copy)]
struct RowBg {
    hovered: bool,
    selected: bool,
    radius: f32,
}

fn paint_row(ui: &mut cui::CanvasUi, r: Rect, b: &RowBg) {
    let active = ui.token_color(cui::Token::Active);
    let hover = ui.token_color(cui::Token::Hover);
    if b.selected {
        ui.draw_rect(r, Corners::all(b.radius), active);
    } else if b.hovered {
        ui.draw_rect(r, Corners::all(b.radius), hover);
    }
}

fn row_background(ui: &mut Ui, id: &str, selected: bool, radius: f32) -> bool {
    let rid = ui.make_id(id);
    let hover = ui.state::<bool>(rid);
    let bg = RowBg { hovered: *hover, selected, radius };
    let res = ui.canvas(id, Size::FILL, Size::FILL, paint_row, &bg);
    *ui.state::<bool>(rid) = res.hovered;
    res.clicked
}

#[allow(dead_code)] // the C original keeps this helper for team colors
fn team_dot(ui: &mut Ui, c: Color) {
    ui.box_(
        &cui::BoxOpts {
            width: Size::px(16.),
            height: Size::px(16.),
            radius: 5.,
            bg: c,
            dir: cui::Dir::Stack,
            align: Align::Center,
            justify: Justify::Center,
            ..Default::default()
        },
        |_| {},
    );
}

// --- panes ------------------------------------------------------------------

fn sidebar(ui: &mut Ui, s: &mut AppState, drawer: bool) {
    ui.column(
        &cui::Column {
            width: if drawer { Size::FILL } else { Size::px(240.) },
            height: Size::FILL,
            bg: if drawer { Color::TRANSPARENT } else { Color::SURFACE_MUTED },
            pad: if drawer { 0. } else { 12. },
            gap: 2.,
            ..Default::default()
        },
        |ui| {
            ui.row(&Row { gap: 10., pad_x: 6., pad_top: 4., pad_bottom: 12., align: Align::Center, ..Default::default() }, |ui| {
                ui.box_(
                    &cui::BoxOpts {
                        width: Size::px(26.),
                        height: Size::px(26.),
                        radius: 8.,
                        bg: Color::ACCENT,
                        dir: cui::Dir::Stack,
                        align: Align::Center,
                        justify: Justify::Center,
                        ..Default::default()
                    },
                    |ui| ui.icon(cui::ICON_TARGET, 16., Color::ACCENT_FOREGROUND),
                );
                ui.text_with("Orbit", &cui::Text { font: cui::font::MEDIUM, size: 15., width: Size::FILL, nowrap: true, ..Default::default() });
                ui.icon_button_with(cui::ICON_SQUARE_PEN, &Button { size: ControlSize::Sm, tooltip: Some("New issue"), ..Default::default() });
            });
            const MAIN_ITEMS: [&str; 3] = ["Inbox", "My issues", "Views"];
            const MAIN_ICONS: [&str; 3] = [cui::ICON_INBOX, cui::ICON_CIRCLE_USER, cui::ICON_LAYERS];
            const MAIN_BADGES: [&str; 3] = ["4", "", ""];
            for (i, item) in MAIN_ITEMS.iter().enumerate() {
                if ui.nav_item_with(
                    item,
                    &Nav {
                        icon: Some(MAIN_ICONS[i]),
                        badge: if MAIN_BADGES[i].is_empty() { None } else { Some(MAIN_BADGES[i]) },
                        selected: s.nav == i as i32,
                        group: Some("nav"),
                    },
                ) {
                    s.nav = i as i32;
                    s.drawer = false;
                }
            }
            ui.box_(&cui::BoxOpts { height: Size::px(12.), ..Default::default() }, |_| {});
            ui.nav_section("Workspace");
            const WS_ITEMS: [&str; 3] = ["Projects", "Cycles", "Roadmap"];
            const WS_ICONS: [&str; 3] = [cui::ICON_BOX, cui::ICON_REFRESH_CW, cui::ICON_MAP_PIN];
            for (i, item) in WS_ITEMS.iter().enumerate() {
                if ui.nav_item_with(
                    item,
                    &Nav { icon: Some(WS_ICONS[i]), selected: s.nav == 3 + i as i32, group: Some("nav"), ..Default::default() },
                ) {
                    s.nav = 3 + i as i32;
                    s.drawer = false;
                }
            }
            ui.box_(&cui::BoxOpts { height: Size::px(12.), ..Default::default() }, |_| {});
            ui.nav_section("Teams");
            const TEAMS: [&str; 3] = ["Engine", "Design", "Platform"];
            const TEAM_ICONS: [&str; 3] = [cui::ICON_CPU, cui::ICON_PALETTE, cui::ICON_SERVER];
            for (i, team) in TEAMS.iter().enumerate() {
                if ui.nav_item_with(
                    team,
                    &Nav { icon: Some(TEAM_ICONS[i]), selected: s.nav == 6 + i as i32, group: Some("nav"), ..Default::default() },
                ) {
                    s.nav = 6 + i as i32;
                }
            }
            ui.spacer();
            ui.card(&Card { pad: 14., gap: 10., ..Default::default() }, |ui| {
                ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                    ui.icon(cui::ICON_ZAP, 14., Color::ACCENT);
                    ui.text_with("Cycle 24", &cui::Text { font: cui::font::MEDIUM, size: 13., width: Size::FILL, ..Default::default() });
                    ui.text_with("6 days left", &cui::Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
                });
                ui.progress_with(0.64, &cui::Progress { height: 6., ..Default::default() });
                ui.text_with("32 of 50 issues done", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.box_(&cui::BoxOpts { height: Size::px(8.), ..Default::default() }, |_| {});
            ui.row(&Row { gap: 10., pad_x: 6., pad_y: 4., align: Align::Center, ..Default::default() }, |ui| {
                ui.avatar_with("Maya Okafor", &Avatar { size: 28., status: Some("online"), ..Default::default() });
                ui.column(&cui::Column { gap: 0., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.text_with("Maya Okafor", &cui::Text { font: cui::font::MEDIUM, size: 13., nowrap: true, ..Default::default() });
                    ui.text_with("maya@orbit.dev", &cui::Text { size: 12., color: Color::TEXT_MUTED, nowrap: true, ..Default::default() });
                });
                ui.icon_button_with(cui::ICON_SETTINGS, &Button { size: ControlSize::Sm, tooltip: Some("Settings"), ..Default::default() });
            });
        },
    );
}

fn issue_row(ui: &mut Ui, s: &mut AppState, i: usize) {
    let it = &issues()[i];
    let _g = ui.push_id_int(i as i32);
    ui.box_(
        &cui::BoxOpts { dir: cui::Dir::Stack, width: Size::FILL, height: Size::px(44.), ..Default::default() },
        |ui| {
            if row_background(ui, "bg", s.selected == i, 8.) {
                s.selected = i;
                s.sheet = s.panes < 3;
            }
            ui.row(
                &Row { width: Size::FILL, height: Size::FILL, pad_x: 12., gap: 12., align: Align::Center, ..Default::default() },
                |ui| {
                    priority_icon(ui, "prio", it.priority);
                    ui.text_with(
                        it.key,
                        &cui::Text { font: cui::font::MONO, size: 12., color: Color::TEXT_MUTED, width: Size::px(62.), nowrap: true, ..Default::default() },
                    );
                    status_icon(ui, "status", it.status);
                    ui.text_with(it.title, &cui::Text { width: Size::FILL, nowrap: true, ..Default::default() });
                    if s.panes > 1 {
                        // a phone keeps the title, the status and who has it
                        for (l, label) in it.labels.iter().enumerate() {
                            if !label.is_empty() {
                                ui.badge_with(label, &Badge { tone: it.label_tones[l], ..Default::default() });
                            }
                        }
                        ui.text_with(
                            it.date,
                            &cui::Text { size: 12., color: Color::TEXT_MUTED, width: Size::px(46.), align: TextAlign::Right, nowrap: true, ..Default::default() },
                        );
                    }
                    ui.avatar_with(it.assignee, &Avatar { size: 22., ..Default::default() });
                },
            );
        },
    );
}

fn issue_list(ui: &mut Ui, s: &mut AppState) {
    ui.column(&cui::Column { width: Size::FILL, height: Size::FILL, ..Default::default() }, |ui| {
        let phone = s.panes == 1;
        ui.row(
            &Row {
                width: Size::FILL,
                pad_x: if phone { 12. } else { 24. },
                pad_y: 14.,
                gap: if phone { 6. } else { 12. },
                align: Align::Center,
                ..Default::default()
            },
            |ui| {
                if phone && ui.icon_button_with(cui::ICON_MENU, &Button { tooltip: Some("Menu"), ..Default::default() }) {
                    s.drawer = true;
                }
                const CRUMBS: [&str; 2] = ["Engine", "Issues"];
                ui.breadcrumb(&CRUMBS);
                ui.spacer();
                if phone {
                    ui.icon_button_with(cui::ICON_FILTER, &Button { tooltip: Some("Filter"), ..Default::default() });
                    ui.icon_button_with(
                        cui::ICON_PLUS,
                        &Button { variant: ButtonVariant::Primary, size: ControlSize::Sm, tooltip: Some("New issue"), ..Default::default() },
                    );
                } else {
                    ui.button_with("Filter", &Button { icon: Some(cui::ICON_FILTER), size: ControlSize::Sm, variant: ButtonVariant::Ghost, ..Default::default() });
                    ui.button_with("Display", &Button { icon: Some(cui::ICON_SLIDERS_HORIZONTAL), size: ControlSize::Sm, variant: ButtonVariant::Ghost, ..Default::default() });
                    ui.button_with("New issue", &Button { icon: Some(cui::ICON_PLUS), size: ControlSize::Sm, variant: ButtonVariant::Primary, ..Default::default() });
                }
            },
        );
        ui.row(
            &Row { width: Size::FILL, pad_x: if phone { 12. } else { 24. }, pad_bottom: 12., align: Align::Center, ..Default::default() },
            |ui| {
                const TABS: [&str; 3] = ["All issues", "Active", "Backlog"];
                ui.tabs_with("views", &TABS, &mut s.tab, &cui::Tabs { variant: cui::TabsVariant::Pill, ..Default::default() });
                ui.spacer();
                const PEOPLE: [&str; 5] = ["Maya Okafor", "Jonas Berg", "Priya Nair", "Lena Fischer", "Aiko Tanaka"];
                ui.avatar_group(&PEOPLE, 4, 24.);
            },
        );
        ui.separator();
        ui.scroll(
            &cui::Scroll { id: Some("issues"), pad: if phone { 4. } else { 12. }, gap: 2., fade_edges: true, ..Default::default() },
            |ui| {
                const ORDER: [i32; 4] = [ST_PROGRESS, ST_REVIEW, ST_TODO, ST_DONE];
                for (g, st) in ORDER.iter().enumerate() {
                    let count = issues().iter().filter(|it| it.status == *st).count();
                    let _g = ui.push_id_int(100 + st);
                    ui.row(
                        &Row {
                            width: Size::FILL,
                            pad_x: 12.,
                            pad_top: if g > 0 { 14. } else { 4. },
                            pad_bottom: 6.,
                            gap: 10.,
                            align: Align::Center,
                            ..Default::default()
                        },
                        |ui| {
                            status_icon(ui, "group", *st);
                            ui.text_with(
                                STATUS_NAMES[*st as usize],
                                &cui::Text { font: cui::font::MEDIUM, size: 13., nowrap: true, ..Default::default() },
                            );
                            ui.text_with(count.to_string(), &cui::Text { size: 13., color: Color::TEXT_MUTED, ..Default::default() });
                            ui.spacer();
                            ui.icon_button_with(cui::ICON_PLUS, &Button { size: ControlSize::Sm, tooltip: Some("Add issue"), ..Default::default() });
                        },
                    );
                    for (i, it) in issues().iter().enumerate() {
                        if it.status == *st {
                            issue_row(ui, s, i);
                        }
                    }
                }
            },
        );
    });
}

fn property(ui: &mut Ui, name: &str) {
    ui.text_with(
        name,
        &cui::Text { size: 13., color: Color::TEXT_SECONDARY, width: Size::px(96.), nowrap: true, ..Default::default() },
    );
}

fn detail(ui: &mut Ui, s: &mut AppState, in_sheet: bool) {
    let idx = s.selected.min(issues().len() - 1);
    let it = &issues()[idx];
    ui.column(
        &cui::Column {
            width: if in_sheet { Size::FILL } else { Size::px(380.) },
            height: Size::FILL,
            bg: if in_sheet { Color::TRANSPARENT } else { Color::SURFACE },
            ..Default::default()
        },
        |ui| {
            if !in_sheet {
                ui.row(&Row { width: Size::FILL, pad_x: 20., pad_y: 14., gap: 6., align: Align::Center, ..Default::default() }, |ui| {
                    ui.text_with(it.key, &cui::Text { font: cui::font::MONO, size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
                    ui.copy_button("copy", it.key);
                    ui.spacer();
                    if ui.icon_button_with(
                        cui::ICON_BELL,
                        &Button { size: ControlSize::Sm, tooltip: Some(if s.subscribed { "Unsubscribe" } else { "Subscribe" }), ..Default::default() },
                    ) {
                        s.subscribed = !s.subscribed;
                    }
                    ui.menu_with(
                        "More",
                        &Menu { icon: Some(cui::ICON_ELLIPSIS), icon_only: true, variant: ButtonVariant::Ghost, placement: cui::Placement::BottomEnd, ..Default::default() },
                        |ui| {
                            ui.menu_item_with("Copy link", &MenuItem { icon: Some(cui::ICON_LINK), shortcut: Some("Ctrl L"), ..Default::default() });
                            ui.menu_item_with("Duplicate", &MenuItem { icon: Some(cui::ICON_COPY), ..Default::default() });
                            ui.menu_separator();
                            ui.menu_item_with("Delete issue", &MenuItem { icon: Some(cui::ICON_TRASH_2), danger: true, ..Default::default() });
                        },
                    );
                });
            }
            if !in_sheet {
                ui.separator();
            }
            ui.scroll(
                &cui::Scroll { id: Some("detail"), pad: if in_sheet { 0. } else { 20. }, gap: 18., fade_edges: true, ..Default::default() },
                |ui| {
                    ui.text_with(it.title, &cui::Text { font: cui::font::DISPLAY, size: 20., line_height: 1.3, ..Default::default() });
                    ui.text_with(
                        "Typing quickly into a focused field sometimes loses the final keystroke when the frame that receives it also changes focus. Repro: type and press Tab in the same frame.",
                        &cui::Text { color: Color::TEXT_SECONDARY, ..Default::default() },
                    );
                    ui.column(&cui::Column { gap: 12., width: Size::FILL, ..Default::default() }, |ui| {
                        ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                            property(ui, "Status");
                            status_icon(ui, "d-status", it.status);
                            ui.text(STATUS_NAMES[it.status as usize]);
                        });
                        ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                            property(ui, "Priority");
                            priority_icon(ui, "d-prio", it.priority);
                            ui.text(PRIORITY_NAMES[it.priority as usize]);
                        });
                        ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                            property(ui, "Assignee");
                            ui.avatar_with(it.assignee, &Avatar { size: 20., ..Default::default() });
                            ui.text(it.assignee);
                        });
                        ui.row(&Row { gap: 6., align: Align::Center, ..Default::default() }, |ui| {
                            property(ui, "Labels");
                            for (l, label) in it.labels.iter().enumerate() {
                                if !label.is_empty() {
                                    ui.badge_with(label, &Badge { tone: it.label_tones[l], ..Default::default() });
                                }
                            }
                        });
                        ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                            property(ui, "Cycle");
                            ui.icon(cui::ICON_REFRESH_CW, 14., Color::TEXT_SECONDARY);
                            ui.text("Cycle 24");
                        });
                        ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                            property(ui, "Due date");
                            ui.icon(cui::ICON_CALENDAR, 14., Color::TEXT_SECONDARY);
                            ui.text(it.date);
                        });
                    });
                    ui.separator();
                    ui.label("Activity");
                    ui.column(&cui::Column { gap: 0., width: Size::FILL, ..Default::default() }, |ui| {
                        ui.timeline_item_with(
                            "Maya moved this to In progress",
                            &cui::TimelineItem { time: Some("2h ago"), icon: Some(cui::ICON_ARROW_RIGHT), tone: Tone::Warning, ..Default::default() },
                        );
                        ui.timeline_item_with(
                            "Jonas linked a pull request",
                            &cui::TimelineItem {
                                time: Some("5h ago"),
                                icon: Some(cui::ICON_GIT_BRANCH),
                                tone: Tone::Accent,
                                description: Some("Keep pending text across focus changes"),
                                ..Default::default()
                            },
                        );
                        ui.timeline_item_with(
                            "Priya set the priority to Urgent",
                            &cui::TimelineItem { time: Some("Yesterday"), icon: Some(cui::ICON_CIRCLE_ALERT), tone: Tone::Warning, ..Default::default() },
                        );
                        ui.timeline_item_with(
                            "Issue created",
                            &cui::TimelineItem { time: Some("Nov 2"), icon: Some(cui::ICON_PLUS), last: true, ..Default::default() },
                        );
                    });
                },
            );
            ui.separator();
            ui.row(
                &Row { width: Size::FILL, pad: if in_sheet { 0. } else { 16. }, pad_top: 16., gap: 8., align: Align::Center, ..Default::default() },
                |ui| {
                    ui.input_with(
                        "comment",
                        &mut s.comment,
                        &Input { placeholder: Some("Leave a comment"), width: Size::FILL, ..Default::default() },
                    );
                    ui.icon_button_with(cui::ICON_SEND, &Button { variant: ButtonVariant::Primary, tooltip: Some("Send"), ..Default::default() });
                },
            );
        },
    );
}

fn frame(ui: &mut Ui, s: &mut AppState) {
    if ui.shortcut(cui::Mods::PRIMARY, cui::Key::K) {
        s.palette = true;
    }
    let vw = ui.viewport_size().x;
    s.panes = if vw < 760. { 1 } else if vw < 1200. { 2 } else { 3 };
    if s.panes > 1 {
        s.drawer = false;
    }
    if s.panes == 3 {
        s.sheet = false;
    }
    ui.row(
        &Row { width: Size::FILL, height: Size::FILL, align: Align::Stretch, bg: Color::BACKGROUND, ..Default::default() },
        |ui| {
            if s.panes > 1 {
                sidebar(ui, s, false);
                ui.separator();
            }
            issue_list(ui, s);
            if s.panes == 3 {
                ui.separator();
                detail(ui, s, false);
            }
        },
    );
    let mut drawer = s.drawer;
    ui.sheet(
        "Orbit",
        &mut drawer,
        &cui::Sheet { side: cui::Side::Left, size: 280., ..Default::default() },
        |ui| sidebar(ui, s, true),
    );
    s.drawer = drawer;
    let mut sheet = s.sheet;
    ui.sheet(issues()[s.selected].key, &mut sheet, &cui::Sheet { size: if s.panes == 1 { vw } else { 420. }, ..Default::default() }, |ui| {
        detail(ui, s, true);
    });
    s.sheet = sheet;
    const CMDS: [Command; 5] = [
        Command { title: "Create issue", icon: Some(cui::ICON_PLUS), group: Some("Issues"), shortcut: Some("C") },
        Command { title: "Assign to me", icon: Some(cui::ICON_USER), group: Some("Issues"), shortcut: Some("I") },
        Command { title: "Change status", icon: Some(cui::ICON_CIRCLE_DOT), group: Some("Issues"), shortcut: Some("S") },
        Command { title: "Go to inbox", icon: Some(cui::ICON_INBOX), group: Some("Navigation"), shortcut: Some("G I") },
        Command { title: "Go to cycles", icon: Some(cui::ICON_REFRESH_CW), group: Some("Navigation"), shortcut: Some("G C") },
    ];
    let chosen = ui.command_palette(&mut s.palette, &CMDS);
    if chosen >= 0 {
        ui.toast_with(CMDS[chosen as usize].title, &cui::Toast { tone: Tone::Success, ..Default::default() });
    }
}

fn main() {
    let mut s = AppState { nav: 6, ..Default::default() };
    let code = App::run(
        &AppDesc {
            title: Some("Orbit"),
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
