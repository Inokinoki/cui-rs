//! The nine base gallery pages — a 1:1 port of the page functions in
//! examples/gallery_ui.c from the C repo.

use core::ffi::c_char;

use cui::{Align, Color, Date, Dir, Justify, Size, TextAlign, Tone};
use cui::{
    Avatar, Badge, BoxOpts, Button, ButtonVariant, Card, Chart, Check, Column, ControlSize,
    EmptyState, Input, Metric, Number, Progress, Row, Segmented, Select, Series, Slider, Table,
    TableColumn, Tabs, TabsVariant, Text, TimelineItem, Toast, Ui,
};

use crate::{section, Gallery, PAGE_BUTTONS};

/// Arena-backed C string for raw `ffi::*_opts` string fields; valid for the frame.
fn s(ui: &Ui, text: &str) -> *const c_char {
    ui.arena().cstr(text)
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

pub fn page_overview(ui: &mut Ui, g: &mut Gallery) {
    ui.column(&Column { gap: 14., pad_bottom: 8., ..Default::default() }, |ui| {
        ui.heading("Components for C", 1);
        ui.text_with(
            "Forms, tables, charts, planning boards and chat pieces that draw themselves the same way on Windows, macOS and Linux. Every one works from the keyboard.",
            &Text { color: Color::TEXT_SECONDARY, size: 15., ..Default::default() },
        );
        ui.row(&Row { gap: 8., pad_top: 6., wrap: true, ..Default::default() }, |ui| {
            if ui.button_with(
                "Browse components",
                &Button { variant: ButtonVariant::Primary, icon_right: Some(cui::ICON_ARROW_RIGHT), ..Default::default() },
            ) {
                g.page = PAGE_BUTTONS;
            }
            if ui.button_with(
                "Open command palette",
                &Button { variant: ButtonVariant::Outline, icon: Some(cui::ICON_COMMAND), ..Default::default() },
            ) {
                g.palette_open = true;
            }
        });
    });
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 3, min_column_width: 200., gap: 16., ..Default::default() }, |ui| {
        let s2 = [9., 7., 8., 6., 5., 6., 4., 4., 3.];
        ui.metric_with("Components", "150", &Metric { icon: Some(cui::ICON_BOX), ..Default::default() });
        ui.metric_with(
            "Frame time",
            "0.4 ms",
            &Metric {
                delta: -18.,
                delta_label: Some("-18% vs v0.9"),
                lower_is_better: true,
                spark: &s2,
                icon: Some(cui::ICON_ZAP),
                ..Default::default()
            },
        );
        ui.metric_with("Dependencies", "0", &Metric { icon: Some(cui::ICON_PACKAGE), ..Default::default() });
    });
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
            ui.column(&Column { gap: 4., ..Default::default() }, |ui| {
                ui.heading("Create a workspace", 4);
                ui.secondary("You can invite people once it exists.");
            });
            ui.input_with("ov-name", &mut g.name, &Input { label: Some("Name"), placeholder: Some("Your name"), ..Default::default() });
            ui.input_with(
                "ov-email",
                &mut g.email,
                &Input { label: Some("Email"), icon: Some(cui::ICON_MAIL), placeholder: Some("you@company.com"), ..Default::default() },
            );
            ui.checkbox("Email me a weekly summary", &mut g.terms);
            if ui.button_with("Create workspace", &Button { variant: ButtonVariant::Primary, full_width: true, ..Default::default() }) {
                ui.toast_with(
                    "Workspace created",
                    &Toast { description: Some("It is empty for now. Start with a project."), tone: Tone::Success, ..Default::default() },
                );
            }
        });
        ui.card(&Card { gap: 12., ..Default::default() }, |ui| {
            ui.row(&Row::default(), |ui| {
                ui.column(&Column { gap: 4., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.heading("Team", 4);
                    ui.secondary("Four people, one owner.");
                });
                ui.button_with("Invite", &Button { icon: Some(cui::ICON_USER_PLUS), size: ControlSize::Sm, ..Default::default() });
            });
            let people: [[&str; 3]; 4] = [
                ["Jonas Berg", "jonas@northlight.io", "Owner"],
                ["Priya Nair", "priya@harbor.dev", "Admin"],
                ["Tomas Rivera", "tomas@kiln.app", "Editor"],
                ["Lena Fischer", "lena@fischer.dev", "Viewer"],
            ];
            for (i, person) in people.iter().enumerate() {
                ui.row(&Row { gap: 12., pad_y: 4., ..Default::default() }, |ui| {
                    ui.avatar_with(
                        person[0],
                        &Avatar {
                            size: 34.,
                            status: match i {
                                0 => Some("online"),
                                1 => Some("away"),
                                _ => None,
                            },
                            ..Default::default()
                        },
                    );
                    ui.column(&Column { width: Size::FILL, gap: 1., ..Default::default() }, |ui| {
                        ui.text_with(person[0], &Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
                        ui.text_with(person[1], &Text { size: 12., color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() });
                    });
                    ui.badge_with(person[2], &Badge { tone: if i == 0 { Tone::Accent } else { Tone::Neutral }, ..Default::default() });
                });
            }
        });
    });
}

