//! Account settings — a 1:1 port of examples/settings.c from the C repo.
//! Section navigation, forms, preferences and a danger zone.

use cui::{
    Align, App, AppDesc, Avatar, Button, ButtonVariant, Card, Check, Color, ControlSize, Dialog,
    Input, Justify, ListItem, Nav, Row, Segmented, Select, Slider, Tone, Ui,
};

#[derive(Default)]
#[allow(dead_code)] // mirrors the C struct's full field set
struct Settings {
    section: i32,
    first: String,
    last: String,
    email: String,
    username: String,
    bio: String,
    website: String,
    timezone: i32,
    language: i32,
    email_comments: bool,
    email_mentions: bool,
    email_digest: bool,
    push_all: bool,
    push_mentions: bool,
    desktop: bool,
    theme: i32,
    density: i32,
    accent: i32,
    font_scale: f32,
    reduced_motion: bool,
    confirm_delete: bool,
    confirm_text: String,
    dirty: bool,
}

const SECTIONS: [&str; 4] = ["Profile", "Notifications", "Appearance", "Account"];
const ICONS: [&str; 4] = [cui::ICON_USER, cui::ICON_BELL, cui::ICON_PALETTE, cui::ICON_SHIELD];
const DESCS: [&str; 4] = ["How others see you.", "Choose what you hear about.", "Make it yours.", "Security and data."];

fn field_row(ui: &mut Ui, title: &str, desc: Option<&str>) {
    ui.column(&cui::Column { width: cui::Size::FILL, min_width: 200., max_width: 240., gap: 3., ..Default::default() }, |ui| {
        ui.label(title);
        if let Some(desc) = desc {
            ui.text_with(desc, &cui::Text { size: 12., color: Color::TEXT_SECONDARY, ..Default::default() });
        }
    });
}

fn profile(ui: &mut Ui, s: &mut Settings) {
    ui.card(&Card { gap: 24., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 16., ..Default::default() }, |ui| {
            ui.avatar_with("Maya Okafor", &Avatar { size: 64., ..Default::default() });
            ui.column(&cui::Column { gap: 8., width: cui::Size::FILL, ..Default::default() }, |ui| {
                ui.label("Profile photo");
                ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
                    ui.button_with("Upload", &Button { icon: Some(cui::ICON_UPLOAD), size: ControlSize::Sm, ..Default::default() });
                    ui.button_with("Remove", &Button { variant: ButtonVariant::Ghost, size: ControlSize::Sm, ..Default::default() });
                });
            });
        });
        ui.separator();
        ui.box_(
            &cui::BoxOpts { dir: cui::Dir::Grid, columns: 2, min_column_width: 220., gap: 16., ..Default::default() },
            |ui| {
                for (id, label, icon) in [
                    ("first", "First name", None),
                    ("last", "Last name", None),
                    ("email", "Email", Some(cui::ICON_MAIL)),
                    ("user", "Username", Some(cui::ICON_AT_SIGN)),
                ] {
                    let error = (id == "user" && s.username.contains(' '))
                        .then_some("Usernames cannot contain spaces.");
                    let value = match id {
                        "first" => &mut s.first,
                        "last" => &mut s.last,
                        "email" => &mut s.email,
                        _ => &mut s.username,
                    };
                    let typed = ui.input_with(
                        id,
                        value,
                        &Input {
                            label: Some(label),
                            icon: icon,
                            hint: (id == "email").then_some("Used for sign in and notifications."),
                            error: error,
                            ..Default::default()
                        },
                    );
                    if typed.changed() {
                        s.dirty = true;
                    }
                }
            },
        );
        let typed = ui.input_with(
            "web",
            &mut s.website,
            &Input {
                label: Some("Website"),
                icon: Some(cui::ICON_GLOBE),
                placeholder: Some("https://"),
                ..Default::default()
            },
        );
        if typed.changed() {
            s.dirty = true;
        }
        let typed = ui.textarea_with(
            "bio",
            &mut s.bio,
            &Input {
                label: Some("Bio"),
                height: 100.,
                hint: Some("A few words about you. Shown on your public profile."),
                ..Default::default()
            },
        );
        if typed.changed() {
            s.dirty = true;
        }
        ui.box_(
            &cui::BoxOpts { dir: cui::Dir::Grid, columns: 2, min_column_width: 220., gap: 16., ..Default::default() },
            |ui| {
                const ZONES: [&str; 6] = [
                    "Pacific Time (UTC-8)", "Eastern Time (UTC-5)", "London (UTC+0)",
                    "Berlin (UTC+1)", "Tehran (UTC+3:30)", "Tokyo (UTC+9)",
                ];
                if ui.select_with(
                    "tz",
                    &ZONES,
                    &mut s.timezone,
                    &cui::Select { label: Some("Time zone"), searchable: true, ..Default::default() },
                ) {
                    s.dirty = true;
                }
                const LANGS: [&str; 6] = ["English", "Deutsch", "Español", "Français", "Italiano", "Português"];
                if ui.select_with("lang", &LANGS, &mut s.language, &cui::Select { label: Some("Language"), ..Default::default() }) {
                    s.dirty = true;
                }
            },
        );
    });
}

