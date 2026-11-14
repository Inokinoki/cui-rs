//! Music player — a 1:1 port of examples/music.c from the C repo. Generated
//! cover art, a track list, a "made for you" grid and a transport bar. Shows
//! custom painting mixed with stock components.

use cui::{
    Align, App, AppDesc, Avatar, Button, ButtonVariant, Card, Color, Command, ControlSize,
    Corners, Input, Justify, Menu, MenuItem, Nav, Rect, Row, Size, TextAlign, Tone, Ui, Vec2,
};
use std::f32::consts::TAU;

struct Track {
    title: &'static str,
    artist: &'static str,
    album: &'static str,
    seconds: i32,
    art: usize,
}

const TRACKS: [Track; 7] = [
    Track { title: "Low Tide", artist: "Marlow Avenue", album: "Night Shift", seconds: 222, art: 0 },
    Track { title: "Static Bloom", artist: "Velvet Circuit", album: "Soft Machines", seconds: 247, art: 1 },
    Track { title: "Low Orbit", artist: "Kaito Mori", album: "Satellites", seconds: 198, art: 2 },
    Track { title: "Paper Lanterns", artist: "The Quiet Hours", album: "Ember", seconds: 263, art: 3 },
    Track { title: "Glass Harbor", artist: "Ilse Varga", album: "Tidal", seconds: 231, art: 4 },
    Track { title: "After Hours", artist: "Hollis & Grey", album: "Night Shift", seconds: 214, art: 0 },
    Track { title: "Signal / Noise", artist: "Pale Static", album: "Frequencies", seconds: 276, art: 5 },
];

// Cover art: flat shapes in a few printed colors, like a screen printed
// sleeve. Each index picks a palette and a composition.
struct Palette {
    bg: (u8, u8, u8),
    a: (u8, u8, u8),
    b: (u8, u8, u8),
}

const PALETTES: [Palette; 6] = [
    Palette { bg: (0xc8, 0x55, 0x3d), a: (0xf2, 0xe8, 0xcf), b: (0x2d, 0x30, 0x47) }, // terracotta, cream, ink
    Palette { bg: (0x2d, 0x30, 0x47), a: (0xe0, 0xa4, 0x58), b: (0xf2, 0xe8, 0xcf) }, // ink, ochre, cream
    Palette { bg: (0x5b, 0x8e, 0x7d), a: (0xf2, 0xe8, 0xcf), b: (0x2d, 0x30, 0x47) }, // sage
    Palette { bg: (0xe0, 0xa4, 0x58), a: (0x2d, 0x30, 0x47), b: (0xc8, 0x55, 0x3d) }, // ochre
    Palette { bg: (0x1f, 0x3a, 0x5f), a: (0x8f, 0xb8, 0xde), b: (0xf2, 0xe8, 0xcf) }, // navy, sky
    Palette { bg: (0xf2, 0xe8, 0xcf), a: (0xc8, 0x55, 0x3d), b: (0x2d, 0x30, 0x47) }, // cream
];

#[derive(Clone, Copy)]
struct ArtPaint {
    art: usize,
    radius: f32,
    icon: bool,
}

