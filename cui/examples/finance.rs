//! Personal finance — a 1:1 port of examples/finance.c from the C repo.
//! Top navigation, a balance chart, a painted payment card, budgets,
//! savings goals and recent transactions. Dark by default.

use cui::{
    Align, App, AppDesc, Badge, Button, ButtonVariant, Card, Color, Command, ControlSize, Input,
    Justify, Menu, MenuItem, Row, Segmented, Series, Size, TextAlign, Tone, Ui, Vec2,
};
use std::f32::consts::TAU;

#[derive(Default)]
struct AppState {
    tab: i32,
    range: i32,
    frozen: bool,
    palette: bool,
    search: String,
}

struct Txn {
    merchant: &'static str,
    category: &'static str,
    icon: &'static str,
    when: &'static str,
    amount: f32,
    tone: Tone,
}

fn txns() -> &'static [Txn] {
    static TXNS: std::sync::OnceLock<Vec<Txn>> = std::sync::OnceLock::new();
    TXNS.get_or_init(|| {
        vec![
            Txn { merchant: "Salary", category: "Income", icon: cui::ICON_BRIEFCASE, when: "Today, 09:12", amount: 6200.00, tone: Tone::Success },
            Txn { merchant: "Blue Bottle", category: "Coffee", icon: cui::ICON_GIFT, when: "Today, 08:40", amount: -6.50, tone: Tone::Warning },
            Txn { merchant: "Fieldnotes Rent", category: "Housing", icon: cui::ICON_BUILDING_2, when: "Yesterday", amount: -1850.00, tone: Tone::Info },
            Txn { merchant: "Night Market", category: "Groceries", icon: cui::ICON_SHOPPING_CART, when: "Yesterday", amount: -84.20, tone: Tone::Teal },
            Txn { merchant: "Cloud Storage", category: "Subscriptions", icon: cui::ICON_CLOUD, when: "Nov 3", amount: -9.99, tone: Tone::Violet },
            Txn { merchant: "Transfer from Sam", category: "Transfer", icon: cui::ICON_ARROW_DOWN, when: "Nov 2", amount: 120.00, tone: Tone::Success },
        ]
    })
}

// --- the payment card ------------------------------------------------------

#[derive(Clone, Copy)]
struct CardPaint {
    frozen: bool,
}

fn paint_card(ui: &mut cui::CanvasUi, r: cui::Rect, c: &CardPaint) {
    let rad = cui::Corners::all(18.);
    ui.draw_shadow(r, rad, 32., -6., Vec2 { x: 0., y: 16. }, Color::rgba(0x00, 0x00, 0x00, 0x80));
    ui.draw_rect_gradient(r, rad, Color::rgb(0x1f, 0x6f, 0x5c), Color::rgb(0x0b, 0x2a, 0x24));
    ui.push_clip(r);
    // printed rings, like the guilloche on a real card; no glow
    ui.draw_arc(
        Vec2 { x: r.x + r.w * 0.92, y: r.y + r.h * 1.05 },
        r.h * 0.75,
        0.,
        6.283_185_3,
        1.,
        Color::rgba(0xff, 0xff, 0xff, 0x1c),
    );
    ui.draw_arc(
        Vec2 { x: r.x + r.w * 0.92, y: r.y + r.h * 1.05 },
        r.h * 0.55,
        0.,
        6.283_185_3,
        1.,
        Color::rgba(0xff, 0xff, 0xff, 0x14),
    );
    ui.pop_clip();
    ui.draw_rect_border(r, rad, Color::TRANSPARENT, Color::rgba(0xff, 0xff, 0xff, 0x22), 1.);

    let white = Color::rgb(0xff, 0xff, 0xff);
    let soft = Color::rgba(0xff, 0xff, 0xff, 0xb0);
    ui.draw_text(cui::font::DISPLAY, 17., Vec2 { x: r.x + 24., y: r.y + 22. }, white, "Halden");
    ui.draw_icon(cui::ICON_WIFI, cui::Rect { x: r.x + r.w - 48., y: r.y + 16., w: 28., h: 28. }, 18., soft);
    // chip
    let chip = cui::Rect { x: r.x + 24., y: r.y + r.h * 0.40, w: 40., h: 30. };
    ui.draw_rect_gradient(chip, cui::Corners::all(6.), Color::rgb(0xf3, 0xd5, 0x8a), Color::rgb(0xb8, 0x91, 0x3f));
    ui.draw_line(
        Vec2 { x: chip.x, y: chip.y + 15. },
        Vec2 { x: chip.x + 40., y: chip.y + 15. },
        1.,
        Color::rgba(0x00, 0x00, 0x00, 0x40),
    );
    ui.draw_line(
        Vec2 { x: chip.x + 14., y: chip.y },
        Vec2 { x: chip.x + 14., y: chip.y + 30. },
        1.,
        Color::rgba(0x00, 0x00, 0x00, 0x40),
    );
    ui.draw_line(
        Vec2 { x: chip.x + 26., y: chip.y },
        Vec2 { x: chip.x + 26., y: chip.y + 30. },
        1.,
        Color::rgba(0x00, 0x00, 0x00, 0x40),
    );
    ui.draw_text(
        cui::font::MONO,
        17.,
        Vec2 { x: r.x + 24., y: r.y + r.h - 64. },
        white,
        "4821  ••••  ••••  0937",
    );
    ui.draw_text(cui::font::BODY, 12., Vec2 { x: r.x + 24., y: r.y + r.h - 34. }, soft, "MARA QUINN");
    ui.draw_text(
        cui::font::BODY,
        12.,
        Vec2 { x: r.x + r.w * 0.5, y: r.y + r.h - 34. },
        soft,
        "09 / 29",
    );
    ui.draw_text(
        cui::font::MEDIUM,
        13.,
        Vec2 { x: r.x + r.w - 66., y: r.y + r.h - 40. },
        white,
        "DEBIT",
    );
    if c.frozen {
        ui.draw_rect(r, rad, Color::rgba(0x0a, 0x0f, 0x0e, 0xc0));
        ui.draw_icon(cui::ICON_LOCK, r, 28., white);
    }
}

