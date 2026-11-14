//! Metrics dashboard — a 1:1 port of examples/dashboard.c from the C repo.
//! Sidebar, top bar, KPIs, charts and a table. Below 760 points the sidebar
//! becomes a drawer and the panels stack.

use cui::{
    Align, App, AppDesc, Avatar, Badge, Button, ButtonVariant, Card, Color, Command, ControlSize,
    Dialog, Input, Justify, Menu, MenuItem, Nav, Row, Segmented, Series, Size, TextAlign, Tone,
    Ui,
};
use std::sync::OnceLock;

struct AppState {
    nav: i32,
    range: i32,
    search: String,
    page: i32,
    sort: cui::TableSort,
    selected: i32,
    dark: bool,
    palette: bool,
    drawer: bool, // the navigation drawer on a narrow window
}

struct Order {
    id: &'static str,
    customer: &'static str,
    email: &'static str,
    date: &'static str,
    status: &'static str,
    tone: Tone,
    amount: f32,
}

fn orders() -> &'static [Order] {
    static ORDERS: OnceLock<Vec<Order>> = OnceLock::new();
    ORDERS.get_or_init(|| {
        vec![
            Order { id: "#3021", customer: "Nora Lindqvist", email: "nora@lindqvist.se", date: "Nov 5", status: "Paid", tone: Tone::Success, amount: 184.20 },
            Order { id: "#3020", customer: "Kwame Asante", email: "kwame@asante.co", date: "Nov 5", status: "Pending", tone: Tone::Warning, amount: 42.75 },
            Order { id: "#3019", customer: "Hana Novak", email: "hana@novak.cz", date: "Nov 4", status: "Paid", tone: Tone::Success, amount: 318.40 },
            Order { id: "#3018", customer: "Felix Moreau", email: "felix@moreau.fr", date: "Nov 4", status: "Refunded", tone: Tone::Danger, amount: 96.00 },
            Order { id: "#3017", customer: "Iris Coleman", email: "iris@coleman.org", date: "Nov 3", status: "Paid", tone: Tone::Success, amount: 471.15 },
            Order { id: "#3016", customer: "Ravi Shah", email: "ravi@shah.in", date: "Nov 3", status: "Paid", tone: Tone::Success, amount: 1236.80 },
        ]
    })
}

fn compact(ui: &mut Ui) -> bool {
    ui.viewport_size().x < 760.
}

fn nav_items(ui: &mut Ui, s: &mut AppState) {
    const ITEMS: [&str; 6] = ["Overview", "Orders", "Customers", "Products", "Analytics", "Settings"];
    const ICONS: [&str; 6] = [
        cui::ICON_LAYOUT_DASHBOARD, cui::ICON_SHOPPING_CART, cui::ICON_USERS, cui::ICON_PACKAGE,
        cui::ICON_CHART_BAR, cui::ICON_SETTINGS,
    ];
    for (i, item) in ITEMS.iter().enumerate() {
        let badge = (i == 1).then_some("12");
        if ui.nav_item_with(
            item,
            &Nav { icon: Some(ICONS[i]), selected: s.nav == i as i32, badge, ..Default::default() },
        ) {
            s.nav = i as i32;
            s.drawer = false;
        }
    }
}

fn sidebar(ui: &mut Ui, s: &mut AppState) {
    ui.column(
        &cui::Column {
            width: Size::px(232.),
            height: Size::FILL,
            bg: Color::SURFACE_MUTED,
            pad: 12.,
            gap: 2.,
            ..Default::default()
        },
        |ui| {
            ui.row(&Row { gap: 10., pad_x: 8., pad_top: 6., pad_bottom: 16., ..Default::default() }, |ui| {
                ui.box_(
                    &cui::BoxOpts {
                        width: Size::px(26.),
                        height: Size::px(26.),
                        radius: 7.,
                        bg: Color::FOREGROUND,
                        dir: cui::Dir::Stack,
                        align: Align::Center,
                        justify: Justify::Center,
                        ..Default::default()
                    },
                    |ui| {
                        ui.text_with(
                            "L",
                            &cui::Text { font: cui::font::DISPLAY, size: 14., color: Color::BACKGROUND, ..Default::default() },
                        );
                    },
                );
                ui.text_with(
                    "Larkspur Supply",
                    &cui::Text { font: cui::font::MEDIUM, size: 15., nowrap: true, ..Default::default() },
                );
            });
            nav_items(ui, s);
            ui.spacer();
            ui.card(&Card { pad: 16., gap: 10., ..Default::default() }, |ui| {
                ui.text_with("Next payout", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
                ui.text_with(
                    "$4,812.30",
                    &cui::Text { font: cui::font::DISPLAY, size: 20., tabular: true, ..Default::default() },
                );
                ui.text_with(
                    "To the account ending 4417 on Nov 8.",
                    &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() },
                );
            });
        },
    );
}