fn paint_art(ui: &mut cui::CanvasUi, r: Rect, a: &ArtPaint) {
    let p = &PALETTES[a.art % 6];
    let bg = Color::rgb(p.bg.0, p.bg.1, p.bg.2);
    let ca = Color::rgb(p.a.0, p.a.1, p.a.2);
    let cb = Color::rgb(p.b.0, p.b.1, p.b.2);
    let s = r.w;
    ui.draw_rect(r, Corners::all(a.radius), bg);
    ui.push_clip(r);
    match a.art % 6 {
        0 => {
            // a low sun and a small moon
            ui.draw_circle(Vec2 { x: r.x + s * 0.62, y: r.y + s * 0.70 }, s * 0.36, ca);
            ui.draw_circle(Vec2 { x: r.x + s * 0.24, y: r.y + s * 0.26 }, s * 0.09, cb);
        }
        1 => {
            // three bands
            for k in 0..3 {
                ui.draw_rect(
                    Rect { x: r.x, y: r.y + s * (0.22 + k as f32 * 0.2), w: s, h: s * 0.09 },
                    Corners::all(0.),
                    if k == 1 { cb } else { ca },
                );
            }
        }
        2 => {
            // a quarter disc in the corner, a bar across
            ui.draw_circle(Vec2 { x: r.x, y: r.y + s }, s * 0.62, ca);
            ui.draw_rect(
                Rect { x: r.x + s * 0.5, y: r.y + s * 0.18, w: s * 0.34, h: s * 0.08 },
                Corners::all(0.),
                cb,
            );
        }
        3 => {
            // rings
            for k in (1..=3).rev() {
                ui.draw_arc(
                    Vec2 { x: r.x + s * 0.5, y: r.y + s * 0.5 },
                    s * 0.12 * k as f32,
                    0.,
                    TAU,
                    s * 0.035,
                    if k == 2 { cb } else { ca },
                );
            }
        }
        4 => {
            // a grid of dots, one out of place
            for y in 0..4 {
                for x in 0..4 {
                    ui.draw_circle(
                        Vec2 { x: r.x + s * (0.2 + x as f32 * 0.2), y: r.y + s * (0.2 + y as f32 * 0.2) },
                        s * 0.045,
                        if x == 2 && y == 1 { cb } else { ca },
                    );
                }
            }
        }
        _ => {
            // half a disc on a line
            ui.draw_circle(Vec2 { x: r.x + s * 0.5, y: r.y + s * 0.58 }, s * 0.3, ca);
            ui.draw_rect(Rect { x: r.x, y: r.y + s * 0.58, w: s, h: s * 0.42 }, Corners::all(0.), bg);
            ui.draw_rect(Rect { x: r.x + s * 0.1, y: r.y + s * 0.58, w: s * 0.8, h: s * 0.025 }, Corners::all(0.), cb);
        }
    }
    ui.pop_clip();
    if a.icon {
        ui.draw_icon(cui::ICON_MUSIC, r, s * 0.3, cb.with_alpha(0.9));
    }
}

fn cover(ui: &mut Ui, id: &str, art: usize, size: f32, radius: f32) {
    let a = ArtPaint { art, radius, icon: false };
    ui.canvas(id, Size::px(size), Size::px(size), paint_art, &a);
}

// Equalizer bars for the playing row.
#[derive(Clone, Copy)]
struct EqPaint {
    t: f32,
}

fn paint_eq(ui: &mut cui::CanvasUi, r: Rect, e: &EqPaint) {
    let accent = ui.token_color(cui::Token::Accent);
    for i in 0..3 {
        let h = 4. + 8. * (0.5 + 0.5 * (e.t * 6. + i as f32 * 1.9).sin());
        ui.draw_rect(
            Rect { x: r.x + 2. + i as f32 * 4.5, y: r.y + r.h - 2. - h, w: 3., h: h },
            Corners::all(1.),
            accent,
        );
    }
}

// A full-width hover/selection background behind a row of components.
#[derive(Clone, Copy)]
struct RowBg {
    hovered: bool,
    selected: bool,
}

fn paint_row(ui: &mut cui::CanvasUi, r: Rect, b: &RowBg) {
    let active = ui.token_color(cui::Token::Active);
    let hover = ui.token_color(cui::Token::Hover);
    if b.selected {
        ui.draw_rect(r, Corners::all(10.), active);
    } else if b.hovered {
        ui.draw_rect(r, Corners::all(10.), hover);
    }
}

#[derive(Default)]
struct AppState {
    nav: i32,
    playing: usize,
    playlist: usize,
    paused: bool,
    shuffle: bool,
    liked: [bool; 7],
    position: f32,
    volume: f32,
    time: f64,
    search: String,
    drawer: bool, // the library drawer on a narrow window
}

fn compact(ui: &mut Ui) -> bool {
    ui.viewport_size().x < 820.
}