// --- pieces ----------------------------------------------------------------

fn compact(ui: &mut Ui) -> bool {
    ui.viewport_size().x < 900.
}

fn topbar(ui: &mut Ui, s: &mut AppState) {
    const TABS: [&str; 5] = ["Overview", "Accounts", "Payments", "Cards", "Insights"];
    let narrow = compact(ui);
    ui.row(
        &Row {
            width: Size::FILL,
            pad_x: if narrow { 16. } else { 32. },
            pad_top: 14.,
            gap: if narrow { 12. } else { 28. },
            align: Align::Center,
            ..Default::default()
        },
        |ui| {
            ui.row(&Row { gap: 10., align: Align::Center, ..Default::default() }, |ui| {
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
                    |ui| {
                        ui.text_with(
                            "h",
                            &cui::Text { font: cui::font::DISPLAY, size: 15., color: Color::ACCENT_FOREGROUND, ..Default::default() },
                        );
                    },
                );
                ui.text_with("Halden", &cui::Text { font: cui::font::DISPLAY, size: 17., ..Default::default() });
            });
            if !narrow {
                ui.tabs("nav", &TABS, &mut s.tab);
            }
            ui.spacer();
            ui.box_(&cui::BoxOpts { width: Size::grow(4.), max_width: 220., ..Default::default() }, |ui| {
                ui.search_with("search", &mut s.search, &Input { placeholder: Some("Search"), ..Default::default() });
            });
            ui.icon_button_with(cui::ICON_BELL, &Button { tooltip: Some("Notifications"), ..Default::default() });
            ui.avatar_with("Mara Quinn", &cui::Avatar { size: 32., status: Some("online"), ..Default::default() });
        },
    );
    if narrow {
        ui.box_(&cui::BoxOpts { width: Size::FILL, pad_x: 16., pad_top: 6., ..Default::default() }, |ui| {
            ui.tabs("nav", &TABS, &mut s.tab);
        });
    }
}