fn topbar(ui: &mut Ui, s: &mut AppState) {
    let narrow = compact(ui);
    ui.row(
        &Row {
            gap: if narrow { 8. } else { 12. },
            pad_x: if narrow { 12. } else { 32. },
            pad_y: 14.,
            width: Size::FILL,
            ..Default::default()
        },
        |ui| {
            if narrow && ui.icon_button_with(cui::ICON_MENU, &Button { tooltip: Some("Menu"), ..Default::default() }) {
                s.drawer = true;
            }
            ui.box_(&cui::BoxOpts { width: Size::grow(4.), max_width: 320., ..Default::default() }, |ui| {
                ui.search_with(
                    "search",
                    &mut s.search,
                    &Input { placeholder: Some("Search orders, customers"), ..Default::default() },
                );
            });
            ui.spacer();
            let dark_now = ui.theme_mode() == cui::ThemeMode::Dark;
            if ui.icon_button_with(
                if dark_now { cui::ICON_SUN } else { cui::ICON_MOON },
                &Button { tooltip: Some("Toggle theme"), ..Default::default() },
            ) {
                s.dark = !dark_now;
                ui.set_theme(if s.dark { cui::ThemeMode::Dark } else { cui::ThemeMode::Light });
            }
            ui.icon_button_with(cui::ICON_BELL, &Button { tooltip: Some("Notifications"), ..Default::default() });
            ui.menu_with(
                "Account",
                &Menu {
                    icon: Some(cui::ICON_CIRCLE_USER),
                    variant: ButtonVariant::Ghost,
                    placement: cui::Placement::BottomEnd,
                    ..Default::default()
                },
                |ui| {
                    ui.menu_label("nora@larkspur.co");
                    ui.menu_item_with("Profile", &MenuItem { icon: Some(cui::ICON_USER), ..Default::default() });
                    ui.menu_item_with("Billing", &MenuItem { icon: Some(cui::ICON_CREDIT_CARD), ..Default::default() });
                    ui.menu_item_with(
                        "Keyboard shortcuts",
                        &MenuItem { icon: Some(cui::ICON_KEYBOARD), shortcut: Some("?"), ..Default::default() },
                    );
                    ui.menu_separator();
                    ui.menu_item_with("Log out", &MenuItem { icon: Some(cui::ICON_LOG_OUT), ..Default::default() });
                },
            );
        },
    );
}

fn kpi(ui: &mut Ui, label: &str, value: &str, change: &str, good: bool) {
    ui.column(
        &cui::Column { width: Size::grow(1.), pad_x: 22., gap: 6., ..Default::default() },
        |ui| {
            ui.text_with(label, &cui::Text { size: 13., color: Color::TEXT_SECONDARY, ..Default::default() });
            ui.number_roll(
                label,
                value,
                &cui::TextFx { font: cui::font::DISPLAY, size: 26., ..Default::default() },
            );
            ui.text_with(
                change,
                &cui::Text { size: 12., color: if good { Color::SUCCESS } else { Color::DANGER }, ..Default::default() },
            );
        },
    );
}