pub fn page_buttons(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Variants", Some("One primary action per view. Secondary for everything else."));
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            ui.button_with("Primary", &Button { variant: ButtonVariant::Primary, ..Default::default() });
            ui.button("Secondary");
            ui.button_with("Outline", &Button { variant: ButtonVariant::Outline, ..Default::default() });
            ui.button_with("Ghost", &Button { variant: ButtonVariant::Ghost, ..Default::default() });
            ui.button_with("Danger", &Button { variant: ButtonVariant::Danger, ..Default::default() });
            ui.button_with("Link", &Button { variant: ButtonVariant::Link, ..Default::default() });
        });
    });
    section(ui, "Sizes and icons", None);
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            ui.button_with("Small", &Button { size: ControlSize::Sm, ..Default::default() });
            ui.button("Medium");
            ui.button_with("Large", &Button { size: ControlSize::Lg, ..Default::default() });
        });
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            ui.button_with("New project", &Button { variant: ButtonVariant::Primary, icon: Some(cui::ICON_PLUS), ..Default::default() });
            ui.button_with("Download", &Button { icon: Some(cui::ICON_DOWNLOAD), ..Default::default() });
            ui.button_with("Continue", &Button { variant: ButtonVariant::Outline, icon_right: Some(cui::ICON_ARROW_RIGHT), ..Default::default() });
            ui.icon_button_with(cui::ICON_COPY, &Button { tooltip: Some("Copy"), shortcut: Some("Ctrl C"), ..Default::default() });
            ui.icon_button_with(cui::ICON_SHARE_2, &Button { tooltip: Some("Share"), ..Default::default() });
            ui.icon_button_with(cui::ICON_HEART, &Button { tooltip: Some("Like"), variant: ButtonVariant::Outline, ..Default::default() });
            ui.icon_button_with(cui::ICON_TRASH_2, &Button { tooltip: Some("Delete"), variant: ButtonVariant::Secondary, ..Default::default() });
        });
    });
    section(ui, "States", Some("Loading keeps the label so the layout does not jump."));
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            if ui.button_with(
                if g.loading { "Saving" } else { "Save changes" },
                &Button { variant: ButtonVariant::Primary, loading: g.loading, ..Default::default() },
            ) {
                g.loading = true;
            }
            if ui.button("Reset##loading") {
                g.loading = false;
            }
            ui.button_with("Disabled", &Button { disabled: true, ..Default::default() });
            ui.button_with(
                "Selected",
                &Button { variant: ButtonVariant::Ghost, selected: true, icon: Some(cui::ICON_CHECK), ..Default::default() },
            );
        });
    });
    section(ui, "Groups, copy and hold to confirm", Some("Hold the button until it fills to confirm a destructive action."));
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
            ui.button_group_begin();
            ui.button_with("Bold##group", &Button { icon: Some(cui::ICON_BOLD), ..Default::default() });
            ui.button_with("Italic##group", &Button { icon: Some(cui::ICON_ITALIC), ..Default::default() });
            ui.button_with("Underline##group", &Button { icon: Some(cui::ICON_UNDERLINE), ..Default::default() });
            ui.button_group_end();
            ui.row(
                &Row {
                    gap: 6.,
                    bg: Color::SURFACE_MUTED,
                    radius: 18.,
                    pad_left: 14.,
                    pad_right: 4.,
                    pad_y: 2.,
                    ..Default::default()
                },
                |ui| {
                    ui.text_with("npm i cui", &Text { font: cui::font::MONO, nowrap: true, ..Default::default() });
                    ui.copy_button("install", "npm i cui");
                },
            );
            let hold = cui_sys::cui_hold_opts { icon: s(ui, cui::ICON_TRASH_2), ..Default::default() };
            if ui.hold_button("Hold to delete", &hold) {
                ui.toast_with("Deleted", &Toast { tone: Tone::Danger, ..Default::default() });
            }
        });
    });
    section(ui, "Segmented control", Some("A highlight glides to the selected option."));
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        let view_icons: [&str; 3] = [cui::ICON_LIST, cui::ICON_LAYOUT_DASHBOARD, cui::ICON_HISTORY];
        ui.segmented_with("view", &["List", "Board", "Timeline"], &mut g.seg_view, &Segmented { icons: &view_icons, ..Default::default() });
        ui.segmented_with("range", &["Day", "Week", "Month", "Year"], &mut g.seg_size, &Segmented { size: ControlSize::Sm, ..Default::default() });
    });
    section(ui, "Dropdown menu", None);
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            ui.menu_with("Options", &cui::Menu { icon: Some(cui::ICON_SETTINGS), ..Default::default() }, |ui| {
                ui.menu_label("Project");
                ui.menu_item_with("Rename", &cui::MenuItem { icon: Some(cui::ICON_PENCIL), shortcut: Some("F2"), ..Default::default() });
                ui.menu_item_with("Duplicate", &cui::MenuItem { icon: Some(cui::ICON_COPY), shortcut: Some("Ctrl D"), ..Default::default() });
                ui.menu_item_with("Share", &cui::MenuItem { icon: Some(cui::ICON_SHARE_2), ..Default::default() });
                ui.menu_separator();
                if ui.menu_item_with("Delete", &cui::MenuItem { icon: Some(cui::ICON_TRASH_2), danger: true, ..Default::default() }) {
                    g.confirm_open = true;
                }
            });
            ui.menu_with("More actions", &cui::Menu { icon_only: true, icon: Some(cui::ICON_ELLIPSIS), ..Default::default() }, |ui| {
                ui.menu_item_with("Archive", &cui::MenuItem { icon: Some(cui::ICON_ARCHIVE), ..Default::default() });
                ui.menu_item_with("Export", &cui::MenuItem { icon: Some(cui::ICON_DOWNLOAD), ..Default::default() });
            });
        });
    });
}