fn notifications(ui: &mut Ui, s: &mut Settings) {
    ui.card(&Card { gap: 20., ..Default::default() }, |ui| {
        ui.row(&Row { align: Align::Start, gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Email", Some("What we send to your inbox."));
            ui.column(&cui::Column { gap: 14., width: cui::Size::FILL, ..Default::default() }, |ui| {
                for (label, desc, flag) in [
                    ("Comments", "When someone replies to your comment.", 0),
                    ("Mentions", "When someone mentions you.", 1),
                    ("Weekly digest", "A summary of activity every Monday.", 2),
                ] {
                    let mut v = match flag {
                        0 => s.email_comments,
                        1 => s.email_mentions,
                        _ => s.email_digest,
                    };
                    if ui.checkbox_with(label, &mut v, &Check { description: Some(desc), ..Default::default() }) {
                        s.dirty = true;
                    }
                    match flag {
                        0 => s.email_comments = v,
                        1 => s.email_mentions = v,
                        _ => s.email_digest = v,
                    }
                }
            });
        });
        ui.separator();
        ui.row(&Row { align: Align::Start, gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Push", Some("Notifications on your devices."));
            ui.column(&cui::Column { gap: 14., width: cui::Size::FILL, ..Default::default() }, |ui| {
                if ui.switch("Everything", &mut s.push_all) {
                    s.dirty = true;
                }
                if ui.switch("Mentions only", &mut s.push_mentions) {
                    s.dirty = true;
                }
                if ui.switch_with(
                    "Desktop notifications",
                    &mut s.desktop,
                    &Check { description: Some("Show a banner even when the app is in the background."), ..Default::default() },
                ) {
                    s.dirty = true;
                }
            });
        });
    });
}