fn content(ui: &mut Ui, s: &mut AppState) {
    const REV: [f32; 12] = [31., 34., 33., 38., 42., 40., 46., 52., 49., 55., 61., 64.];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let narrow = compact(ui);
    ui.row(&Row { gap: 12., wrap: true, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&cui::Column { gap: 4., width: Size::FILL, min_width: 220., ..Default::default() }, |ui| {
            ui.heading("Overview", 2);
            ui.secondary("November 1 to 30, all channels");
        });
        const RANGES: [&str; 3] = ["7 days", "30 days", "12 months"];
        ui.segmented_with(
            "range",
            &RANGES,
            &mut s.range,
            &Segmented { size: ControlSize::Sm, ..Default::default() },
        );
        ui.button_with(
            "Export",
            &Button { icon: Some(cui::ICON_DOWNLOAD), size: ControlSize::Sm, ..Default::default() },
        );
    });
    // one strip of figures rather than four look-alike cards; the chart below carries the trend
    ui.card(&Card { dir: cui::Dir::Row, gap: 0., pad: 0., ..Default::default() }, |ui| {
        const REV_FIG: [&str; 3] = ["$15,830", "$64,210", "$702,480"];
        const ORD_FIG: [&str; 3] = ["318", "1,286", "14,903"];
        const CONV_FIG: [&str; 3] = ["4.3%", "4.1%", "3.8%"];
        const REF_FIG: [&str; 3] = ["1.5%", "1.7%", "2.0%"];
        const SINCE: [&str; 3] = ["on last week", "on October", "on last year"];
        let k = s.range as usize;
        let c1 = format!("{} {}", ["+3.1%", "+12.4%", "+31.0%"][k], SINCE[k]);
        let c2 = format!("{} {}", ["+1.9%", "+7.3%", "+22.6%"][k], SINCE[k]);
        // without room for four figures in a strip, two per line and no hairlines
        let room = ui.viewport_size().x - if narrow { 32. } else { 297. };
        let two = room < 720.;
        ui.box_(
            &cui::BoxOpts {
                dir: if two { cui::Dir::Grid } else { cui::Dir::Row },
                columns: 2,
                width: Size::FILL,
                gap: if two { 20. } else { 0. },
                pad_y: if two { 4. } else { 0. },
                ..Default::default()
            },
            |ui| {
                kpi(ui, "Revenue", REV_FIG[k], &c1, true);
                if !two {
                    ui.separator();
                }
                kpi(ui, "Orders", ORD_FIG[k], &c2, true);
                if !two {
                    ui.separator();
                }
                kpi(
                    ui,
                    "Conversion",
                    CONV_FIG[k],
                    match k {
                        2 => "+0.6 pts",
                        1 => "+0.4 pts",
                        _ => "+0.2 pts",
                    },
                    true,
                );
                if !two {
                    ui.separator();
                }
                kpi(
                    ui,
                    "Refund rate",
                    REF_FIG[k],
                    match k {
                        2 => "-0.3 pts, fewer returns",
                        1 => "-0.6 pts, fewer returns",
                        _ => "-0.2 pts, fewer returns",
                    },
                    true,
                );
            },
        );
    });
    ui.row(&Row { gap: 16., align: Align::Stretch, wrap: true, width: Size::FILL, ..Default::default() }, |ui| {
        ui.card(&Card { gap: 16., width: Size::grow(2.), min_width: 300., ..Default::default() }, |ui| {
            ui.row(&Row::default(), |ui| {
                ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.label("Revenue");
                    ui.text_with(
                        "Compared with last year",
                        &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() },
                    );
                });
                ui.badge_with(
                    "+18%",
                    &Badge { tone: Tone::Success, icon: Some(cui::ICON_TRENDING_UP), ..Default::default() },
                );
            });
            const LAST: [f32; 12] = [24., 26., 27., 29., 31., 30., 34., 36., 35., 39., 41., 44.];
            ui.line_chart_with(
                "rev",
                &[
                    Series { name: Some("This year"), values: &REV, color: Color::TRANSPARENT },
                    Series { name: Some("Last year"), values: &LAST, color: Color::TRANSPARENT },
                ],
                &cui::Chart {
                    labels: &MONTHS,
                    area: true,
                    smooth: true,
                    height: 230.,
                    format: Some("$%.0fk"),
                    ..Default::default()
                },
            );
        });
        ui.card(&Card { gap: 16., width: Size::grow(1.), min_width: 260., ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 2., ..Default::default() }, |ui| {
                ui.label("Where orders come from");
                ui.text_with(
                    "Share of November orders",
                    &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() },
                );
            });
            const SRC: [f32; 4] = [42., 26., 18., 14.];
            const NAMES: [&str; 4] = ["Search", "Direct", "Social", "Email"];
            ui.column(
                &cui::Column { width: Size::FILL, height: Size::FILL, justify: Justify::Center, ..Default::default() },
                |ui| {
                    ui.donut_chart_with(
                        "src",
                        &SRC,
                        &NAMES,
                        &cui::Chart { height: 170., format: Some("%.0f%%"), ..Default::default() },
                    );
                },
            );
        });
    });
    ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
        ui.row(&Row::default(), |ui| {
            ui.column(&cui::Column { gap: 2., width: Size::FILL, ..Default::default() }, |ui| {
                ui.label("Recent orders");
                ui.text_with("Newest first", &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.button_with(
                "View all",
                &Button {
                    variant: ButtonVariant::Ghost,
                    icon_right: Some(cui::ICON_ARROW_RIGHT),
                    size: ControlSize::Sm,
                    ..Default::default()
                },
            );
        });
        let cols = [
            cui::TableColumn { title: "Order", width: Size::px(90.), align: TextAlign::Left, sortable: true },
            cui::TableColumn { title: "Customer", width: Size::FILL, align: TextAlign::Left, sortable: true },
            cui::TableColumn { title: "Date", width: Size::px(90.), align: TextAlign::Left, sortable: false },
            cui::TableColumn { title: "Status", width: Size::px(120.), align: TextAlign::Left, sortable: true },
            cui::TableColumn { title: "Amount", width: Size::px(110.), align: TextAlign::Right, sortable: true },
        ];
        if ui.table_begin("orders", &cols, &mut cui::Table { sort: Some(&mut s.sort), selectable: true, ..Default::default() }) {
            for (i, o) in orders().iter().enumerate() {
                if !s.search.is_empty() && !o.customer.contains(&s.search) && !o.id.contains(&s.search) {
                    continue;
                }
                if ui.table_row(s.selected == i as i32) {
                    s.selected = i as i32;
                }
                ui.table_cell_text(o.id);
                ui.table_cell();
                ui.avatar_with(o.customer, &Avatar { size: 28., ..Default::default() });
                ui.column(&cui::Column { gap: 0., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.text_with(o.customer, &cui::Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                    ui.text_with(o.email, &cui::Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() });
                });
                ui.table_cell_text(o.date);
                ui.table_cell();
                ui.badge_with(o.status, &Badge { tone: o.tone, dot: true, ..Default::default() });
                ui.table_cell_text(format!("${:.2}", o.amount));
            }
            ui.table_end();
        }
        ui.row(&Row { wrap: true, gap: 8., width: Size::FILL, ..Default::default() }, |ui| {
            ui.box_(&cui::BoxOpts { width: Size::FILL, min_width: 200., ..Default::default() }, |ui| {
                ui.text_with(
                    "Showing 1 to 6 of 1,286 orders",
                    &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() },
                );
            });
            ui.pagination("pages", &mut s.page, 215);
        });
    });
}

