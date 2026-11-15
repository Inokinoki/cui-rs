//! The CUI component gallery — a 1:1 port of examples/gallery.c,
//! gallery_ui.c, gallery_more.c and gallery_more.h from the C repo.
//! Every component in the library, on one screen.

mod more;
mod pages;

use cui::{Accent, Color, Date, Key, Mods, Size, ThemeMode, Tone, Vec2};
use cui::{
    Align, App, AppDesc, Badge, BoxOpts, Button, ButtonVariant, Column, ControlSize, Dir, Input,
    Justify, Nav, Row, Scroll, Segmented, Sheet, Side,
};

use pages::*;

pub const PAGE_OVERVIEW: usize = 0;
pub const PAGE_BUTTONS: usize = 1;
pub const PAGE_INPUTS: usize = 2;
pub const PAGE_SELECTION: usize = 3;
pub const PAGE_NAVIGATION: usize = 4;
pub const PAGE_OVERLAYS: usize = 5;
pub const PAGE_FEEDBACK: usize = 6;
pub const PAGE_DATA: usize = 7;
pub const PAGE_CHARTS: usize = 8;
pub const PAGE_COUNT: usize = 9;

pub const PAGE_NAMES: [&str; PAGE_COUNT] = [
    "Overview", "Buttons", "Inputs", "Selection", "Navigation", "Overlays", "Feedback", "Data display", "Charts",
];
pub const PAGE_ICONS: [&str; PAGE_COUNT] = [
    cui::ICON_LAYOUT_DASHBOARD, cui::ICON_ZAP, cui::ICON_TYPE, cui::ICON_SLIDERS_HORIZONTAL, cui::ICON_COMPASS,
    cui::ICON_LAYERS, cui::ICON_BELL, cui::ICON_DATABASE, cui::ICON_CHART_LINE,
];

/// All the state, one field per widget (the C `gallery` struct).
pub struct Gallery {
    pub page: usize,
    /* buttons */
    pub loading: bool,
    pub seg_view: i32,
    pub seg_size: i32,
    /* inputs */
    pub name: String,
    pub email: String,
    pub password: String,
    pub search: String,
    pub bio: String,
    pub amount_text: String,
    pub quantity: f64,
    pub date: Date,
    pub color: Color,
    /* selection */
    pub terms: bool,
    pub newsletter: bool,
    pub marketing: bool,
    pub wifi: bool,
    pub bluetooth: bool,
    pub airplane: bool,
    pub plan: i32,
    pub volume: f32,
    pub brightness: f32,
    pub lo: f32,
    pub hi: f32,
    pub rating: i32,
    pub chips: [bool; 6],
    pub framework: i32,
    pub plan_card: i32,
    pub tags: String,
    pub otp: String,
    pub country: i32,
    /* navigation */
    pub tab: i32,
    pub tab2: i32,
    pub page_index: i32,
    pub step: i32,
    /* overlays */
    pub dialog_open: bool,
    pub sheet_open: bool,
    pub palette_open: bool,
    pub confirm_open: bool,
    pub drawer: bool,
    /* data */
    pub acc: [bool; 3],
    pub tree_src: bool,
    pub tree_ui: bool,
    pub tree_assets: bool,
    pub selected_row: usize,
    pub sort: cui::TableSort,
    pub selected_file: usize,
    pub task_order: [i32; 5],
    /* app */
    pub dark: bool,
    pub accent: usize,
    /* the extended pages' state (C: function statics) */
    pub more: more::More,
}

/// A String whose capacity matches the C buffer it stands in for — the
/// capacity is the input limit, like `char buf[96]` in C.
fn buf(init: &str, cap: usize) -> String {
    let mut s = String::with_capacity(cap);
    s.push_str(init);
    s
}

impl Gallery {
    fn new() -> Gallery {
        Gallery {
            page: PAGE_OVERVIEW,
            loading: false,
            seg_view: 0,
            seg_size: 0,
            name: buf("Maya Okafor", 64),
            email: buf("maya@fieldwork.studio", 96),
            password: buf("", 64),
            search: buf("", 64),
            bio: buf("Mathematician and writer. I like engines that compute and interfaces that stay out of the way.", 512),
            amount_text: buf("", 32),
            quantity: 3.0,
            date: Date { year: 2026, month: 11, day: 14 },
            color: Color::rgb(0x77, 0x47, 0xff),
            terms: false,
            newsletter: true,
            marketing: false,
            wifi: true,
            bluetooth: false,
            airplane: false,
            plan: 1,
            volume: 64.0,
            brightness: 40.0,
            lo: 20.0,
            hi: 75.0,
            rating: 4,
            chips: [false; 6],
            framework: -1,
            plan_card: 1,
            tags: buf("design, c99, native", 256),
            otp: buf("418", 8),
            country: 2,
            tab: 0,
            tab2: 0,
            page_index: 0,
            step: 1,
            dialog_open: false,
            sheet_open: false,
            palette_open: false,
            confirm_open: false,
            drawer: false,
            acc: [true, false, false],
            tree_src: true,
            tree_ui: true,
            tree_assets: false,
            selected_row: 1,
            sort: cui::TableSort { column: 0, descending: false },
            selected_file: 2,
            task_order: [0, 1, 2, 3, 4],
            dark: false,
            accent: 1,
            more: more::More::new(),
        }
    }