pub fn page_inputs(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Text fields", Some("Labels above, hints and errors below."));
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
            ui.input_with("name", &mut g.name, &Input { label: Some("Full name"), placeholder: Some("Maya Okafor"), ..Default::default() });
            ui.input_with(
                "email",
                &mut g.email,
                &Input { label: Some("Email"), icon: Some(cui::ICON_AT_SIGN), hint: Some("We never share your email."), ..Default::default() },
            );
            let error = if !g.password.is_empty() && g.password.len() < 8 { Some("Use 8 or more characters.") } else { None };
            ui.input_with(
                "password",
                &mut g.password,
                &Input {
                    label: Some("Password"),
                    password: true,
                    placeholder: Some("At least 8 characters"),
                    error,
                    ..Default::default()
                },
            );
            ui.search_with("search", &mut g.search, &Input { label: Some("Search"), placeholder: Some("Search projects"), ..Default::default() });
        });
        ui.textarea_with("bio", &mut g.bio, &Input { label: Some("About"), height: 96., hint: Some("Markdown is supported."), ..Default::default() });
    });
    section(ui, "Tags, codes and password strength", None);
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        let tags = cui_sys::cui_tag_opts { label: s(ui, "Topics"), placeholder: s(ui, "Add a topic"), ..Default::default() };
        ui.tag_input("tags", &mut g.tags, &tags);
        ui.column(&Column { gap: 8., ..Default::default() }, |ui| {
            ui.label("Verification code");
            if ui.otp_input("otp", &mut g.otp, 6) {
                ui.toast_with("Code entered", &Toast { tone: Tone::Success, ..Default::default() });
            }
        });
        ui.column(&Column { gap: 8., ..Default::default() }, |ui| {
            ui.input_with(
                "new-pw",
                &mut g.password,
                &Input { label: Some("New password"), password: true, placeholder: Some("Try a long passphrase"), ..Default::default() },
            );
            ui.password_strength(&g.password);
        });
    });
    section(ui, "Pickers", None);
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
            ui.select_with(
                "country",
                &["Canada", "Denmark", "Germany", "Iran", "Japan", "Kenya", "Netherlands", "Portugal", "United States"],
                &mut g.country,
                &Select { label: Some("Country"), ..Default::default() },
            );
            ui.select_with(
                "framework",
                &["C99", "C11", "C17", "C23", "C++17", "Objective-C", "Zig", "Odin", "Rust"],
                &mut g.framework,
                &Select { label: Some("Language"), searchable: true, placeholder: Some("Pick a language"), ..Default::default() },
            );
            let date_o = cui_sys::cui_date_opts { label: s(ui, "Due date"), ..Default::default() };
            let mut date = g.date.to_ffi();
            if ui.date_picker("due", &mut date, &date_o) {
                g.date = Date { year: date.year, month: date.month, day: date.day };
            }
            ui.number_with("qty", &mut g.quantity, 0., 99., &Number { label: Some("Seats"), width: Size::FILL, ..Default::default() });
        });
        ui.column(&Column { gap: 6., ..Default::default() }, |ui| {
            ui.label("Brand color");
            let mut color = g.color.to_ffi();
            if ui.color_picker("brand", &mut color) {
                g.color = Color { r: color.r, g: color.g, b: color.b, a: color.a };
            }
        });
    });
}