fn fmt_time(seconds: i32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn row_background(ui: &mut Ui, selected: bool) -> bool {
    let bg_id = ui.make_id("bg");
    let hover = ui.state::<bool>(bg_id);
    let bg = RowBg { hovered: *hover, selected };
    let res = ui.canvas("bg", Size::FILL, Size::FILL, paint_row, &bg);
    *ui.state::<bool>(bg_id) = res.hovered;
    res.clicked
}

fn sidebar(ui: &mut Ui, s: &mut AppState) {
    ui.column(
        &cui::Column {
            width: Size::px(248.),
            height: Size::FILL,
            bg: Color::SURFACE_MUTED,
            pad: 12.,
            gap: 2.,
            ..Default::default()
        },
        |ui| {
            ui.row(&Row { gap: 10., pad_x: 6., pad_top: 4., pad_bottom: 14., align: Align::Center, ..Default::default() }, |ui| {
                ui.box_(
                    &cui::BoxOpts {
                        width: Size::px(26.),
                        height: Size::px(26.),
                        radius: 13.,
                        bg: Color::ACCENT,
                        dir: cui::Dir::Stack,
                        align: Align::Center,
                        justify: Justify::Center,
                        ..Default::default()
                    },
                    |ui| ui.icon(cui::ICON_AUDIO_LINES, 14., Color::ACCENT_FOREGROUND),
                );
                ui.text_with("Tempo", &cui::Text { font: cui::font::MEDIUM, size: 15., ..Default::default() });
            });
            library(ui, s);
        },
    );
}

// navigation and playlists: in the sidebar, or in a drawer on a narrow window
fn library(ui: &mut Ui, s: &mut AppState) {
    const ITEMS: [&str; 3] = ["Home", "Browse", "Radio"];
    const ICONS: [&str; 3] = [cui::ICON_HOUSE, cui::ICON_COMPASS, cui::ICON_RADIO];
    for (i, item) in ITEMS.iter().enumerate() {
        if ui.nav_item_with(
            item,
            &Nav { icon: Some(ICONS[i]), selected: s.nav == i as i32, group: Some("nav"), ..Default::default() },
        ) {
            s.nav = i as i32;
            s.drawer = false;
        }
    }
    ui.box_(&cui::BoxOpts { height: Size::px(14.), ..Default::default() }, |_| {});
    ui.row(&Row { pad_right: 4., align: Align::Center, ..Default::default() }, |ui| {
        ui.box_(&cui::BoxOpts { width: Size::FILL, ..Default::default() }, |ui| {
            ui.nav_section("Your library");
        });
        ui.icon_button_with(cui::ICON_PLUS, &Button { size: ControlSize::Sm, tooltip: Some("New playlist"), ..Default::default() });
    });
    const LISTS: [&str; 5] = ["Late Night Coding", "Deep Focus", "Long Drive", "Morning Coffee", "Rainy Afternoons"];
    const COUNTS: [&str; 5] = ["48 songs", "112 songs", "64 songs", "37 songs", "29 songs"];
    let _g = ui.push_id("library");
    for (i, list) in LISTS.iter().enumerate() {
        let _g = ui.push_id_int(i as i32);
        ui.box_(
            &cui::BoxOpts { dir: cui::Dir::Stack, width: Size::FILL, height: Size::px(52.), ..Default::default() },
            |ui| {
                if row_background(ui, s.playlist == i) {
                    s.playlist = i;
                    s.drawer = false;
                }
                ui.row(
                    &Row { width: Size::FILL, height: Size::FILL, pad_x: 8., gap: 12., align: Align::Center, ..Default::default() },
                    |ui| {
                        cover(ui, "art", if i == 4 { 5 } else { i }, 38., 8.);
                        ui.column(&cui::Column { gap: 0., width: Size::FILL, ..Default::default() }, |ui| {
                            ui.text_with(list, &cui::Text { font: cui::font::MEDIUM, size: 13., nowrap: true, ..Default::default() });
                            ui.text_with(COUNTS[i], &cui::Text { size: 12., color: Color::TEXT_MUTED, nowrap: true, ..Default::default() });
                        });
                    },
                );
            },
        );
    }
}

fn hero(ui: &mut Ui, s: &mut AppState) {
    let narrow = compact(ui);
    ui.row(&Row { width: Size::FILL, gap: if narrow { 20. } else { 28. }, align: Align::End, wrap: true, ..Default::default() }, |ui| {
        let a = ArtPaint { art: 0, radius: 16., icon: false };
        let art = if narrow { 148. } else { 196. };
        ui.canvas("hero-art", Size::px(art), Size::px(art), paint_art, &a);
        ui.column(
            &cui::Column { gap: 10., width: Size::FILL, min_width: 260., ..Default::default() },
            |ui| {
                ui.text_with(
                    "Late Night Coding",
                    &cui::Text {
                        font: cui::font::DISPLAY,
                        size: if narrow { 32. } else { 44. },
                        line_height: 1.1,
                        ..Default::default()
                    },
                );
                ui.text_with(
                    "Warm synths and steady beats for the hours after everyone has logged off.",
                    &cui::Text { color: Color::TEXT_SECONDARY, ..Default::default() },
                );
                ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                    ui.avatar_with("Mara Quinn", &Avatar { size: 22., ..Default::default() });
                    ui.text_with("Mara Quinn", &cui::Text { font: cui::font::MEDIUM, size: 13., ..Default::default() });
                    ui.text_with(
                        "·  48 songs  ·  3 h 12 min",
                        &cui::Text { size: 13., color: Color::TEXT_MUTED, ..Default::default() },
                    );
                });
                ui.box_(&cui::BoxOpts { height: Size::px(4.), ..Default::default() }, |_| {});
                ui.row(&Row { gap: 8., align: Align::Center, wrap: true, ..Default::default() }, |ui| {
                    if ui.button_with(
                        if s.paused { "Play" } else { "Pause" },
                        &Button {
                            icon: Some(if s.paused { cui::ICON_PLAY } else { cui::ICON_PAUSE }),
                            variant: ButtonVariant::Primary,
                            size: ControlSize::Lg,
                            ..Default::default()
                        },
                    ) {
                        s.paused = !s.paused;
                    }
                    if ui.button_with(
                        "Shuffle",
                        &Button {
                            icon: Some(cui::ICON_SHUFFLE),
                            size: ControlSize::Lg,
                            variant: if s.shuffle { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                            ..Default::default()
                        },
                    ) {
                        s.shuffle = !s.shuffle;
                    }
                    ui.icon_button_with(cui::ICON_HEART, &Button { size: ControlSize::Lg, tooltip: Some("Save to library"), ..Default::default() });
                    ui.icon_button_with(cui::ICON_DOWNLOAD, &Button { size: ControlSize::Lg, tooltip: Some("Download"), ..Default::default() });
                    ui.menu_with(
                        "More",
                        &Menu { icon: Some(cui::ICON_ELLIPSIS), icon_only: true, variant: ButtonVariant::Ghost, ..Default::default() },
                        |ui| {
                            ui.menu_item_with("Add to queue", &MenuItem { icon: Some(cui::ICON_LIST), ..Default::default() });
                            ui.menu_item_with("Share", &MenuItem { icon: Some(cui::ICON_SHARE_2), ..Default::default() });
                        },
                    );
                });
            },
        );
    });
}