    fn page_name(&self, page: usize) -> &'static str {
        if page < PAGE_COUNT {
            PAGE_NAMES[page]
        } else {
            more::PAGES[page - PAGE_COUNT].0
        }
    }
}

/// More room above a heading than below it, so it reads as the start of what
/// follows.
pub fn section(ui: &mut cui::Ui, title: &str, desc: Option<&str>) {
    ui.column(&Column { gap: 4., pad_top: 18., ..Default::default() }, |ui| {
        ui.heading(title, 4);
        if let Some(desc) = desc {
            ui.secondary(desc);
        }
    });
}

#[derive(Clone, Copy)]
struct Swatch {
    color: Color,
    selected: bool,
}

fn paint_swatch(ui: &mut cui::CanvasUi, r: cui::Rect, d: &Swatch) {
    let c = Vec2 { x: r.x + r.w / 2., y: r.y + r.h / 2. };
    let fg = ui.token_color(cui::Token::Foreground);
    let surface = ui.token_color(cui::Token::SurfaceMuted);
    if d.selected {
        ui.draw_circle(c, r.w / 2., fg);
    }
    ui.draw_circle(c, r.w / 2. - if d.selected { 2. } else { 0. }, surface);
    ui.draw_circle(c, r.w / 2. - 4., d.color);
}

/// The accent swatches' hex values, in `Accent` order.
pub const ACCENT_HEX: [u32; 8] = [0x555555, 0x7747ff, 0x0562ef, 0x00864b, 0xf3ad20, 0xc25300, 0xd1403a, 0xd23286];