pub fn page_selection(ui: &mut Ui, g: &mut Gallery) {
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.column(&Column { gap: 12., ..Default::default() }, |ui| {
            section(ui, "Checkbox", None);
            ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
                ui.checkbox_with(
                    "Email notifications",
                    &mut g.newsletter,
                    &Check { description: Some("Product news, at most once a month."), ..Default::default() },
                );
                ui.checkbox_with("Marketing", &mut g.marketing, &Check { description: Some("Offers from our partners."), ..Default::default() });
                ui.checkbox_with("Disabled option", &mut g.terms, &Check { disabled: true, ..Default::default() });
            });
        });
        ui.column(&Column { gap: 12., ..Default::default() }, |ui| {
            section(ui, "Switch", None);
            ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
                ui.switch("Wi-Fi", &mut g.wifi);
                ui.switch("Bluetooth", &mut g.bluetooth);
                ui.last_rect(); // where the Bluetooth switch landed (the smoke test clicks it)
                ui.switch_with("Airplane mode", &mut g.airplane, &Check { description: Some("Turns off all radios."), ..Default::default() });
            });
        });
    });
    section(ui, "Radio group", None);
    ui.card(&Card { gap: 12., ..Default::default() }, |ui| {
        ui.radio_with("Every change", &mut g.plan, 0, &Check { description: Some("Save as you type. Uses the most disk."), ..Default::default() });
        ui.radio_with("Every 5 minutes", &mut g.plan, 1, &Check { description: Some("A good balance for most projects."), ..Default::default() });
        ui.radio_with("Only when I save", &mut g.plan, 2, &Check { description: Some("Nothing is written until you press Ctrl S."), ..Default::default() });
    });
    section(ui, "Radio cards", Some("For choices that need a sentence of explanation."));
    let places = [
        cui_sys::cui_choice { title: s(ui, "This computer"), description: s(ui, "Saved next to the project. Nothing leaves the machine."), icon: s(ui, cui::ICON_MONITOR), badge: core::ptr::null() },
        cui_sys::cui_choice {
            title: s(ui, "Team server"),
            description: s(ui, "Synced to your company server every few minutes."),
            icon: s(ui, cui::ICON_SERVER),
            badge: s(ui, "Default"),
        },
        cui_sys::cui_choice {
            title: s(ui, "Both"),
            description: s(ui, "A local copy, uploaded to the server every night."),
            icon: s(ui, cui::ICON_LAYERS),
            badge: core::ptr::null(),
        },
    ];
    ui.radio_cards("places", &places, &mut g.plan_card, &cui_sys::cui_radio_cards_opts::default());
    section(ui, "Sliders", Some("Drag past either end: the track stretches like rubber."));
    ui.card(&Card { gap: 20., ..Default::default() }, |ui| {
        ui.slider_with("Volume", &mut g.volume, 0., 100., &Slider { show_value: true, ..Default::default() });
        ui.slider_with(
            "Brightness",
            &mut g.brightness,
            0.,
            100.,
            &Slider { step: 10., show_value: true, format: Some("%.0f%%"), ..Default::default() },
        );
        ui.range_slider("Price range", &mut g.lo, &mut g.hi, 0., 100.);
    });
    section(ui, "Chips and rating", None);
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            let chips = ["Design", "Engineering", "Marketing", "Sales", "Support", "Finance"];
            for (i, chip) in chips.iter().enumerate() {
                ui.chip(chip, &mut g.chips[i]);
            }
        });
        ui.rating("rating", &mut g.rating, 5);
    });
}

pub fn page_navigation(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Tabs", Some("The indicator slides to the active tab."));
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.tabs_with("tabs", &["Overview", "Analytics", "Reports", "Settings"], &mut g.tab, &Tabs::default());
        ui.column(&Column { gap: 6., pad_y: 4., ..Default::default() }, |ui| {
            ui.label("Overview");
            ui.secondary("Tabs switch between views of the same thing. Content changes in place.");
        });
        let tab_icons: [&str; 3] = [cui::ICON_USER, cui::ICON_LOCK, cui::ICON_CREDIT_CARD];
        ui.tabs_with(
            "tabs2",
            &["Account", "Password", "Billing"],
            &mut g.tab2,
            &Tabs { variant: TabsVariant::Pill, icons: &tab_icons, ..Default::default() },
        );
    });
    section(ui, "Breadcrumb and pagination", None);
    ui.card(&Card { gap: 18., ..Default::default() }, |ui| {
        let crumbs = ["Home", "Projects", "CUI", "Settings"];
        let c = ui.breadcrumb(&crumbs);
        if c >= 0 {
            ui.toast(crumbs[c as usize]);
        }
        ui.pagination("pages", &mut g.page_index, 12);
    });
    section(ui, "Stepper", None);
    ui.card(&Card { gap: 18., ..Default::default() }, |ui| {
        ui.stepper(&["Account", "Workspace", "Invite team", "Done"], g.step);
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            if ui.button_with("Back", &Button { disabled: g.step == 0, ..Default::default() }) {
                g.step -= 1;
            }
            let label = if g.step >= 3 { "Finish" } else { "Next" };
            if ui.button_with(label, &Button { variant: ButtonVariant::Primary, icon_right: Some(cui::ICON_ARROW_RIGHT), ..Default::default() }) {
                if g.step < 3 {
                    g.step += 1;
                } else {
                    ui.toast_with("All set", &Toast { tone: Tone::Success, ..Default::default() });
                }
            }
        });
    });
}