fn balance_card(ui: &mut Ui, _s: &mut AppState) {
    const MONTHS: [&str; 12] = ["Dec", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov"];
    const BALANCE: [f32; 12] = [31.2, 32.8, 32.1, 34.6, 36.0, 35.2, 38.4, 40.1, 39.3, 42.8, 45.1, 48.2];
    ui.card(&Card { gap: 18., width: Size::grow(2.), min_width: 300., ..Default::default() }, |ui| {
        ui.row(&Row { width: Size::FILL, align: Align::Start, wrap: true, gap: 12., ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 6., width: Size::FILL, min_width: 240., ..Default::default() }, |ui| {
                ui.text_with("Total balance", &cui::Text { size: 13., color: Color::TEXT_SECONDARY, ..Default::default() });
                ui.row(&Row { gap: 12., align: Align::Center, ..Default::default() }, |ui| {
                    ui.text_with(
                        "$48,250.32",
                        &cui::Text { font: cui::font::DISPLAY, size: 38., tabular: true, ..Default::default() },
                    );
                    ui.badge_with("+6.9%", &Badge { tone: Tone::Success, icon: Some(cui::ICON_TRENDING_UP), ..Default::default() });
                });
                ui.text_with(
                    "+$3,104 since last month across 4 accounts",
                    &cui::Text { size: 13., color: Color::TEXT_MUTED, ..Default::default() },
                );
            });
            const RANGES: [&str; 4] = ["1M", "6M", "1Y", "All"];
            ui.segmented_with(
                "range",
                &RANGES,
                &mut _s.range,
                &Segmented { size: ControlSize::Sm, ..Default::default() },
            );
        });
        ui.line_chart_with(
            "balance",
            &[Series { name: Some("Balance"), values: &BALANCE, color: Color::TRANSPARENT }],
            &cui::Chart {
                labels: &MONTHS,
                area: true,
                smooth: true,
                height: 236.,
                format: Some("$%.0fk"),
                no_legend: true,
                ..Default::default()
            },
        );
    });
}

fn card_panel(ui: &mut Ui, s: &mut AppState) {
    ui.card(&Card { gap: 18., width: Size::grow(1.), min_width: 280., ..Default::default() }, |ui| {
        ui.row(&Row { width: Size::FILL, align: Align::Center, ..Default::default() }, |ui| {
            ui.label("My cards");
            ui.spacer();
            ui.button_with("Add", &Button { icon: Some(cui::ICON_PLUS), size: ControlSize::Sm, variant: ButtonVariant::Ghost, ..Default::default() });
        });
        let c = CardPaint { frozen: s.frozen };
        ui.canvas("card", Size::FILL, Size::px(200.), paint_card, &c);
        ui.box_(&cui::BoxOpts { dir: cui::Dir::Grid, columns: 3, gap: 10., width: Size::FILL, ..Default::default() }, |ui| {
            const LABELS: [&str; 3] = ["Send", "Request", "Top up"];
            const ICONS: [&str; 3] = [cui::ICON_ARROW_UP_RIGHT, cui::ICON_ARROW_DOWN, cui::ICON_PLUS];
            for (i, label) in LABELS.iter().enumerate() {
                ui.button_with(label, &Button { icon: Some(ICONS[i]), full_width: true, ..Default::default() });
            }
        });
        ui.row(&Row { width: Size::FILL, gap: 12., align: Align::Center, ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                ui.text_with("Freeze card", &cui::Text { font: cui::font::MEDIUM, size: 13., ..Default::default() });
                ui.text_with("Block new payments instantly", &cui::Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
            });
            ui.switch("##freeze", &mut s.frozen);
        });
    });
}