fn appearance(ui: &mut Ui, s: &mut Settings) {
    ui.card(&Card { gap: 20., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Theme", Some("Choose how the app looks."));
            const THEMES: [&str; 2] = ["Light", "Dark"];
            const THEME_ICONS: [&str; 2] = [cui::ICON_SUN, cui::ICON_MOON];
            if ui.segmented_with(
                "theme",
                &THEMES,
                &mut s.theme,
                &Segmented { icons: &THEME_ICONS, ..Default::default() },
            ) {
                ui.set_theme(if s.theme == 1 { cui::ThemeMode::Dark } else { cui::ThemeMode::Light });
            }
        });
        ui.separator();
        ui.row(&Row { gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Accent", Some("Used for selection and focus."));
            const ACCENTS: [&str; 8] = ["Neutral", "Violet", "Blue", "Green", "Amber", "Orange", "Coral", "Rose"];
            if ui.select_with(
                "accent",
                &ACCENTS,
                &mut s.accent,
                &cui::Select { width: cui::Size::px(200.), ..Default::default() },
            ) {
                ui.set_accent(cui::Accent::from_index(s.accent));
            }
        });
        ui.separator();
        ui.row(&Row { gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Density", None);
            const DENSITIES: [&str; 2] = ["Comfortable", "Compact"];
            ui.segmented_with(
                "density",
                &DENSITIES,
                &mut s.density,
                &Segmented { size: ControlSize::Sm, ..Default::default() },
            );
        });
        ui.separator();
        ui.row(&Row { gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Text size", Some("Scales all text in the app."));
            if ui.slider_with(
                "##fontscale",
                &mut s.font_scale,
                0.85,
                1.3,
                &Slider { step: 0.05, show_value: true, format: Some("%.2fx"), ..Default::default() },
            ) {
                ui.set_font_scale(s.font_scale);
            }
        });
        ui.separator();
        ui.row(&Row { gap: 24., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            field_row(ui, "Reduce motion", Some("Keep state changes, drop travel and bounce."));
            if ui.switch("##motion", &mut s.reduced_motion) {
                ui.set_reduced_motion(s.reduced_motion);
            }
        });
    });
}

fn account(ui: &mut Ui, s: &mut Settings) {
    ui.card(&Card { gap: 16., ..Default::default() }, |ui| {
        ui.label("Sessions");
        for (name, sub, when) in [
            ("MacBook Pro, Berlin", "This device", "Now"),
            ("Windows desktop, Tehran", "Chrome", "2h ago"),
            ("Linux laptop, Tokyo", "Firefox", "Yesterday"),
        ] {
            ui.list_item_with(
                name,
                &ListItem {
                    icon: Some(cui::ICON_MONITOR),
                    subtitle: Some(sub),
                    trailing: Some(when),
                    ..Default::default()
                },
            );
        }
    });
    ui.card(&Card { gap: 14., ..Default::default() }, |ui| {
        ui.row(&Row { gap: 16., wrap: true, width: cui::Size::FILL, ..Default::default() }, |ui| {
            ui.column(&cui::Column { gap: 4., width: cui::Size::FILL, min_width: 220., ..Default::default() }, |ui| {
                ui.text_with("Delete account", &cui::Text { font: cui::font::MEDIUM, color: Color::DANGER, ..Default::default() });
                ui.secondary("Permanently remove your account and all of its data. This cannot be undone.");
            });
            if ui.button_with("Delete account", &Button { variant: ButtonVariant::Danger, ..Default::default() }) {
                s.confirm_delete = true;
                s.confirm_text.clear();
            }
        });
    });
    #[derive(PartialEq)]
    enum Action {
        None,
        Cancel,
        Delete,
    }
    let mut confirm = s.confirm_delete;
    let mut action = Action::None;
    ui.dialog_with(
        "Delete account",
        &mut confirm,
        &Dialog {
            description: Some("Type DELETE to confirm. Your projects, comments and files will be removed."),
            ..Default::default()
        },
        |ui| {
            ui.input_with(
                "confirm",
                &mut s.confirm_text,
                &Input { placeholder: Some("DELETE"), autofocus: true, ..Default::default() },
            );
            ui.dialog_footer(|ui| {
                if ui.button("Cancel") {
                    action = Action::Cancel;
                }
                if ui.button_with(
                    "Delete account",
                    &Button { variant: ButtonVariant::Danger, disabled: s.confirm_text != "DELETE", ..Default::default() },
                ) {
                    action = Action::Delete;
                    ui.toast_with(
                        "Account scheduled for deletion",
                        &cui::Toast {
                            description: Some("You have 14 days to change your mind."),
                            tone: Tone::Danger,
                            ..Default::default()
                        },
                    );
                }
            });
        },
    );
    if action != Action::None {
        s.confirm_delete = false;
    }
    s.confirm_delete = confirm;
}