pub fn page_overlays(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Dialogs and sheets", Some("Esc or a click outside closes them. Focus stays inside."));
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            if ui.button_with("Edit profile", &Button { icon: Some(cui::ICON_PENCIL), ..Default::default() }) {
                g.dialog_open = true;
            }
            if ui.button_with("Open sheet", &Button { icon: Some(cui::ICON_PANEL_LEFT), ..Default::default() }) {
                g.sheet_open = true;
            }
            if ui.button_with("Delete project", &Button { variant: ButtonVariant::Danger, icon: Some(cui::ICON_TRASH_2), ..Default::default() }) {
                g.confirm_open = true;
            }
            if ui.button_with(
                "Command palette",
                &Button {
                    variant: ButtonVariant::Outline,
                    icon: Some(cui::ICON_COMMAND),
                    tooltip: Some("Search everything"),
                    shortcut: Some("Ctrl K"),
                    ..Default::default()
                },
            ) {
                g.palette_open = true;
            }
        });
    });
    section(ui, "Popover and context menu", None);
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            if ui.button_with("Filters", &Button { icon: Some(cui::ICON_FILTER), ..Default::default() }) {
                ui.popover_toggle("filters");
            }
            if ui.popover_begin("filters", &cui_sys::cui_popover_opts { width: 280., ..Default::default() }) {
                ui.label("Filter results");
                ui.checkbox("Only active", &mut g.newsletter);
                ui.checkbox("Include archived", &mut g.marketing);
                ui.slider_with("Minimum score", &mut g.brightness, 0., 100., &Slider { show_value: true, ..Default::default() });
                ui.row(&Row { gap: 8., justify: Justify::End, ..Default::default() }, |ui| {
                    if ui.button_with("Apply", &Button { variant: ButtonVariant::Primary, size: ControlSize::Sm, ..Default::default() }) {
                        ui.popover_close("filters");
                    }
                });
                ui.popover_end();
            }
        });
        ui.card(
            &Card { muted: true, id: Some("ctx-area"), pad: 28., align: Align::Center, hoverable: true, ..Default::default() },
            |ui| {
                ui.icon(cui::ICON_ELLIPSIS, 18., Color::TEXT_MUTED);
                ui.text_with("Right click this area", &Text { color: Color::TEXT_SECONDARY, align: TextAlign::Center, ..Default::default() });
            },
        );
        ui.context_menu("ctx", |ui| {
            ui.menu_item_with("Cut", &cui::MenuItem { icon: Some(cui::ICON_SCISSORS), shortcut: Some("Ctrl X"), ..Default::default() });
            ui.menu_item_with("Copy", &cui::MenuItem { icon: Some(cui::ICON_COPY), shortcut: Some("Ctrl C"), ..Default::default() });
            ui.menu_item_with("Paste", &cui::MenuItem { icon: Some(cui::ICON_CLIPBOARD), shortcut: Some("Ctrl V"), ..Default::default() });
            ui.menu_separator();
            ui.menu_item_with("Delete", &cui::MenuItem { icon: Some(cui::ICON_TRASH_2), danger: true, ..Default::default() });
        });
    });
    section(ui, "Toasts", Some("They stack, expand on hover and pause while you read."));
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            if ui.button("Save profile") {
                ui.toast_with("Changes saved", &Toast { description: Some("Your profile is up to date."), tone: Tone::Success, ..Default::default() });
            }
            if ui.button("Upload a 30 MB file") {
                ui.toast_with("Upload failed", &Toast { description: Some("The file is larger than 25 MB."), tone: Tone::Danger, ..Default::default() });
            }
            if ui.button("Archive a message") {
                ui.toast_with("Message archived", &Toast { action: Some("Undo"), ..Default::default() });
            }
            if ui.button("Check for updates") {
                ui.toast_with("Update available", &Toast { description: Some("Version 1.1 is ready to install."), tone: Tone::Info, ..Default::default() });
            }
        });
    });
    section(ui, "Tooltips", None);
    ui.card(&Card::default(), |ui| {
        ui.row(&Row { gap: 4., wrap: true, ..Default::default() }, |ui| {
            ui.icon_button_with(cui::ICON_BOLD, &Button { tooltip: Some("Bold"), shortcut: Some("Ctrl B"), ..Default::default() });
            ui.icon_button_with(cui::ICON_ITALIC, &Button { tooltip: Some("Italic"), shortcut: Some("Ctrl I"), ..Default::default() });
            ui.icon_button_with(cui::ICON_UNDERLINE, &Button { tooltip: Some("Underline"), shortcut: Some("Ctrl U"), ..Default::default() });
            ui.separator();
            ui.icon_button_with(cui::ICON_ALIGN_LEFT, &Button { tooltip: Some("Align left"), ..Default::default() });
            ui.icon_button_with(cui::ICON_LIST, &Button { tooltip: Some("Bulleted list"), ..Default::default() });
            ui.icon_button_with(cui::ICON_LINK, &Button { tooltip: Some("Insert link"), ..Default::default() });
        });
    });
}