fn sidebar(ui: &mut cui::Ui, g: &mut Gallery, drawer: bool) {
    ui.column(
        &Column {
            width: if drawer { Size::FILL } else { Size::px(240.) },
            height: Size::FILL,
            pad: if drawer { 0. } else { 12. },
            gap: 2.,
            ..Default::default()
        },
        |ui| {
            if !drawer {
                ui.row(&Row { gap: 10., pad_x: 8., pad_top: 6., pad_bottom: 18., ..Default::default() }, |ui| {
                    ui.box_(
                        &BoxOpts {
                            width: Size::px(28.),
                            height: Size::px(28.),
                            radius: 9.,
                            bg: cui::Color::FOREGROUND,
                            dir: Dir::Stack,
                            align: Align::Center,
                            justify: Justify::Center,
                            ..Default::default()
                        },
                        |ui| {
                            ui.text_with(
                                "C",
                                &cui::Text {
                                    font: cui::font::DISPLAY,
                                    size: 16.,
                                    color: cui::Color::BACKGROUND,
                                    line_height: 1.0,
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    ui.text_with("CUI", &cui::Text { font: cui::font::MEDIUM, size: 15., nowrap: true, ..Default::default() });
                    ui.spacer();
                    ui.badge_with(format!("v{}", cui::CUI_VERSION), &Badge { size: ControlSize::Sm, ..Default::default() });
                });
            }
            ui.scroll(&Scroll { id: Some("nav"), height: Size::FILL, gap: 2., ..Default::default() }, |ui| {
                ui.nav_section("Components");
                for i in 0..PAGE_COUNT {
                    if ui.nav_item_with(
                        PAGE_NAMES[i],
                        &Nav { icon: Some(PAGE_ICONS[i]), selected: g.page == i, group: Some("pages"), ..Default::default() },
                    ) {
                        g.page = i;
                        g.drawer = false;
                        ui.scroll_to("main", 0.);
                    }
                }
                ui.gap(10.);
                ui.nav_section("Extended");
                for (i, (name, icon)) in more::PAGES.iter().enumerate() {
                    if ui.nav_item_with(
                        *name,
                        &Nav { icon: Some(icon), selected: g.page == PAGE_COUNT + i, group: Some("pages"), ..Default::default() },
                    ) {
                        g.page = PAGE_COUNT + i;
                        g.drawer = false;
                        ui.scroll_to("main", 0.);
                    }
                }
            });
            ui.column(&Column { gap: 12., pad: 8., ..Default::default() }, |ui| {
                let mode_icons: [&str; 2] = [cui::ICON_SUN, cui::ICON_MOON];
                let mut mode = g.dark as i32;
                if ui.segmented_with(
                    "theme",
                    &["Light", "Dark"],
                    &mut mode,
                    &Segmented { size: ControlSize::Sm, full_width: true, icons: &mode_icons },
                ) {
                    g.dark = mode == 1;
                    ui.set_theme(if g.dark { ThemeMode::Dark } else { ThemeMode::Light });
                }
                ui.row(&Row { justify: Justify::Between, ..Default::default() }, |ui| {
                    for (a, hex) in ACCENT_HEX.iter().enumerate() {
                        let d = Swatch { color: Color::hex(*hex), selected: g.accent == a };
                        let r = ui.canvas(format!("accent-{a}"), Size::px(22.), Size::px(22.), paint_swatch, &d);
                        ui.tooltip(Accent::ALL[a].name());
                        if r.clicked {
                            g.accent = a;
                            ui.set_accent(Accent::ALL[a]);
                        }
                    }
                });
            });
        },
    );
}

fn dialogs(ui: &mut cui::Ui, g: &mut Gallery) {
    let mut open = g.dialog_open;
    let (mut cancel, mut save) = (false, false);
    ui.dialog_with(
        "Edit profile",
        &mut open,
        &cui::Dialog { description: Some("Changes are visible to your team."), ..Default::default() },
        |ui| {
            ui.input_with("dlg-name", &mut g.name, &Input { label: Some("Name"), ..Default::default() });
            ui.input_with(
                "dlg-email",
                &mut g.email,
                &Input { label: Some("Email"), icon: Some(cui::ICON_MAIL), ..Default::default() },
            );
            ui.switch("Public profile", &mut g.wifi);
            ui.dialog_footer(|ui| {
                cancel |= ui.button("Cancel");
                if ui.button_with("Save changes", &Button { variant: ButtonVariant::Primary, ..Default::default() }) {
                    save = true;
                    ui.toast_with("Profile updated", &cui::Toast { tone: Tone::Success, ..Default::default() });
                }
            });
        },
    );
    g.dialog_open = open && !cancel && !save;

    let mut open = g.confirm_open;
    let (mut cancel, mut confirm) = (false, false);
    ui.dialog_with(
        "Delete this project?",
        &mut open,
        &cui::Dialog {
            description: Some("This removes the project and its history for everyone. You cannot undo this."),
            width: 400.,
            ..Default::default()
        },
        |ui| {
            ui.dialog_footer(|ui| {
                cancel |= ui.button("Cancel##confirm");
                if ui.button_with("Delete project", &Button { variant: ButtonVariant::Danger, ..Default::default() }) {
                    confirm = true;
                    ui.toast_with("Project deleted", &cui::Toast { tone: Tone::Danger, action: Some("Undo"), ..Default::default() });
                }
            });
        },
    );
    g.confirm_open = open && !cancel && !confirm;
    ui.sheet(
        "Notifications",
        &mut g.sheet_open,
        &Sheet { description: Some("Choose what we tell you about."), ..Default::default() },
        |ui| {
            ui.switch_with("Comments", &mut g.newsletter, &cui::Check { description: Some("When someone replies to you."), ..Default::default() });
            ui.switch_with("Mentions", &mut g.wifi, &cui::Check { description: Some("When someone mentions you."), ..Default::default() });
            ui.switch_with("Weekly digest", &mut g.bluetooth, &cui::Check { description: Some("A summary every Monday."), ..Default::default() });
            ui.separator();
            ui.label("Recent");
            ui.list_item_with("Jonas commented on Release 1.0", &cui::ListItem { avatar: Some("Jonas Berg"), subtitle: Some("2 minutes ago"), ..Default::default() });
            ui.list_item_with("Priya invited you to Engines", &cui::ListItem { avatar: Some("Priya Nair"), subtitle: Some("1 hour ago"), ..Default::default() });
        },
    );
    let commands = [
        cui::Command { title: "Go to Overview", icon: Some(cui::ICON_LAYOUT_DASHBOARD), group: Some("Navigation"), shortcut: None },
        cui::Command { title: "Go to Buttons", icon: Some(cui::ICON_ZAP), group: Some("Navigation"), shortcut: None },
        cui::Command { title: "Go to Inputs", icon: Some(cui::ICON_TYPE), group: Some("Navigation"), shortcut: None },
        cui::Command { title: "Go to Charts", icon: Some(cui::ICON_CHART_LINE), group: Some("Navigation"), shortcut: None },
        cui::Command { title: "Toggle dark mode", icon: Some(cui::ICON_MOON), group: Some("Preferences"), shortcut: Some("Ctrl J") },
        cui::Command { title: "Change accent color", icon: Some(cui::ICON_PALETTE), group: Some("Preferences"), shortcut: None },
        cui::Command { title: "New project", icon: Some(cui::ICON_PLUS), group: Some("Actions"), shortcut: Some("Ctrl N") },
        cui::Command { title: "Invite teammate", icon: Some(cui::ICON_USER_PLUS), group: Some("Actions"), shortcut: None },
        cui::Command { title: "Open settings", icon: Some(cui::ICON_SETTINGS), group: Some("Actions"), shortcut: Some("Ctrl ,") },
    ];
    match ui.command_palette(&mut g.palette_open, &commands) {
        0 => g.page = PAGE_OVERVIEW,
        1 => g.page = PAGE_BUTTONS,
        2 => g.page = PAGE_INPUTS,
        3 => g.page = PAGE_CHARTS,
        4 => {
            g.dark = !g.dark;
            ui.set_theme(if g.dark { ThemeMode::Dark } else { ThemeMode::Light });
        }
        5 => {
            g.accent = (g.accent + 1) % ACCENT_HEX.len();
            ui.set_accent(Accent::ALL[g.accent]);
        }
        6 => g.page = PAGE_FEEDBACK,
        7 => g.dialog_open = true,
        8 => g.sheet_open = true,
        _ => {}
    }
}

fn frame(ui: &mut cui::Ui, g: &mut Gallery) {
    if !ui.wants_keyboard() {
        if ui.shortcut(Mods::PRIMARY, Key::K) {
            g.palette_open = !g.palette_open;
        }
        if ui.shortcut(Mods::PRIMARY, Key::J) {
            g.dark = !g.dark;
            ui.set_theme(if g.dark { ThemeMode::Dark } else { ThemeMode::Light });
        }
    }
    // below 760 points the page list moves into a drawer behind a menu button
    let narrow = ui.viewport_size().x < 760.;
    if !narrow {
        g.drawer = false;
    }
    ui.row(
        &Row {
            width: Size::FILL,
            height: Size::FILL,
            align: Align::Stretch,
            ..Default::default()
        },
        |ui| {
            if !narrow {
                sidebar(ui, g, false);
                ui.separator();
            }
            ui.scroll(
                &Scroll { id: Some("main"), width: Size::FILL, fade_edges: true, ..Default::default() },
                |ui| {
                    ui.row(&Row { justify: Justify::Center, align: Align::Start, ..Default::default() }, |ui| {
                        ui.column(
                            &Column {
                                width: Size::FILL,
                                max_width: 920.,
                                pad_x: if narrow { 16. } else { 40. },
                                pad_top: if narrow { 12. } else { 36. },
                                pad_bottom: 64.,
                                gap: 20.,
                                ..Default::default()
                            },
                            |ui| {
                                if narrow {
                                    ui.row(&Row { gap: 8., width: Size::FILL, ..Default::default() }, |ui| {
                                        if ui.icon_button_with(cui::ICON_MENU, &Button { tooltip: Some("Components"), ..Default::default() }) {
                                            g.drawer = true;
                                        }
                                        ui.text_with("CUI", &cui::Text { font: cui::font::MEDIUM, size: 15., ..Default::default() });
                                    });
                                }
                                if g.page != PAGE_OVERVIEW {
                                    ui.column(&Column { gap: 6., pad_bottom: 8., ..Default::default() }, |ui| {
                                        let name = g.page_name(g.page);
                                        ui.heading(name, 2);
                                    });
                                }
                                if g.page >= PAGE_COUNT {
                                    more::page(ui, g, g.page - PAGE_COUNT);
                                } else {
                                    match g.page {
                                        PAGE_OVERVIEW => page_overview(ui, g),
                                        PAGE_BUTTONS => page_buttons(ui, g),
                                        PAGE_INPUTS => page_inputs(ui, g),
                                        PAGE_SELECTION => page_selection(ui, g),
                                        PAGE_NAVIGATION => page_navigation(ui, g),
                                        PAGE_OVERLAYS => page_overlays(ui, g),
                                        PAGE_FEEDBACK => page_feedback(ui, g),
                                        PAGE_DATA => page_data(ui, g),
                                        PAGE_CHARTS => page_charts(ui, g),
                                        _ => {}
                                    }
                                }
                            },
                        );
                    });
                },
            );
        },
    );
    let mut drawer = g.drawer;
    ui.sheet("CUI", &mut drawer, &Sheet { side: Side::Left, size: 280., ..Default::default() }, |ui| {
        sidebar(ui, g, true);
    });
    g.drawer = drawer;
    dialogs(ui, g);
}

fn main() {
    // `CUI_GALLERY_SHOT=path:page` renders that page and writes a PNG — the
    // smoke test uses it to screenshot every page without a display grab.
    let pages = PAGE_COUNT + more::PAGES.len();
    let shot = std::env::var("CUI_GALLERY_SHOT").ok().and_then(|v| {
        let (p, page) = v.split_once(':')?;
        Some((p.to_string(), page.parse::<usize>().ok()?))
    });
    if let Some((_, page)) = shot {
        if page >= pages {
            eprintln!("CUI_GALLERY_SHOT: page {page} is out of range (0..{pages})");
            std::process::exit(2);
        }
    }

    let mut g = Gallery::new();
    if let Some((_, page)) = shot {
        g.page = page;
    }
    let mut app = App::new(&AppDesc {
        title: Some("CUI Gallery"),
        width: 1240,
        height: 820,
        min_width: 720,
        min_height: 480,
        // the shot mode must pump frames while nothing moves
        continuous: shot.is_some(),
        ..Default::default()
    })
    .expect("could not create a window");

    let mut frames = 0u32;
    while app.frame() {
        let mut ui = app.ui();
        frame(&mut ui, &mut g);
        if let Some((path, _)) = &shot {
            frames += 1;
            if frames == 30 {
                if let (Some((w, h)), Some(rgba)) = (app.framebuffer_size(), app.screenshot_rgba()) {
                    // physical pixels: the buffer is w x h, not the logical size
                    save_png(path, &rgba, w, h);
                    println!("saved {path}");
                    return;
                }
            }
        }
    }
}

/// Minimal PNG writer (BGRA input, CUI's framebuffer order).
fn save_png(path: &str, rgba: &[u8], w: u32, h: u32) {
    use std::io::Write;
    let f = std::fs::File::create(path).expect("screenshot path");
    let mut out = std::io::BufWriter::new(f);
    let chunk = |tag: [u8; 4], data: &[u8], out: &mut dyn std::io::Write| {
        out.write_all(&(data.len() as u32).to_be_bytes()).unwrap();
        out.write_all(&tag).unwrap();
        out.write_all(data).unwrap();
        let mut crc = 0xFFFF_FFFFu32;
        for b in tag.iter().chain(data) {
            crc ^= *b as u32;
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
            }
        }
        out.write_all(&(crc ^ 0xFFFF_FFFF).to_be_bytes()).unwrap();
    };
    let mut ihdr = [0u8; 13];
    ihdr[0..4].copy_from_slice(&w.to_be_bytes());
    ihdr[4..8].copy_from_slice(&h.to_be_bytes());
    ihdr[8] = 8; // bit depth
    ihdr[9] = 6; // RGBA
    out.write_all(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]).unwrap();
    chunk(*b"IHDR", &ihdr, &mut out);
    let mut raw = Vec::with_capacity((w as usize * 4 + 1) * h as usize);
    for row in rgba.chunks(w as usize * 4) {
        raw.push(0);
        for px in row.chunks(4) {
            raw.extend_from_slice(&[px[2], px[1], px[0], 255]);
        }
    }
    // zlib wrapper around stored (uncompressed) deflate blocks. The stream
    // trailer is Adler-32 (big-endian), NOT the CRC-32 used for the chunk
    // framing above — decoders reject the file otherwise.
    let mut idat = vec![0x78, 0x01];
    let mut parts = raw.chunks(65535).peekable();
    while let Some(part) = parts.next() {
        idat.push(if parts.peek().is_none() { 1 } else { 0 });
        let n = part.len() as u16;
        idat.extend_from_slice(&n.to_le_bytes());
        idat.extend_from_slice(&(!n).to_le_bytes());
        idat.extend_from_slice(part);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for block in raw.chunks(5552) {
        // 5552 is the largest run proven to keep the 32-bit sums exact
        for byte in block {
            a += *byte as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    idat.extend_from_slice(&((b << 16) | a).to_be_bytes());
    chunk(*b"IDAT", &idat, &mut out);
    chunk(*b"IEND", &[], &mut out);
}