fn frame(ui: &mut Ui, s: &mut AppState) {
    if ui.shortcut(cui::Mods::PRIMARY, cui::Key::K) {
        s.palette = true;
    }
    let narrow = compact(ui);
    if !narrow {
        s.drawer = false;
    }
    ui.row(
        &Row { width: Size::FILL, height: Size::FILL, align: Align::Stretch, bg: Color::BACKGROUND, ..Default::default() },
        |ui| {
            if !narrow {
                sidebar(ui, s);
                ui.separator();
            }
            ui.column(&cui::Column { width: Size::FILL, ..Default::default() }, |ui| {
                topbar(ui, s);
                ui.separator();
                ui.scroll(
                    &cui::Scroll {
                        id: Some("content"),
                        pad: if narrow { 16. } else { 32. },
                        gap: if narrow { 16. } else { 24. },
                        fade_edges: true,
                        ..Default::default()
                    },
                    |ui| content(ui, s),
                );
            });
        },
    );
    let mut drawer = s.drawer;
    ui.sheet(
        "Larkspur Supply",
        &mut drawer,
        &cui::Sheet { side: cui::Side::Left, size: 280., ..Default::default() },
        |ui| nav_items(ui, s),
    );
    s.drawer = drawer;
    const CMDS: [Command; 4] = [
        Command { title: "Create order", icon: Some(cui::ICON_PLUS), group: Some("Actions"), shortcut: Some("Ctrl N") },
        Command { title: "Add customer", icon: Some(cui::ICON_USER_PLUS), group: Some("Actions"), shortcut: None },
        Command { title: "Export report", icon: Some(cui::ICON_DOWNLOAD), group: Some("Actions"), shortcut: None },
        Command { title: "Go to analytics", icon: Some(cui::ICON_CHART_BAR), group: Some("Navigation"), shortcut: None },
    ];
    let chosen = ui.command_palette(&mut s.palette, &CMDS);
    if chosen >= 0 {
        ui.toast_with(CMDS[chosen as usize].title, &cui::Toast { tone: Tone::Success, ..Default::default() });
    }
    let _ = Dialog::default();
    let _ = Align::Stretch;
}

fn main() {
    let mut s = AppState {
        nav: 0,
        range: 1,
        search: String::new(),
        page: 0,
        sort: cui::TableSort { column: 0, descending: true },
        selected: 0,
        dark: false,
        palette: false,
        drawer: false,
    };
    let code = App::run(
        &AppDesc {
            title: Some("Larkspur Dashboard"),
            width: 1360,
            height: 900,
            accent: cui::Accent::Blue,
            ..Default::default()
        },
        |ui| frame(ui, &mut s),
    );
    std::process::exit(code);
}