pub fn page_feedback(ui: &mut Ui, _g: &mut Gallery) {
    section(ui, "Alerts", Some("Color only when something succeeded, needs attention or failed."));
    ui.column(&Column { gap: 10., ..Default::default() }, |ui| {
        ui.alert("A new version is available", &cui_sys::cui_alert_opts { description: s(ui, "Restart to update to 1.1."), ..Default::default() });
        ui.alert(
            "Payment received",
            &cui_sys::cui_alert_opts { description: s(ui, "Your invoice was paid on November 5."), tone: Tone::Success as i32, ..Default::default() },
        );
        ui.alert(
            "Storage almost full",
            &cui_sys::cui_alert_opts {
                description: s(ui, "You have used 92% of your plan."),
                tone: Tone::Warning as i32,
                dismissible: true,
                ..Default::default()
            },
        );
        ui.alert(
            "Build failed",
            &cui_sys::cui_alert_opts {
                description: s(ui, "Missing symbol cui_frame in main.c line 42."),
                tone: Tone::Danger as i32,
                ..Default::default()
            },
        );
    });
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.card(&Card { gap: 18., ..Default::default() }, |ui| {
            ui.heading("Progress", 4);
            ui.progress_with(0.68, &Progress { label: Some("Uploading assets"), show_value: true, ..Default::default() });
            ui.progress_with(0.0, &Progress { label: Some("Indexing"), indeterminate: true, ..Default::default() });
            ui.meter("Storage", 18.4, 20., "GB");
            ui.row(&Row { gap: 16., wrap: true, ..Default::default() }, |ui| {
                ui.spinner(16.);
                ui.spinner(22.);
                ui.spinner(28.);
            });
        });
        ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
            ui.heading("Loading", 4);
            ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
                ui.skeleton(Size::px(40.).to_ffi(), 40.);
                ui.column(&Column { gap: 8., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.skeleton(Size::pct(70.).to_ffi(), 12.);
                    ui.skeleton(Size::pct(45.).to_ffi(), 12.);
                });
            });
            ui.skeleton(Size::FILL.to_ffi(), 88.);
            ui.skeleton(Size::pct(90.).to_ffi(), 12.);
            ui.skeleton(Size::pct(60.).to_ffi(), 12.);
        });
    });
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
            ui.heading("Activity", 4);
            ui.timeline_item_with(
                "Deployed to production",
                &TimelineItem { time: Some("2m ago"), description: Some("Release 1.0.3 is live in all regions."), icon: Some(cui::ICON_ROCKET), tone: Tone::Success, ..Default::default() },
            );
            ui.timeline_item_with(
                "Review requested",
                &TimelineItem {
                    time: Some("1h ago"),
                    description: Some("Jonas asked you to review the theming changes."),
                    icon: Some(cui::ICON_MESSAGE_SQUARE),
                    tone: Tone::Info,
                    ..Default::default()
                },
            );
            ui.timeline_item_with(
                "Build failed",
                &TimelineItem { time: Some("3h ago"), icon: Some(cui::ICON_CIRCLE_ALERT), tone: Tone::Danger, last: true, ..Default::default() },
            );
        });
        ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
            ui.heading("Goals", 4);
            ui.row(&Row { gap: 24., justify: Justify::Around, ..Default::default() }, |ui| {
                ui.progress_ring("Onboarding", 0.72, 104.);
                ui.progress_ring("Quota", 0.35, 104.);
            });
        });
    });
    section(ui, "Badges and avatars", None);
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
            ui.badge("Draft");
            ui.badge_with("Recommended", &Badge { tone: Tone::Accent, ..Default::default() });
            ui.badge_with("Active", &Badge { tone: Tone::Success, dot: true, ..Default::default() });
            ui.badge_with("Pending", &Badge { tone: Tone::Warning, dot: true, ..Default::default() });
            ui.badge_with("Overdue", &Badge { tone: Tone::Danger, dot: true, ..Default::default() });
            ui.badge_with("Beta", &Badge { tone: Tone::Violet, ..Default::default() });
            ui.badge_with("New", &Badge { tone: Tone::Info, solid: true, ..Default::default() });
        });
        ui.row(&Row { gap: 16., wrap: true, ..Default::default() }, |ui| {
            ui.avatar_with("Sam Whitfield", &Avatar { size: 40., status: Some("online"), ..Default::default() });
            ui.avatar_with("Daniel Mensah", &Avatar { size: 40., status: Some("busy"), ..Default::default() });
            ui.avatar_with("Rosa Delgado", &Avatar { size: 40., square: true, ..Default::default() });
            let team = ["Kwame Asante", "Hana Novak", "Elif Kaya", "Ben Carter", "Omar Haddad", "Felix Moreau", "Iris Coleman"];
            ui.avatar_group(&team, 4, 32.);
        });
    });
    ui.card(&Card { pad: 0., ..Default::default() }, |ui| {
        if ui.empty_state_with(
            "No projects yet",
            &EmptyState {
                icon: Some(cui::ICON_FOLDER_OPEN),
                description: Some("A project holds your screens, their builds and who can see them."),
                action: Some("New project"),
            },
        ) {
            ui.toast_with("Project created", &Toast { tone: Tone::Success, ..Default::default() });
        }
    });
}

/// One row of the table page's people.
struct PersonRow {
    name: &'static str,
    email: &'static str,
    role: &'static str,
    status: &'static str,
    status_tone: Tone,
    amount: &'static str,
}

const PEOPLE: [PersonRow; 5] = [
    PersonRow { name: "Jonas Berg", email: "jonas@northlight.io", role: "Owner", status: "Active", status_tone: Tone::Success, amount: "$12,384" },
    PersonRow { name: "Priya Nair", email: "priya@harbor.dev", role: "Admin", status: "Active", status_tone: Tone::Success, amount: "$9,846" },
    PersonRow { name: "Tomas Rivera", email: "tomas@kiln.app", role: "Editor", status: "Invited", status_tone: Tone::Warning, amount: "$7,215" },
    PersonRow { name: "Lena Fischer", email: "lena@fischer.dev", role: "Viewer", status: "Active", status_tone: Tone::Success, amount: "$3,127" },
    PersonRow { name: "Aiko Tanaka", email: "aiko@tanaka.design", role: "Editor", status: "Suspended", status_tone: Tone::Danger, amount: "$1,962" },
];