fn frame(ui: &mut Ui, s: &mut Settings) {
    let narrow = ui.viewport_size().x < 760.;
    ui.scroll(
        &cui::Scroll { id: Some("page"), width: cui::Size::FILL, fade_edges: true, ..Default::default() },
        |ui| {
            ui.row(&Row { justify: Justify::Center, align: Align::Start, ..Default::default() }, |ui| {
                ui.column(
                    &cui::Column {
                        width: cui::Size::FILL,
                        max_width: 1040.,
                        pad_x: if narrow { 16. } else { 40. },
                        pad_top: if narrow { 20. } else { 40. },
                        pad_bottom: 80.,
                        gap: if narrow { 20. } else { 28. },
                        ..Default::default()
                    },
                    |ui| {
                        ui.column(&cui::Column { gap: 6., ..Default::default() }, |ui| {
                            let crumbs: [&str; 3] = ["Workspace", "Settings", SECTIONS[s.section as usize]];
                            ui.breadcrumb(&crumbs);
                            ui.heading("Settings", 1);
                        });
                        if narrow {
                            ui.tabs_with(
                                "sections",
                                &SECTIONS,
                                &mut s.section,
                                &cui::Tabs { variant: cui::TabsVariant::Pill, icons: &ICONS, ..Default::default() },
                            );
                        }
                        ui.row(&Row { gap: 40., align: Align::Start, ..Default::default() }, |ui| {
                            if !narrow {
                                ui.column(&cui::Column { width: cui::Size::px(220.), gap: 2., ..Default::default() }, |ui| {
                                    for (i, section) in SECTIONS.iter().enumerate() {
                                        if ui.nav_item_with(
                                            section,
                                            &Nav {
                                                icon: Some(ICONS[i]),
                                                selected: s.section == i as i32,
                                                group: Some("sections"),
                                                ..Default::default()
                                            },
                                        ) {
                                            s.section = i as i32;
                                        }
                                    }
                                });
                            }
                            ui.column(&cui::Column { width: cui::Size::FILL, gap: 20., ..Default::default() }, |ui| {
                                ui.column(&cui::Column { gap: 4., ..Default::default() }, |ui| {
                                    ui.heading(SECTIONS[s.section as usize], 3);
                                    ui.secondary(DESCS[s.section as usize]);
                                });
                                match s.section {
                                    0 => profile(ui, s),
                                    1 => notifications(ui, s),
                                    2 => appearance(ui, s),
                                    _ => account(ui, s),
                                }
                                ui.row(&Row { gap: 8., justify: Justify::End, ..Default::default() }, |ui| {
                                    if ui.button_with(
                                        "Discard",
                                        &Button { variant: ButtonVariant::Ghost, disabled: !s.dirty, ..Default::default() },
                                    ) {
                                        s.dirty = false;
                                    }
                                    if ui.button_with(
                                        "Save changes",
                                        &Button {
                                            variant: ButtonVariant::Primary,
                                            disabled: !s.dirty,
                                            icon: Some(cui::ICON_CHECK),
                                            ..Default::default()
                                        },
                                    ) {
                                        s.dirty = false;
                                        ui.toast_with("Settings saved", &cui::Toast { tone: Tone::Success, ..Default::default() });
                                    }
                                });
                            });
                        });
                    },
                );
            });
        },
    );
}

fn main() {
    let mut s = Settings {
        first: "Maya".into(),
        last: "Okafor".into(),
        email: "maya@fieldwork.studio".into(),
        username: "maya.okafor".into(),
        website: "https://fieldwork.studio".into(),
        bio: "Product designer in Berlin. Currently on onboarding and the settings pages.".into(),
        timezone: 3,
        email_comments: true,
        email_mentions: true,
        push_mentions: true,
        font_scale: 1.0,
        ..Default::default()
    };
    let code = App::run(
        &AppDesc { title: Some("Settings"), width: 1180, height: 860, ..Default::default() },
        |ui| frame(ui, &mut s),
    );
    std::process::exit(code);
}