fn track_list(ui: &mut Ui, s: &mut AppState) {
    let album = ui.viewport_size().x >= 600.; // a phone shows the title, the artist and the length
    ui.column(&cui::Column { width: Size::FILL, gap: 2., ..Default::default() }, |ui| {
        ui.row(
            &Row { width: Size::FILL, pad_x: 14., pad_bottom: 8., gap: 14., align: Align::Center, ..Default::default() },
            |ui| {
                ui.text_with(
                    "#",
                    &cui::Text { size: 12., color: Color::TEXT_MUTED, width: Size::px(18.), align: TextAlign::Center, ..Default::default() },
                );
                ui.text_with("Title", &cui::Text { size: 12., color: Color::TEXT_MUTED, width: Size::FILL, ..Default::default() });
                if album {
                    ui.text_with("Album", &cui::Text { size: 12., color: Color::TEXT_MUTED, width: Size::px(180.), ..Default::default() });
                }
                ui.box_(&cui::BoxOpts { width: Size::px(28.), ..Default::default() }, |_| {});
                ui.box_(&cui::BoxOpts { width: Size::px(44.), align: Align::End, ..Default::default() }, |ui| {
                    ui.icon(cui::ICON_CLOCK, 14., Color::TEXT_MUTED);
                });
            },
        );
        ui.separator();
        ui.box_(&cui::BoxOpts { height: Size::px(6.), ..Default::default() }, |_| {});
        let _g = ui.push_id("tracks");
        for (i, t) in TRACKS.iter().enumerate() {
            let playing = s.playing == i;
            let _g = ui.push_id_int(i as i32);
            ui.box_(
                &cui::BoxOpts { dir: cui::Dir::Stack, width: Size::FILL, height: Size::px(56.), ..Default::default() },
                |ui| {
                    if row_background(ui, playing) {
                        s.playing = i;
                        s.position = 0.;
                        s.paused = false;
                    }
                    ui.row(
                        &Row { width: Size::FILL, height: Size::FILL, pad_x: 14., gap: 14., align: Align::Center, ..Default::default() },
                        |ui| {
                            if playing && !s.paused {
                                let e = EqPaint { t: s.time as f32 };
                                ui.canvas("eq", Size::px(18.), Size::px(16.), paint_eq, &e);
                            } else {
                                ui.text_with(
                                    (i + 1).to_string(),
                                    &cui::Text {
                                        size: 13.,
                                        color: if playing { Color::ACCENT } else { Color::TEXT_MUTED },
                                        width: Size::px(18.),
                                        align: TextAlign::Center,
                                        tabular: true,
                                        ..Default::default()
                                    },
                                );
                            }
                            ui.row(&Row { gap: 12., width: Size::FILL, align: Align::Center, ..Default::default() }, |ui| {
                                cover(ui, "art", t.art, 40., 8.);
                                ui.column(&cui::Column { gap: 1., width: Size::FILL, ..Default::default() }, |ui| {
                                    ui.text_with(
                                        t.title,
                                        &cui::Text {
                                            font: cui::font::MEDIUM,
                                            nowrap: true,
                                            color: if playing { Color::ACCENT } else { Color::FOREGROUND },
                                            ..Default::default()
                                        },
                                    );
                                    ui.text_with(
                                        t.artist,
                                        &cui::Text { size: 13., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() },
                                    );
                                });
                            });
                            if album {
                                ui.text_with(
                                    t.album,
                                    &cui::Text { size: 13., color: Color::TEXT_SECONDARY, width: Size::px(180.), nowrap: true, ..Default::default() },
                                );
                            }
                            if ui.icon_button_with(
                                cui::ICON_HEART,
                                &Button {
                                    size: ControlSize::Sm,
                                    color: if s.liked[i] { Color::ACCENT } else { Color::TEXT_MUTED },
                                    tooltip: Some(if s.liked[i] { "Remove from liked" } else { "Like" }),
                                    ..Default::default()
                                },
                            ) {
                                s.liked[i] = !s.liked[i];
                            }
                            ui.text_with(
                                fmt_time(t.seconds),
                                &cui::Text {
                                    size: 13.,
                                    color: Color::TEXT_SECONDARY,
                                    width: Size::px(44.),
                                    align: TextAlign::Right,
                                    tabular: true,
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        }
    });
}

fn made_for_you(ui: &mut Ui) {
    const NAMES: [&str; 4] = ["Daily Mix 1", "Focus Flow", "Discover Weekly", "Release Radar"];
    const DESC: [&str; 4] = [
        "Hollis & Grey, Kaito Mori and more",
        "Instrumental, no lyrics",
        "New music picked for you",
        "Fresh from artists you follow",
    ];
    ui.row(&Row { width: Size::FILL, align: Align::Center, ..Default::default() }, |ui| {
        ui.text_with(
            "Made for you",
            &cui::Text { font: cui::font::DISPLAY, size: 20., width: Size::FILL, ..Default::default() },
        );
        ui.button_with(
            "Show all",
            &Button { variant: ButtonVariant::Ghost, size: ControlSize::Sm, icon_right: Some(cui::ICON_ARROW_RIGHT), ..Default::default() },
        );
    });
    ui.box_(
        &cui::BoxOpts { dir: cui::Dir::Grid, columns: 4, min_column_width: 160., gap: 16., width: Size::FILL, ..Default::default() },
        |ui| {
            for (i, name) in NAMES.iter().enumerate() {
                let _g = ui.push_id_int(i as i32);
                ui.card(&Card { pad: 12., gap: 10., hoverable: true, ..Default::default() }, |ui| {
                    let a = ArtPaint { art: (i + 2) % 6, radius: 10., icon: false };
                    ui.canvas("art", Size::FILL, Size::px(150.), paint_art, &a);
                    ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                        ui.text_with(name, &cui::Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                        ui.text_with(DESC[i], &cui::Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() });
                    });
                });
            }
        },
    );
}

fn player_bar(ui: &mut Ui, s: &mut AppState) {
    let t = &TRACKS[s.playing];
    let narrow = compact(ui);
    ui.row(
        &Row {
            width: Size::FILL,
            height: Size::px(if narrow { 72. } else { 84. }),
            pad_x: if narrow { 12. } else { 20. },
            gap: if narrow { 8. } else { 24. },
            align: Align::Center,
            bg: Color::SURFACE,
            ..Default::default()
        },
        |ui| {
            ui.row(
                &Row { width: if narrow { Size::FILL } else { Size::px(300.) }, gap: 12., align: Align::Center, ..Default::default() },
                |ui| {
                    cover(ui, "now-art", t.art, 52., 10.);
                    ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                        ui.text_with(t.title, &cui::Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                        ui.text_with(t.artist, &cui::Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() });
                    });
                    let liked = s.liked[s.playing];
                    if ui.icon_button_with(
                        cui::ICON_HEART,
                        &Button {
                            size: ControlSize::Sm,
                            color: if liked { Color::ACCENT } else { Color::TEXT_MUTED },
                            tooltip: Some(if liked { "Remove from liked" } else { "Like" }),
                            ..Default::default()
                        },
                    ) {
                        s.liked[s.playing] = !liked;
                    }
                },
            );
            if narrow {
                // a mini player: play and skip; the full controls need the width
                if ui.icon_button_with(
                    if s.paused { cui::ICON_PLAY } else { cui::ICON_PAUSE },
                    &Button {
                        variant: ButtonVariant::Primary,
                        size: ControlSize::Lg,
                        tooltip: Some(if s.paused { "Play" } else { "Pause" }),
                        ..Default::default()
                    },
                ) {
                    s.paused = !s.paused;
                }
                if ui.icon_button_with(cui::ICON_SKIP_FORWARD, &Button { tooltip: Some("Next"), ..Default::default() }) {
                    s.playing = (s.playing + 1) % TRACKS.len();
                    s.position = 0.;
                }
                return;
            }
            ui.column(
                &cui::Column { width: Size::FILL, gap: 6., align: Align::Center, ..Default::default() },
                |ui| {
                    ui.row(&Row { gap: 10., align: Align::Center, ..Default::default() }, |ui| {
                        if ui.icon_button_with(
                            cui::ICON_SHUFFLE,
                            &Button {
                                size: ControlSize::Sm,
                                selected: s.shuffle,
                                color: if s.shuffle { Color::ACCENT } else { Color::TRANSPARENT },
                                tooltip: Some("Shuffle"),
                                ..Default::default()
                            },
                        ) {
                            s.shuffle = !s.shuffle;
                        }
                        if ui.icon_button_with(cui::ICON_SKIP_BACK, &Button { tooltip: Some("Previous"), ..Default::default() }) {
                            s.playing = (s.playing + TRACKS.len() - 1) % TRACKS.len();
                            s.position = 0.;
                        }
                        if ui.icon_button_with(
                            if s.paused { cui::ICON_PLAY } else { cui::ICON_PAUSE },
                            &Button {
                                variant: ButtonVariant::Primary,
                                size: ControlSize::Lg,
                                tooltip: Some(if s.paused { "Play" } else { "Pause" }),
                                ..Default::default()
                            },
                        ) {
                            s.paused = !s.paused;
                        }
                        if ui.icon_button_with(cui::ICON_SKIP_FORWARD, &Button { tooltip: Some("Next"), ..Default::default() }) {
                            s.playing = (s.playing + 1) % TRACKS.len();
                            s.position = 0.;
                        }
                        ui.icon_button_with(cui::ICON_REPEAT, &Button { size: ControlSize::Sm, tooltip: Some("Repeat"), ..Default::default() });
                    });
                    ui.row(&Row { width: Size::FILL, gap: 12., align: Align::Center, ..Default::default() }, |ui| {
                        ui.text_with(
                            fmt_time(s.position as i32),
                            &cui::Text { size: 12., color: Color::TEXT_MUTED, tabular: true, width: Size::px(36.), align: TextAlign::Right, ..Default::default() },
                        );
                        ui.slider("##position", &mut s.position, 0., t.seconds as f32);
                        ui.text_with(
                            fmt_time(t.seconds),
                            &cui::Text { size: 12., color: Color::TEXT_MUTED, tabular: true, width: Size::px(36.), ..Default::default() },
                        );
                    });
                },
            );
            ui.row(
                &Row { width: Size::px(300.), gap: 8., align: Align::Center, justify: Justify::End, ..Default::default() },
                |ui| {
                    ui.icon_button_with(cui::ICON_MIC, &Button { size: ControlSize::Sm, tooltip: Some("Lyrics"), ..Default::default() });
                    ui.icon_button_with(cui::ICON_LIST, &Button { size: ControlSize::Sm, tooltip: Some("Queue"), ..Default::default() });
                    ui.icon(
                        if s.volume > 0. { cui::ICON_VOLUME_2 } else { cui::ICON_VOLUME_X },
                        16.,
                        Color::TEXT_SECONDARY,
                    );
                    ui.slider("##volume", &mut s.volume, 0., 100.);
                },
            );
        },
    );
}

fn frame(ui: &mut Ui, s: &mut AppState) {
    let now = ui.time();
    let mut dt = if s.time > 0. { (now - s.time) as f32 } else { 0. };
    if dt > 0.25 {
        dt = 0.25;
    }
    s.time = now;
    if !s.paused {
        s.position += dt;
        if s.position >= TRACKS[s.playing].seconds as f32 {
            s.playing = (s.playing + 1) % TRACKS.len();
            s.position = 0.;
        }
        ui.request_redraw(); // keep the clock and the equalizer moving
    }
    if !ui.wants_keyboard() && ui.key_pressed(cui::Key::Space) {
        s.paused = !s.paused;
    }
    ui.column(
        &cui::Column { width: Size::FILL, height: Size::FILL, bg: Color::BACKGROUND, ..Default::default() },
        |ui| {
            let narrow = compact(ui);
            if !narrow {
                s.drawer = false;
            }
            ui.row(&Row { width: Size::FILL, height: Size::FILL, align: Align::Stretch, ..Default::default() }, |ui| {
                if !narrow {
                    sidebar(ui, s);
                    ui.separator();
                }
                ui.column(&cui::Column { width: Size::FILL, ..Default::default() }, |ui| {
                    ui.row(
                        &Row { width: Size::FILL, pad_x: if narrow { 12. } else { 32. }, pad_y: 14., gap: 8., align: Align::Center, ..Default::default() },
                        |ui| {
                            if narrow {
                                if ui.icon_button_with(cui::ICON_MENU, &Button { tooltip: Some("Library"), ..Default::default() }) {
                                    s.drawer = true;
                                }
                            } else {
                                ui.icon_button_with(cui::ICON_CHEVRON_LEFT, &Button { tooltip: Some("Back"), ..Default::default() });
                                ui.icon_button_with(cui::ICON_CHEVRON_RIGHT, &Button { tooltip: Some("Forward"), ..Default::default() });
                            }
                            ui.box_(&cui::BoxOpts { width: Size::grow(4.), max_width: 320., ..Default::default() }, |ui| {
                                ui.search_with(
                                    "search",
                                    &mut s.search,
                                    &Input { placeholder: Some("Songs, artists, podcasts"), ..Default::default() },
                                );
                            });
                            ui.spacer();
                            ui.icon_button_with(cui::ICON_BELL, &Button { tooltip: Some("What's new"), ..Default::default() });
                            ui.avatar_with("Mara Quinn", &Avatar { size: 30., ..Default::default() });
                        },
                    );
                    ui.scroll(
                        &cui::Scroll {
                            id: Some("main"),
                            pad: if narrow { 16. } else { 32. },
                            gap: if narrow { 20. } else { 28. },
                            fade_edges: true,
                            ..Default::default()
                        },
                        |ui| {
                            hero(ui, s);
                            track_list(ui, s);
                            made_for_you(ui);
                        },
                    );
                });
            });
            ui.separator();
            player_bar(ui, s);
        },
    );
    let mut drawer = s.drawer;
    ui.sheet(
        "Tempo",
        &mut drawer,
        &cui::Sheet { side: cui::Side::Left, size: 300., ..Default::default() },
        |ui| library(ui, s),
    );
    s.drawer = drawer;
}

fn main() {
    let mut s = AppState { position: 74., volume: 70., liked: [true, false, true, false, false, false, false], ..Default::default() };
    let code = App::run(
        &AppDesc {
            title: Some("Tempo"),
            width: 1440,
            height: 1180,
            theme: cui::ThemeMode::Dark,
            accent: cui::Accent::Coral,
            ..Default::default()
        },
        |ui| frame(ui, &mut s),
    );
    std::process::exit(code);
}