pub fn page_data(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Table", Some("Sortable columns, row selection and components in cells."));
    let mut order = [0usize, 1, 2, 3, 4];
    for i in 1..5 {
        let mut j = i;
        let v = order[i];
        while j > 0 {
            let a = sort_key(&PEOPLE[order[j - 1]], g.sort.column);
            let b = sort_key(&PEOPLE[v], g.sort.column);
            let c = a.cmp(b);
            if if g.sort.descending { c == core::cmp::Ordering::Less } else { c == core::cmp::Ordering::Greater } {
                order[j] = order[j - 1];
                j -= 1;
            } else {
                break;
            }
        }
        order[j] = v;
    }
    let cols = [
        TableColumn { title: "Name", width: Size::FILL, align: TextAlign::Left, sortable: true },
        TableColumn { title: "Email", width: Size::grow(1.2), align: TextAlign::Left, sortable: false },
        TableColumn { title: "Role", width: Size::px(110.), align: TextAlign::Left, sortable: true },
        TableColumn { title: "Status", width: Size::px(120.), align: TextAlign::Left, sortable: true },
        TableColumn { title: "Revenue", width: Size::px(110.), align: TextAlign::Right, sortable: false },
    ];
    let mut sort = g.sort;
    if ui.table_begin(
        "people",
        &cols,
        &mut Table { sort: Some(&mut sort), selectable: true, ..Default::default() },
    ) {
        for &k in order.iter() {
            let r = &PEOPLE[k];
            if ui.table_row(g.selected_row == k) {
                g.selected_row = k;
            }
            ui.table_cell();
            ui.avatar_with(r.name, &Avatar { size: 26., ..Default::default() });
            ui.text_with(r.name, &Text { font: cui::font::MEDIUM, nowrap: true, ..Default::default() });
            ui.table_cell();
            ui.text_with(r.email, &Text { color: Color::TEXT_SECONDARY, nowrap: true, ..Default::default() });
            ui.table_cell_text(r.role);
            ui.table_cell();
            ui.badge_with(r.status, &Badge { tone: r.status_tone, dot: true, ..Default::default() });
            ui.table_cell_text(r.amount);
        }
        ui.table_end();
    }
    g.sort = sort;
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.column(&Column { gap: 12., ..Default::default() }, |ui| {
            section(ui, "Tree view", None);
            ui.card(&Card { pad: 10., gap: 1., ..Default::default() }, |ui| {
                if ui.tree_begin("src", &mut g.tree_src, &cui_sys::cui_tree_opts::default()) {
                    if ui.tree_begin("components", &mut g.tree_ui, &cui_sys::cui_tree_opts::default()) {
                        let files = ["button.c", "dialog.c", "slider.c", "table.c"];
                        for (i, file) in files.iter().enumerate() {
                            let mut open = false;
                            let o = cui_sys::cui_tree_opts { leaf: true, selected: g.selected_file == i, ..Default::default() };
                            ui.tree_begin(*file, &mut open, &o);
                            if ui.tree_clicked() {
                                g.selected_file = i;
                            }
                            ui.tree_end();
                        }
                        ui.tree_end();
                    }
                    if ui.tree_begin("assets", &mut g.tree_assets, &cui_sys::cui_tree_opts::default()) {
                        let mut open = false;
                        let o = cui_sys::cui_tree_opts { leaf: true, icon: s(ui, cui::ICON_IMAGE), ..Default::default() };
                        ui.tree_begin("logo.png", &mut open, &o);
                        ui.tree_end();
                    }
                    let mut open = false;
                    let o = cui_sys::cui_tree_opts { leaf: true, ..Default::default() };
                    ui.tree_begin("main.c", &mut open, &o);
                    ui.tree_end();
                }
            });
        });
        ui.column(&Column { gap: 12., ..Default::default() }, |ui| {
            section(ui, "List", None);
            ui.card(&Card { pad: 8., gap: 2., ..Default::default() }, |ui| {
                ui.list_item_with(
                    "Design review",
                    &cui::ListItem { avatar: Some("Jonas Berg"), subtitle: Some("Tomorrow at 10:00"), trailing: Some("45m"), ..Default::default() },
                );
                ui.list_item_with(
                    "Release 1.0",
                    &cui::ListItem { avatar: Some("Priya Nair"), subtitle: Some("Ship it"), trailing: Some("2h"), selected: true, ..Default::default() },
                );
                ui.list_item_with(
                    "Planning",
                    &cui::ListItem { avatar: Some("Tomas Rivera"), subtitle: Some("Q1 roadmap"), trailing: Some("1h"), ..Default::default() },
                );
            });
        });
    });
    section(ui, "Reorderable list", Some("Drag a row. The others make room as you go."));
    ui.column(&Column { width: Size::FILL, ..Default::default() }, |ui| {
        let tasks = ["Write the release notes", "Record the demo video", "Update the screenshots", "Tag version 1.0", "Post the announcement"];
        let tones = [Tone::Info, Tone::Neutral, Tone::Info, Tone::Warning, Tone::Teal];
        let labels = ["Docs", "Video", "Docs", "Release", "Blog"];
        ui.reorder_begin("tasks", &mut g.task_order);
        for i in 0..5 {
            ui.reorder_item_begin(i);
            let t = g.task_order[i as usize] as usize;
            ui.text_with(tasks[t], &Text { nowrap: true, width: Size::FILL, ..Default::default() });
            ui.badge_with(labels[t], &Badge { tone: tones[t], ..Default::default() });
            ui.reorder_item_end();
        }
        if ui.reorder_end() {
            ui.toast("Order saved");
        }
    });
    section(ui, "Accordion", None);
    ui.card(&Card { pad_y: 4., gap: 0., ..Default::default() }, |ui| {
        if ui.accordion_begin("Is it really dependency free?", &mut g.acc[0]) {
            ui.secondary("Yes. The core is plain C99. Windows uses Win32, macOS uses Cocoa and Linux uses X11. Fonts and icons are embedded.");
            ui.accordion_end();
        }
        if ui.accordion_begin("Can I use my own renderer?", &mut g.acc[1]) {
            ui.secondary("CUI outputs vertices and draw commands. Port the 60 line shader to Vulkan, Metal or Direct3D and you are done.");
            ui.accordion_end();
        }
        if ui.accordion_begin("Does it respect reduced motion?", &mut g.acc[2]) {
            ui.secondary("Every animation has a reduced motion path that keeps the final state and feedback.");
            ui.accordion_end();
        }
    });
    section(ui, "Virtual list", Some("Ten thousand rows. Only the visible ones are built."));
    ui.card(&Card { pad: 6., ..Default::default() }, |ui| {
        let (mut first, mut last) = (0, 0);
        ui.virtual_list_begin("big", 10000, 36., Size::px(220.).to_ffi(), &mut first, &mut last);
        for i in first..last {
            let h = (i as u32).wrapping_add(1).wrapping_mul(2654435761);
            let amount = format!("${}.{:02}", 40 + (h >> 8) % 2400, (h >> 4) % 100);
            ui.row(&Row { height: Size::px(36.), pad_x: 12., gap: 12., ..Default::default() }, |ui| {
                ui.icon(cui::ICON_FILE_TEXT, 16., Color::TEXT_MUTED);
                ui.text_with(format!("Invoice 2026-{:05}", 10000 - i), &Text { nowrap: true, width: Size::FILL, tabular: true, ..Default::default() });
                ui.text_with(amount, &Text { size: 13., tabular: true, nowrap: true, ..Default::default() });
                let due = (h >> 20).is_multiple_of(5);
                ui.text_with(
                    if due { "Due" } else { "Paid" },
                    &Text { size: 12., color: Color::TEXT_MUTED, nowrap: true, width: Size::px(32.), ..Default::default() },
                );
            });
        }
        ui.virtual_list_end();
    });
}