fn spending_card(ui: &mut Ui) {
    ui.card(&Card { gap: 20., width: Size::grow(1.), min_width: 260., ..Default::default() }, |ui| {
        ui.row(&Row { width: Size::FILL, align: Align::Center, ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                ui.label("Spending");
                ui.text_with("November, by category", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.menu_with(
                "Options",
                &Menu { icon: Some(cui::ICON_ELLIPSIS), icon_only: true, variant: ButtonVariant::Ghost, placement: cui::Placement::BottomEnd, ..Default::default() },
                |ui| {
                    ui.menu_item_with("Export CSV", &MenuItem { icon: Some(cui::ICON_DOWNLOAD), ..Default::default() });
                    ui.menu_item_with("Edit categories", &MenuItem { icon: Some(cui::ICON_TAG), ..Default::default() });
                },
            );
        });
        const VALS: [f32; 4] = [1850., 640., 420., 310.];
        const NAMES: [&str; 4] = ["Housing", "Groceries", "Dining", "Other"];
        ui.donut_chart_with("spend", &VALS, &NAMES, &cui::Chart { height: 150., format: Some("$%.0f"), ..Default::default() });
    });
}

fn budgets_card(ui: &mut Ui) {
    struct Budget {
        name: &'static str,
        icon: &'static str,
        used: f32,
        total: f32,
    }
    const B: [Budget; 4] = [
        Budget { name: "Groceries", icon: cui::ICON_SHOPPING_CART, used: 640., total: 800. },
        Budget { name: "Dining out", icon: cui::ICON_GIFT, used: 420., total: 400. },
        Budget { name: "Transport", icon: cui::ICON_MAP_PIN, used: 96., total: 250. },
        Budget { name: "Entertainment", icon: cui::ICON_MUSIC, used: 186., total: 200. },
    ];
    ui.card(&Card { gap: 16., width: Size::grow(1.), min_width: 260., ..Default::default() }, |ui| {
        ui.row(&Row { width: Size::FILL, align: Align::Center, ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                ui.label("Budgets");
                ui.text_with("12 days left this month", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.button_with("Edit", &Button { size: ControlSize::Sm, variant: ButtonVariant::Ghost, ..Default::default() });
        });
        for (i, b) in B.iter().enumerate() {
            let _g = ui.push_id_int(i as i32);
            ui.column(&cui::Column { gap: 8., width: Size::FILL, ..Default::default() }, |ui| {
                ui.row(&Row { width: Size::FILL, gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                    ui.icon(b.icon, 14., Color::TEXT_SECONDARY);
                    ui.text_with(b.name, &cui::Text { size: 13., width: Size::FILL, nowrap: true, ..Default::default() });
                    let t = format!("${:.0} / ${:.0}", b.used, b.total);
                    ui.text_with(
                        t,
                        &cui::Text {
                            size: 12.,
                            tabular: true,
                            color: if b.used > b.total { Color::DANGER } else { Color::TEXT_SECONDARY },
                            ..Default::default()
                        },
                    );
                });
                // one rule for every budget: over is red, the last tenth is amber, the rest is plain
                let v = b.used / b.total;
                let tone = if v > 1. { Tone::Danger } else if v >= 0.9 { Tone::Warning } else { Tone::Neutral };
                ui.progress_with(v.min(1.), &cui::Progress { height: 6., tone, ..Default::default() });
            });
        }
    });
}

fn goals_card(ui: &mut Ui) {
    ui.card(&Card { gap: 18., width: Size::grow(1.), min_width: 260., ..Default::default() }, |ui| {
        ui.row(&Row { width: Size::FILL, align: Align::Center, ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                ui.label("Savings goals");
                ui.text_with("On track for 2 of 3", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.icon_button_with(cui::ICON_PLUS, &Button { size: ControlSize::Sm, tooltip: Some("New goal"), ..Default::default() });
        });
        ui.row(&Row { width: Size::FILL, justify: Justify::Between, ..Default::default() }, |ui| {
            const NAMES: [&str; 3] = ["Japan trip", "New laptop", "Emergency"];
            const AMTS: [&str; 3] = ["$3,240 of $4,000", "$1,100 of $2,400", "$9,800 of $10k"];
            const V: [f32; 3] = [0.81, 0.46, 0.98];
            for (i, name) in NAMES.iter().enumerate() {
                let _g = ui.push_id_int(i as i32);
                ui.column(&cui::Column { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
                    ui.progress_ring("##goal", V[i], 72.);
                    ui.text_with(name, &cui::Text { font: cui::font::MEDIUM, size: 13., ..Default::default() });
                    ui.text_with(AMTS[i], &cui::Text { size: 11., color: Color::TEXT_MUTED, ..Default::default() });
                });
            }
        });
    });
}

fn tone_icon(ui: &mut Ui, icon: &str, tone: Tone) {
    const TOK: [cui::Token; 5] = [
        cui::Token::TextSecondary, cui::Token::Info, cui::Token::Success, cui::Token::Warning,
        cui::Token::Danger,
    ];
    let idx = match tone {
        Tone::Neutral => 0,
        Tone::Info => 1,
        Tone::Success => 2,
        Tone::Warning => 3,
        _ => 4,
    };
    let c = ui.color_of(TOK[idx].color());
    ui.box_(
        &cui::BoxOpts {
            width: Size::px(38.),
            height: Size::px(38.),
            radius: 12.,
            bg: c.with_alpha(0.14),
            dir: cui::Dir::Stack,
            align: Align::Center,
            justify: Justify::Center,
            ..Default::default()
        },
        |ui| ui.icon(icon, 16., c),
    );
}

fn transactions_card(ui: &mut Ui) {
    ui.card(&Card { gap: 6., width: Size::grow(2.), ..Default::default() }, |ui| {
        ui.row(&Row { width: Size::FILL, pad_bottom: 8., align: Align::Center, ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                ui.label("Recent transactions");
                ui.text_with("Across all accounts", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.button_with(
                "View all",
                &Button { variant: ButtonVariant::Ghost, size: ControlSize::Sm, icon_right: Some(cui::ICON_ARROW_RIGHT), ..Default::default() },
            );
        });
        for (i, t) in txns().iter().enumerate() {
            let _g = ui.push_id_int(i as i32);
            ui.row(&Row { width: Size::FILL, gap: 14., pad_y: 7., align: Align::Center, ..Default::default() }, |ui| {
                tone_icon(ui, t.icon, t.tone);
                ui.column(&cui::Column { gap: 1., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.text_with(t.merchant, &cui::Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                    ui.text_with(t.when, &cui::Text { size: 12., color: Color::TEXT_MUTED, nowrap: true, ..Default::default() });
                });
                if ui.viewport_size().x >= 560. {
                    ui.badge(t.category);
                }
                let amount = format!("{}${:.2}", if t.amount < 0. { "-" } else { "+" }, t.amount.abs());
                ui.text_with(
                    amount,
                    &cui::Text {
                        font: cui::font::MEDIUM,
                        tabular: true,
                        width: Size::px(110.),
                        align: TextAlign::Right,
                        color: if t.amount > 0. { Color::SUCCESS } else { Color::FOREGROUND },
                        ..Default::default()
                    },
                );
            });
        }
    });
}

fn frame(ui: &mut Ui, s: &mut AppState) {
    if ui.shortcut(cui::Mods::PRIMARY, cui::Key::K) {
        s.palette = true;
    }
    ui.column(
        &cui::Column { width: Size::FILL, height: Size::FILL, bg: Color::BACKGROUND, ..Default::default() },
        |ui| {
            topbar(ui, s);
            ui.separator();
            let narrow = compact(ui);
            ui.scroll(
                &cui::Scroll {
                    id: Some("main"),
                    pad: if narrow { 16. } else { 32. },
                    gap: if narrow { 16. } else { 20. },
                    fade_edges: true,
                    ..Default::default()
                },
                |ui| {
                    ui.row(&Row { width: Size::FILL, align: Align::End, wrap: true, gap: 12., ..Default::default() }, |ui| {
                        ui.column(&cui::Column { gap: 4., width: Size::FILL, min_width: 240., ..Default::default() }, |ui| {
                            ui.text_with("Overview", &cui::Text { font: cui::font::DISPLAY, size: 28., ..Default::default() });
                            ui.text_with(
                                "Four accounts, synced at 6:42 pm",
                                &cui::Text { color: Color::TEXT_SECONDARY, ..Default::default() },
                            );
                        });
                        ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
                            ui.button_with("Statements", &Button { icon: Some(cui::ICON_FILE_TEXT), variant: ButtonVariant::Outline, ..Default::default() });
                            ui.button_with("Move money", &Button { icon: Some(cui::ICON_SEND), variant: ButtonVariant::Primary, ..Default::default() });
                        });
                    });
                    ui.row(&Row { width: Size::FILL, gap: if narrow { 16. } else { 20. }, align: Align::Stretch, wrap: true, ..Default::default() }, |ui| {
                        balance_card(ui, s);
                        card_panel(ui, s);
                    });
                    ui.row(&Row { width: Size::FILL, gap: if narrow { 16. } else { 20. }, align: Align::Stretch, wrap: true, ..Default::default() }, |ui| {
                        spending_card(ui);
                        budgets_card(ui);
                        goals_card(ui);
                    });
                    ui.row(&Row { width: Size::FILL, gap: 20., align: Align::Stretch, ..Default::default() }, |ui| {
                        transactions_card(ui);
                    });
                },
            );
        },
    );
    const CMDS: [Command; 3] = [
        Command { title: "Send money", icon: Some(cui::ICON_SEND), group: Some("Actions"), shortcut: None },
        Command { title: "Freeze card", icon: Some(cui::ICON_LOCK), group: Some("Actions"), shortcut: None },
        Command { title: "Download statement", icon: Some(cui::ICON_DOWNLOAD), group: Some("Actions"), shortcut: None },
    ];
    let chosen = ui.command_palette(&mut s.palette, &CMDS);
    if chosen == 1 {
        s.frozen = !s.frozen;
    } else if chosen >= 0 {
        ui.toast_with(CMDS[chosen as usize].title, &cui::Toast { tone: Tone::Success, ..Default::default() });
    }
}

fn main() {
    let mut s = AppState { range: 2, ..Default::default() };
    let code = App::run(
        &AppDesc {
            title: Some("Halden"),
            width: 1440,
            height: 940,
            theme: cui::ThemeMode::Dark,
            accent: cui::Accent::Green,
            ..Default::default()
        },
        |ui| frame(ui, &mut s),
    );
    std::process::exit(code);
}