fn sort_key(row: &PersonRow, column: i32) -> &'static str {
    match column {
        0 => row.name,
        2 => row.role,
        _ => row.status,
    }
}

pub fn page_charts(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    const REVENUE: [f32; 12] = [18., 22., 21., 27., 31., 29., 36., 41., 39., 46., 52., 58.];
    const COSTS: [f32; 12] = [12., 13., 15., 14., 18., 19., 21., 22., 24., 25., 27., 29.];
    section(ui, "Line chart", Some("Hover to inspect values. The crosshair glides between points."));
    ui.card(&Card::default(), |ui| {
        let series = [
            Series { name: Some("Revenue"), values: &REVENUE, ..Default::default() },
            Series { name: Some("Costs"), values: &COSTS, ..Default::default() },
        ];
        ui.line_chart_with(
            "revenue",
            &series,
            &Chart { labels: &MONTHS, area: true, smooth: true, height: 240., format: Some("$%.0fk"), ..Default::default() },
        );
    });
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.column(&Column { gap: 12., ..Default::default() }, |ui| {
            section(ui, "Bar chart", None);
            ui.card(&Card::default(), |ui| {
                const A: [f32; 6] = [42., 51., 38., 64., 58., 71.];
                const B: [f32; 6] = [28., 33., 30., 41., 46., 49.];
                let series = [
                    Series { name: Some("Desktop"), values: &A, ..Default::default() },
                    Series { name: Some("Mobile"), values: &B, ..Default::default() },
                ];
                ui.bar_chart_with("visits", &series, &Chart { labels: &MONTHS[..6], height: 200., ..Default::default() });
            });
        });
        ui.column(&Column { gap: 12., ..Default::default() }, |ui| {
            section(ui, "Donut chart", None);
            ui.card(&Card { align: Align::Center, ..Default::default() }, |ui| {
                let shares = [46., 27., 17., 10.];
                ui.donut_chart_with(
                    "platforms",
                    &shares,
                    &["Windows", "macOS", "Linux", "Other"],
                    &Chart { height: 180., format: Some("%.0f%%"), ..Default::default() },
                );
            });
        });
    });
    section(ui, "Stacked bars and gauges", None);
    ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, min_column_width: 260., gap: 16., ..Default::default() }, |ui| {
        ui.card(&Card::default(), |ui| {
            const X: [f32; 8] = [112., 124., 131., 128., 140., 152., 149., 161.];
            const Y: [f32; 8] = [9., 11., 8., 12., 10., 9., 13., 8.];
            const Z: [f32; 8] = [4., 6., 3., 5., 7., 4., 6., 3.];
            let series = [
                Series { name: Some("Passed"), values: &X, ..Default::default() },
                Series { name: Some("Retried"), values: &Y, ..Default::default() },
                Series { name: Some("Failed"), values: &Z, ..Default::default() },
            ];
            let weeks = ["W1", "W2", "W3", "W4", "W5", "W6", "W7", "W8"];
            ui.bar_chart_with("runs", &series, &Chart { labels: &weeks, stacked: true, height: 200., ..Default::default() });
        });
        ui.card(&Card { dir: Dir::Row, gap: 24., align: Align::Center, ..Default::default() }, |ui| {
            ui.gauge("CPU", 0.42, 130.);
            ui.gauge("Memory", 0.81, 130.);
        });
    });
}
