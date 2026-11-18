//! The extended gallery pages — a 1:1 port of examples/gallery_more.c from
//! the C repo. Each page keeps its own state in the [`More`] struct (C uses
//! function statics), held by the gallery state.

use core::ffi::{c_char, c_void};

use cui::{Align, Color, Dir, Justify, Key, Mods, Size, TextAlign, Tone};
use cui::{
    AgentPlan, AgentStatus, Axis, Badge, BoxOpts, Card, ChatMessage, Column, Composer,
    ControlSize, DotChart, DotDisplay, DotSeries, EventTrack, Input, MenuItem, Model, PlanStep,
    Placement, PromptSuggestion, PromptSuggestions, Reasoning, Readout, Row, Series, Source,
    TaskStatus, Text, TextFx, Toast, ToolCall, Ui, UserMenu,
};

use crate::{section, Gallery};

/// (name, icon) per extended page, in the sidebar's order.
pub const PAGES: [(&str, &str); 18] = [
    ("Actions", cui::ICON_COMMAND),
    ("Smart inputs", cui::ICON_SQUARE_PEN),
    ("Pickers", cui::ICON_CALENDAR),
    ("Capture", cui::ICON_UPLOAD),
    ("Disclosure", cui::ICON_LAYERS),
    ("Reader", cui::ICON_FILE_TEXT),
    ("Data grid", cui::ICON_GRID_3X3),
    ("Code", cui::ICON_CODE),
    ("Social and media", cui::ICON_MESSAGE_SQUARE),
    ("Planning", cui::ICON_LIST_CHECKS),
    ("More charts", cui::ICON_CHART_PIE),
    ("Motion", cui::ICON_ACTIVITY),
    ("AI", cui::ICON_MESSAGE_SQUARE),
    ("Audio", cui::ICON_AUDIO_LINES),
    ("Files", cui::ICON_FOLDER),
    ("Developer tools", cui::ICON_SQUARE_TERMINAL),
    ("More developer tools", cui::ICON_BUG),
    ("Instruments", cui::ICON_CHART_LINE),
];

/// All the extended pages' state (C keeps these in function statics).
pub struct More {
    // actions
    fmt: [bool; 4],
    align: [bool; 3],
    save_status: i32,
    status_at: f64,
    reactions: [cui_sys::cui_reaction; 4],
    dial_choice: i32,
    theme_mode: i32,
    deleted: i32,
    split_choice: i32,
    // smart inputs
    title: String,
    owner: String,
    opacity: f32,
    radius: f32,
    blur: f32,
    amount: f64,
    salary: f64,
    combo: cui_sys::cui_key_combo,
    combo2: cui_sys::cui_key_combo,
    url: String,
    weight: String,
    message: String,
    recent: String,
    tone: i32,
    // pickers
    langs: [bool; 8],
    range: cui_sys::cui_date_range,
    stay: cui_sys::cui_date_range,
    hour: i32,
    minute: i32,
    h2: i32,
    m2: i32,
    day: i32,
    month: i32,
    year: i32,
    dh: i32,
    dm: i32,
    body_height: f32,
    temp: f32,
    price_lo: f32,
    price_hi: f32,
    // capture
    avatar_tex: usize,
    last_dropped: String,
    // disclosure
    banner_open: bool,
    card1: bool,
    card2: bool,
    steps: [cui_sys::cui_check_step; 4],
    checklist_deleted: i32,
    countdown_target: f64,
    // data grid
    people: Vec<Person>,
    order_ix: Vec<usize>,
    people_sel: Vec<bool>,
    people_sort: cui_sys::cui_table_sort,
    grid_last_sort: cui_sys::cui_table_sort,
    filters: [u32; 3],
    grid_search: String,
    // social
    likes: [i32; 3],
    liked: [bool; 3],
    slide: i32,
    textures: Option<(usize, usize)>,
    split: f32,
    deck: usize,
    revenue: f64,
    image_view: cui_sys::cui_image_view,
    crop: cui_sys::cui_rect,
    // planning
    kb_cols: [[i32; 8]; 4],
    kb_len: [i32; 4],
    // motion
    replay: i32,
    peaks: [f32; 96],
    wave_progress: f32,
    tile_wifi: bool,
    tile_hotspot: bool,
    tile_focus: bool,
    tile_airshare: bool,
    tile_dark: bool,
    motion_tab: i32,
    np: cui_sys::cui_now_playing,
    // ai
    draft: String,
    model: i32,
    vote: i32,
    orb_state: i32,
    removed: i32,
    busy: bool,
    started: f64,
    files: [&'static str; 2],
    file_count: i32,
    // audio
    cutoff: f32,
    res: f32,
    attack: f32,
    pan: f32,
    faders: [f32; 6],
    tick: u32,
    tour: i32,
    tour_steps: [cui_sys::cui_tour_step; 3],
    // files
    folder: String,
    open_dlg: bool,
    save_dlg: bool,
    chosen: String,
    term: cui_sys::cui_terminal_buffer,
    term_text: String,
    term_lines: Vec<cui_sys::cui_terminal_line>,
    // devtools
    kv_pairs: [cui_sys::cui_kv_pair; 12],
    kv_n: i32,
    pl_sel: i32,
    qb_fields: [cui_sys::cui_query_field; 4],
    qb_nodes: [cui_sys::cui_query_node; 16],
    qb_n: i32,
    sc_cron: String,
    km_mods: i32,
    cg_sel: i32,
    gv_sel: i32,
    wf_sel: i32,
    tr_tracks: Vec<cui_sys::cui_trace_track>,
    tr_spans: Vec<cui_sys::cui_trace_span>,
    tr_view: cui_sys::cui_trace_view,
    tr_sel: i32,
    // devtools2
    rx_pat: String,
    rx_text: String,
    rq_method: i32,
    rq_url: String,
    rq_params: [cui_sys::cui_kv_pair; 8],
    rq_n: i32,
    ax_sel: i32,
    ff_flags: [cui_sys::cui_feature_flag; 4],
    up_api: [f32; 90],
    up_web: [f32; 90],
    up_db: [f32; 90],
    up_cdn: [f32; 90],
    th_info: [f32; 96],
    th_warn: [f32; 96],
    th_err: [f32; 96],
    th_s0: f64,
    th_s1: f64,
    // instruments
    cups: [f64; 8],
    cup_count: i32,
    cup_selected: i32,
    curve: [[f32; 97]; 8],
}

impl Default for More {
    fn default() -> More {
        // scalars zeroed, heap types empty; `new` seeds the demo values on top
        More {
            fmt: [false; 4],
            align: [false; 3],
            save_status: i32::default(),
            status_at: f64::default(),
            reactions: [
                cui_sys::cui_reaction { icon: s_icon(cui::ICON_THUMBS_UP), count: 12, active: true },
                cui_sys::cui_reaction { icon: s_icon(cui::ICON_HEART), count: 4, active: false },
                cui_sys::cui_reaction { icon: s_icon(cui::ICON_ROCKET), count: 7, active: false },
                cui_sys::cui_reaction { icon: s_icon(cui::ICON_SMILE), count: 2, active: false },
            ],
            dial_choice: i32::default(),
            theme_mode: i32::default(),
            deleted: i32::default(),
            split_choice: i32::default(),
            title: String::new(),
            owner: String::new(),
            opacity: f32::default(),
            radius: f32::default(),
            blur: f32::default(),
            amount: f64::default(),
            salary: f64::default(),
            combo: cui_sys::cui_key_combo::default(),
            combo2: cui_sys::cui_key_combo::default(),
            url: String::new(),
            weight: String::new(),
            message: String::new(),
            recent: String::new(),
            tone: i32::default(),
            langs: [false; 8],
            range: cui_sys::cui_date_range::default(),
            stay: cui_sys::cui_date_range::default(),
            hour: i32::default(),
            minute: i32::default(),
            h2: i32::default(),
            m2: i32::default(),
            day: i32::default(),
            month: i32::default(),
            year: i32::default(),
            dh: i32::default(),
            dm: i32::default(),
            body_height: f32::default(),
            temp: f32::default(),
            price_lo: f32::default(),
            price_hi: f32::default(),
            avatar_tex: usize::default(),
            last_dropped: String::new(),
            banner_open: bool::default(),
            card1: bool::default(),
            card2: bool::default(),
            steps: [cui_sys::cui_check_step::default(); 4],
            checklist_deleted: i32::default(),
            countdown_target: f64::default(),
            people: Vec::new(),
            order_ix: Vec::new(),
            people_sel: Vec::new(),
            people_sort: cui_sys::cui_table_sort::default(),
            grid_last_sort: cui_sys::cui_table_sort::default(),
            filters: [0; 3],
            grid_search: String::new(),
            likes: [0; 3],
            liked: [false; 3],
            slide: i32::default(),
            textures: None,
            split: f32::default(),
            deck: usize::default(),
            revenue: f64::default(),
            image_view: cui_sys::cui_image_view::default(),
            crop: cui_sys::cui_rect::default(),
            kb_cols: [[0; 8]; 4],
            kb_len: [0; 4],
            replay: i32::default(),
            peaks: [0.; 96],
            wave_progress: f32::default(),
            tile_wifi: bool::default(),
            tile_hotspot: bool::default(),
            tile_focus: bool::default(),
            tile_airshare: bool::default(),
            tile_dark: bool::default(),
            motion_tab: i32::default(),
            np: cui_sys::cui_now_playing::default(),
            draft: String::new(),
            model: i32::default(),
            vote: i32::default(),
            orb_state: i32::default(),
            removed: i32::default(),
            busy: bool::default(),
            started: f64::default(),
            files: ["", ""],
            file_count: i32::default(),
            cutoff: f32::default(),
            res: f32::default(),
            attack: f32::default(),
            pan: f32::default(),
            faders: [0.; 6],
            tick: u32::default(),
            tour: i32::default(),
            tour_steps: [cui_sys::cui_tour_step::default(); 3],
            folder: String::new(),
            open_dlg: bool::default(),
            save_dlg: bool::default(),
            chosen: String::new(),
            term: cui_sys::cui_terminal_buffer::default(),
            term_text: String::new(),
            term_lines: Vec::new(),
            kv_pairs: [cui_sys::cui_kv_pair::default(); 12],
            kv_n: i32::default(),
            pl_sel: i32::default(),
            qb_fields: [cui_sys::cui_query_field::default(); 4],
            qb_nodes: [cui_sys::cui_query_node::default(); 16],
            qb_n: i32::default(),
            sc_cron: String::new(),
            km_mods: i32::default(),
            cg_sel: i32::default(),
            gv_sel: i32::default(),
            wf_sel: i32::default(),
            tr_tracks: Vec::new(),
            tr_spans: Vec::new(),
            tr_view: cui_sys::cui_trace_view::default(),
            tr_sel: i32::default(),
            rx_pat: String::new(),
            rx_text: String::new(),
            rq_method: i32::default(),
            rq_url: String::new(),
            rq_params: [cui_sys::cui_kv_pair::default(); 8],
            rq_n: i32::default(),
            ax_sel: i32::default(),
            ff_flags: [cui_sys::cui_feature_flag::default(); 4],
            up_api: [0.; 90],
            up_web: [0.; 90],
            up_db: [0.; 90],
            up_cdn: [0.; 90],
            th_info: [0.; 96],
            th_warn: [0.; 96],
            th_err: [0.; 96],
            th_s0: f64::default(),
            th_s1: f64::default(),
            cups: [0.; 8],
            cup_count: i32::default(),
            cup_selected: i32::default(),
            curve: [[0.; 97]; 8],
        }
    }
}
/// One row of the data grid's demo people.
pub struct Person {
    name: String,
    email: String,
    team: &'static str,
    commits: i32,
    score: f32,
}

impl More {
    /// C seeds its function statics with initializers; we seed a
    /// default-constructed `More` the same way.
    #[allow(clippy::field_reassign_with_default)]
    pub fn new() -> More {
        let mut m = More::default();
        m.title = buf("Q3 roadmap", 128);
        m.owner = buf("Jonas Berg", 64);
        m.opacity = 80.;
        m.radius = 12.;
        m.blur = 24.;
        m.amount = 1249.5;
        m.salary = 98000.;
        m.combo = cui_sys::cui_key_combo { mods: (Mods::PRIMARY | Mods::SHIFT).0, key: Key::P.to_ffi() as i32 };
        m.url = buf("cui.dev", 96);
        m.weight = buf("72", 16);
        m.message = buf("Release notes are up ", 160);
        m.recent = String::with_capacity(256);
        m.langs = [true, false, true, false, false, true, false, false];
        m.range = range(2026, 11, 3, 2026, 11, 14);
        m.stay = range(2026, 12, 18, 2026, 12, 27);
        m.hour = 9;
        m.minute = 30;
        m.h2 = 18;
        m.m2 = 45;
        m.day = 13;
        m.month = 10;
        m.year = 6;
        m.dh = 7;
        m.dm = 25;
        m.body_height = 172.;
        m.temp = 21.5;
        m.price_lo = 40.;
        m.price_hi = 160.;
        m.banner_open = true;
        m.card1 = true;
        m.steps = [
            step("Create your workspace", "Pick a name and invite your team.", "Create", true),
            step("Connect a repository", "Link GitHub or GitLab to sync issues and pull requests.", "Connect", false),
            step("Import your data", "Bring tasks over from a CSV or another tool.", "Import", false),
            step("Turn on notifications", "Get pinged when something needs you.", "Enable", false),
        ];
        m.people_sort = cui_sys::cui_table_sort { column: 3, descending: true };
        m.grid_last_sort = cui_sys::cui_table_sort { column: -1, descending: false };
        m.filters = [1 | 4, 0, 0];
        m.grid_search = String::with_capacity(64);
        m.likes = [12, 3, 0];
        m.liked = [true, false, false];
        m.slide = 1;
        m.split = 0.45;
        m.revenue = 48210.;
        m.kb_cols = [[1, 2, 3, 0, 0, 0, 0, 0], [4, 5, 0, 0, 0, 0, 0, 0], [6, 0, 0, 0, 0, 0, 0, 0], [7, 8, 0, 0, 0, 0, 0, 0]];
        m.kb_len = [3, 2, 1, 2];
        m.wave_progress = 0.36;
        m.tile_wifi = true;
        m.tile_focus = true;
        m.tile_dark = true;
        m.np = cui_sys::cui_now_playing {
            title: str_ptr("Low Tide"),
            artist: str_ptr("Marlow Avenue"),
            art_color: cui_sys::cui_color { r: 0xc8, g: 0x55, b: 0x3d, a: 255 },
            duration: 222.,
            playing: true,
            ..Default::default()
        };
        m.draft = buf("", 512);
        m.model = 0;
        m.orb_state = 1;
        m.removed = -1;
        m.started = -1.;
        m.files = ["quarterly-report.pdf", "metrics.csv"];
        m.file_count = 2;
        m.cutoff = 1800.;
        m.res = 0.3;
        m.attack = 12.;
        m.pan = 0.15;
        m.faders = [-4., -9., 0., -15., -6., -2.];
        m.tour = -1;
        m.tour_steps = [
            tour_step(0, "Turn a knob", "Drag up and down. Hold Shift for fine steps, double-click to reset.", cui_sys::cui_placement::CUI_PLACE_BOTTOM_START),
            tour_step(0, "Watch the levels", "Bars fall back after each peak; the mark above them holds the peak a moment.", cui_sys::cui_placement::CUI_PLACE_TOP_START),
            tour_step(0, "Ride the faders", "Click a fader, then use the wheel or the arrow keys.", cui_sys::cui_placement::CUI_PLACE_LEFT_START),
        ];
        m.folder = buf(".", 1024);
        m.chosen = buf("", 1024);
        m.term_text = String::with_capacity(64 * 1024);
        m.term_lines = vec![cui_sys::cui_terminal_line::default(); 1024];
        m.kv_n = -1;
        m.pl_sel = 3;
        m.qb_fields = [
            field_choice("Status", 0, &["Open", "In progress", "Closed"]),
            field_text("Assignee", 1),
            field_text("Priority", 2),
            field_text("Has PR", 3),
        ];
        m.qb_nodes[0] = node_group(0, true, false);
        m.qb_nodes[1] = node_value(0, false, 0, 0, "Open");
        m.qb_nodes[2] = node_group(1, true, true);
        m.qb_nodes[3] = node_value(1, false, 1, 0, "me");
        m.qb_nodes[4] = node_value(1, false, 2, 2, "2");
        m.qb_n = 5;
        m.sc_cron = buf("30 9 * * 1-5", 64);
        m.km_mods = Mods::PRIMARY.0;
        m.cg_sel = 1;
        m.gv_sel = 1;
        m.wf_sel = 2;
        m.tr_view = cui_sys::cui_trace_view { start: 0.0042, end: 0.0418, mark_start: 0.0161, mark_end: 0.0236, ..Default::default() };
        m.tr_sel = -1;
        m.rx_pat = buf(r"(?<user>[\w.]+)@(?<host>[\w-]+)\.(com|org)", 128);
        m.rx_text = buf(
            "Contact maya.okafor@example.com or jonas@cui-lib.org for access.\nOld address: priya@legacy.net (no longer read).\nSupport: help@example.com",
            1024,
        );
        m.rq_method = 1;
        m.rq_url = buf("https://api.example.com/v1/orders?status=open&limit=20", 256);
        m.rq_n = -1;
        m.ax_sel = 2;
        m.ff_flags = [
            flag("new-checkout", "Single-page checkout with saved cards", [true, true, false], [0, 25, 0]),
            flag("dark-mode-default", "Start new accounts in dark mode", [true, false, false], [0, 0, 0]),
            flag("search-v2", "Ranked search with typo tolerance", [true, true, true], [0, 100, 5]),
            flag("export-pdf", "Export reports as PDF", [false, false, false], [0, 0, 0]),
        ];
        for i in 0..90 {
            m.up_api[i] = 1.;
            m.up_web[i] = 1.;
            m.up_db[i] = 1.;
            m.up_cdn[i] = if i < 20 { -1. } else { 1. };
            if i % 17 == 5 {
                m.up_api[i] = 0.9985;
            }
            if i == 61 {
                m.up_api[i] = 0.962;
            }
            if i == 44 {
                m.up_web[i] = 0.93;
            }
            if i % 23 == 9 {
                m.up_db[i] = 0.995;
            }
            if i == 88 {
                m.up_db[i] = 0.97;
            }
        }
        for i in 0..96 {
            let x = i as f64 / 96.;
            m.th_info[i] = (120. + 80. * (x * core::f64::consts::TAU).sin() + ((i * 37) % 23) as f64) as f32;
            m.th_warn[i] = (12 + (i * 13) % 9 + if (60..70).contains(&i) { 40 } else { 0 }) as f32;
            m.th_err[i] = ((i * 7) % 5 + if (62..67).contains(&i) { 30 } else { 0 }) as f32;
        }
        m.th_s0 = 1794240000. + 3600. * 5.;
        m.th_s1 = 1794240000. + 3600. * 7.5;
        m.cups = [8., 10.75, 12.25, 0., 0., 0., 0., 0.];
        m.cup_count = 3;
        m.cup_selected = 2;
        m
    }
}

// ---- small helpers ---------------------------------------------------------

/// An editable String with its C buffer's capacity — the capacity is the limit.
fn buf(init: &str, cap: usize) -> String {
    let mut s = String::with_capacity(cap);
    s.push_str(init);
    s
}

/// A leaked, NUL-terminated copy of a `&'static str`: the demo data lives as
/// long as the process, like C's string literals. A Rust `&str` is not
/// NUL-terminated, so a plain `as_ptr()` would run off the end.
fn str_ptr(s: &'static str) -> *mut c_char {
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    Box::into_raw(v.into_boxed_slice()) as *mut c_char
}

fn s_icon(icon: &str) -> *const c_char {
    let leaked: &'static str = Box::leak(icon.to_string().into_boxed_str());
    str_ptr(leaked)
}

fn range(y1: i32, m1: i32, d1: i32, y2: i32, m2: i32, d2: i32) -> cui_sys::cui_date_range {
    cui_sys::cui_date_range {
        start: cui_sys::cui_date { year: y1, month: m1, day: d1 },
        end: cui_sys::cui_date { year: y2, month: m2, day: d2 },
    }
}

fn step(title: &'static str, description: &'static str, action: &'static str, done: bool) -> cui_sys::cui_check_step {
    cui_sys::cui_check_step { title: str_ptr(title), description: str_ptr(description), action: str_ptr(action), done }
}

fn tour_step(target: u32, title: &'static str, body: &'static str, placement: cui_sys::cui_placement) -> cui_sys::cui_tour_step {
    cui_sys::cui_tour_step { target, title: str_ptr(title), body: str_ptr(body), placement: placement as i32 }
}

fn flag(key: &'static str, description: &'static str, on: [bool; 3], rollout: [i32; 3]) -> cui_sys::cui_feature_flag {
    let mut o = [false; 8];
    o[..3].copy_from_slice(&on);
    let mut r = [0; 8];
    r[..3].copy_from_slice(&rollout);
    cui_sys::cui_feature_flag { key: str_ptr(key), description: str_ptr(description), on: o, rollout: r }
}

/// A leaked `const char *const *` (one NUL-terminated string per item), the
/// shape C indexes as `options[i]`; lives as long as the process.
fn leak_str_array(items: &[&'static str]) -> *const *const c_char {
    let ptrs: Vec<*const c_char> = items.iter().map(|s| str_ptr(s) as *const c_char).collect();
    Box::leak(ptrs.into_boxed_slice()).as_ptr()
}

fn filter_field(name: &'static str, icon: &'static str, options: &[&'static str]) -> cui_sys::cui_filter_field {
    cui_sys::cui_filter_field {
        name: str_ptr(name),
        icon: s_icon(icon),
        options: leak_str_array(options),
        option_count: options.len() as i32,
    }
}

fn field_choice(name: &'static str, ty: i32, choices: &[&'static str]) -> cui_sys::cui_query_field {
    cui_sys::cui_query_field {
        name: str_ptr(name),
        type_: ty,
        choices: leak_str_array(choices),
        choice_count: choices.len() as i32,
    }
}

fn field_text(name: &'static str, ty: i32) -> cui_sys::cui_query_field {
    cui_sys::cui_query_field { name: str_ptr(name), type_: ty, choices: core::ptr::null(), choice_count: 0 }
}

fn node_group(depth: i32, _enabled: bool, any: bool) -> cui_sys::cui_query_node {
    cui_sys::cui_query_node { depth, group: true, any, field: 0, op: 0, value: [0; 64] }
}

fn node_value(depth: i32, _negated: bool, field: i32, op: i32, value: &'static str) -> cui_sys::cui_query_node {
    cui_sys::cui_query_node {
        depth,
        group: false,
        any: false,
        field,
        op,
        value: {
            // the rule's text lives in the node's own char[64]
            let mut v = [0; 64];
            for (i, b) in value.as_bytes().iter().take(63).enumerate() {
                v[i] = *b as i8;
            }
            v
        },
    }
}

/// `s(ui, "...")` for raw opts string fields: valid for one frame.
fn s(ui: &Ui, text: &str) -> *const c_char {
    ui.arena().cstr(text)
}

// ---- dispatch --------------------------------------------------------------

pub fn page(ui: &mut Ui, g: &mut Gallery, index: usize) {
    let _page_id = ui.push_id(PAGES[index].0);
    match index {
        0 => page_actions(ui, g),
        1 => page_inputs(ui, g),
        2 => page_pickers(ui, g),
        3 => page_capture(ui, g),
        4 => page_disclosure(ui, g),
        5 => page_reader(ui, g),
        6 => page_grid(ui, g),
        7 => page_code(ui, g),
        8 => page_social(ui, g),
        9 => page_planning(ui, g),
        10 => page_viz(ui, g),
        11 => page_motion(ui, g),
        12 => page_ai(ui, g),
        13 => page_audio(ui, g),
        14 => page_files(ui, g),
        15 => page_devtools(ui, g),
        16 => page_devtools2(ui, g),
        17 => page_instruments(ui, g),
        _ => {}
    }
    ui.pop_id();
}

// ---- actions ---------------------------------------------------------------

fn page_actions(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    section(ui, "Split button", Some("A main action with related actions one click away."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        let items = ["Save as draft", "Save and publish", "Schedule…"];
        let icons = [cui::ICON_FILE_TEXT, cui::ICON_SEND, cui::ICON_CLOCK];
        let o = cui_sys::cui_split_button_opts { item_icons: ui.arena().cstr_array(&icons) as *mut _, icon: s(ui, cui::ICON_SAVE), ..Default::default() };
        let r = ui.split_button("Save", &items, &o);
        if r >= 0 {
            m.split_choice = r;
        }
        let o2 = cui_sys::cui_split_button_opts {
            item_icons: ui.arena().cstr_array(&icons) as *mut _,
            icon: s(ui, cui::ICON_DOWNLOAD),
            variant: cui::ButtonVariant::Outline as i32,
            ..Default::default()
        };
        ui.split_button("Export", &items, &o2);
        if m.split_choice >= 0 {
            ui.muted(if m.split_choice == 0 { "Saved".to_string() } else { items[m.split_choice as usize - 1].to_string() });
        }
    });

    section(ui, "Toggle group", Some("Independent toggles for toolbars, or exclusive like radio buttons."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        let labels = ["Bold", "Italic", "Underline", "Code"];
        let icons = [cui::ICON_BOLD, cui::ICON_ITALIC, cui::ICON_UNDERLINE, cui::ICON_CODE];
        let o = cui_sys::cui_toggle_group_opts { icons: ui.arena().cstr_array(&icons) as *mut _, icon_only: true, ..Default::default() };
        ui.toggle_group("format", &labels, &mut m.fmt, &o);
        let alabels = ["Left", "Center", "Right"];
        let aicons = [cui::ICON_ALIGN_LEFT, cui::ICON_LIST, cui::ICON_MENU];
        let o2 = cui_sys::cui_toggle_group_opts { icons: ui.arena().cstr_array(&aicons) as *mut _, exclusive: true, ..Default::default() };
        ui.toggle_group("align", &alabels, &mut m.align, &o2);
    });

    section(ui, "Morphing buttons", Some("Confirm before destroying, and show the result of work in place."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        if ui.confirm_button("Delete project", &cui_sys::cui_confirm_opts { icon: s(ui, cui::ICON_TRASH_2), ..Default::default() }) {
            m.deleted += 1;
        }
        let now = ui.time();
        if ui.status_button(
            "Save changes",
            m.save_status,
            &cui_sys::cui_status_button_opts { icon: s(ui, cui::ICON_SAVE), success_label: s(ui, "Saved"), ..Default::default() },
        ) {
            m.save_status = cui_sys::CUI_STATUS_BUSY;
            m.status_at = now;
        }
        let age = ui.time() - m.status_at;
        if m.save_status == cui_sys::CUI_STATUS_BUSY && age > 1.4 {
            m.save_status = cui_sys::CUI_STATUS_SUCCESS;
            m.status_at = ui.time();
        } else if m.save_status == cui_sys::CUI_STATUS_SUCCESS && age > 1.6 {
            m.save_status = cui_sys::CUI_STATUS_IDLE;
        }
        if m.save_status != cui_sys::CUI_STATUS_IDLE {
            ui.request_redraw();
        }
        if m.deleted != 0 {
            ui.muted("Project deleted");
        }
    });

    section(ui, "Reactions", Some("Counts roll when they change."));
    ui.reactions("reactions", &mut m.reactions);

    section(ui, "Theme", Some("A compact switcher and a single toggle that morphs between sun and moon."));
    ui.row(&Row { gap: 16., wrap: true, ..Default::default() }, |ui| {
        if ui.theme_switcher(&mut m.theme_mode, true) {
            ui.set_theme(if m.theme_mode == 1 { cui::ThemeMode::Dark } else { cui::ThemeMode::Light });
        }
        if ui.theme_toggle() {
            let dark = ui.theme_mode() == cui::ThemeMode::Dark;
            ui.set_theme(if dark { cui::ThemeMode::Light } else { cui::ThemeMode::Dark });
        }
    });

    section(ui, "User menu", Some("Account trigger for sidebars and headers."));
    ui.box_(&BoxOpts { width: Size::px(260.), ..Default::default() }, |ui| {
        ui.user_menu_with(
            "Maya Okafor",
            &UserMenu { detail: Some("maya@fieldwork.studio"), status: Some("online"), placement: Placement::BottomStart, ..Default::default() },
            |ui| {
                ui.menu_item_with("Profile", &MenuItem { icon: Some(cui::ICON_USER), shortcut: Some("Ctrl P"), ..Default::default() });
                ui.menu_item_with("Billing", &MenuItem { icon: Some(cui::ICON_CREDIT_CARD), ..Default::default() });
                ui.menu_item_with("Settings", &MenuItem { icon: Some(cui::ICON_SETTINGS), shortcut: Some("Ctrl ,"), ..Default::default() });
                ui.menu_separator();
                ui.menu_item_with("Log out", &MenuItem { icon: Some(cui::ICON_LOG_OUT), ..Default::default() });
            },
        );
    });

    section(ui, "Swipe actions", Some("Drag a row sideways or swipe with two fingers."));
    ui.card(&Card { pad: 6., gap: 6., ..Default::default() }, |ui| {
        let left = [cui_sys::cui_swipe_action { icon: s_icon(cui::ICON_CHECK), label: str_ptr("Done"), tone: Tone::Success as i32 }];
        let right = [
            cui_sys::cui_swipe_action { icon: s_icon(cui::ICON_ARCHIVE), label: str_ptr("Archive"), tone: Tone::Info as i32 },
            cui_sys::cui_swipe_action { icon: s_icon(cui::ICON_TRASH_2), label: str_ptr("Delete"), tone: Tone::Danger as i32 },
        ];
        let titles = ["Review the design tokens", "Reply to Jonas about the release", "Book the offsite venue"];
        for (i, title) in titles.iter().enumerate() {
            let _row_id = ui.push_id_int(i as i32);
            ui.swipe_item_begin("row", &left, &right);
            ui.icon(cui::ICON_CIRCLE, 16., Color::TEXT_MUTED);
            ui.text_with(*title, &Text { width: Size::FILL, nowrap: true, ..Default::default() });
            ui.text_with("Today", &Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
            let a = ui.swipe_item_end();
            if a >= 0 {
                let (msg, tone) = match a {
                    0 => ("Marked as done", Tone::Success),
                    100 => ("Archived", Tone::Success),
                    _ => ("Deleted", Tone::Danger),
                };
                ui.toast_with(msg, &Toast { tone, ..Default::default() });
            }
        }
    });

    section(ui, "Speed dial", Some("A floating action button that fans out related actions."));
    ui.row(&Row { gap: 16., align: Align::Center, height: Size::px(80.), ..Default::default() }, |ui| {
        let acts = [
            cui_sys::cui_speed_action { icon: s_icon(cui::ICON_FILE_TEXT), label: str_ptr("New note") },
            cui_sys::cui_speed_action { icon: s_icon(cui::ICON_IMAGE), label: str_ptr("Upload image") },
            cui_sys::cui_speed_action { icon: s_icon(cui::ICON_MIC), label: str_ptr("Record memo") },
        ];
        let c = ui.speed_dial("dial", &acts, &cui_sys::cui_speed_dial_opts { tooltip: s(ui, "Create"), ..Default::default() });
        if c >= 0 {
            m.dial_choice = c;
        }
        if m.dial_choice >= 0 {
            ui.muted("New note");
        }
    });
}

// ---- smart inputs ----------------------------------------------------------

fn page_inputs(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    section(ui, "Inline edit", Some("Click the text to edit it in place. Enter saves, Esc cancels."));
    ui.column(&Column { gap: 6., ..Default::default() }, |ui| {
        ui.inline_edit("title", &mut m.title, &cui_sys::cui_inline_edit_opts { font: cui::font::DISPLAY, size: 26., ..Default::default() });
        ui.inline_edit("owner", &mut m.owner, &cui_sys::cui_inline_edit_opts { placeholder: s(ui, "Add an owner"), ..Default::default() });
    });

    section(ui, "Scrub fields", Some("Drag sideways to change the value, Shift for fine steps, click to type."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        ui.scrub("Opacity", &mut m.opacity, 0., 100., &cui_sys::cui_scrub_opts { suffix: s(ui, "%"), ..Default::default() });
        ui.scrub("Radius", &mut m.radius, 0., 48., &cui_sys::cui_scrub_opts { step: 0.5, format: s(ui, "%.1f"), suffix: s(ui, "px"), ..Default::default() });
        ui.scrub("Blur", &mut m.blur, 0., 80., &cui_sys::cui_scrub_opts { suffix: s(ui, "px"), width: Size::px(140.).to_ffi(), ..Default::default() });
    });

    section(ui, "Money", Some("Grouped digits at rest, plain digits while typing."));
    ui.row(&Row { gap: 12., align: Align::Start, ..Default::default() }, |ui| {
        ui.box_(&BoxOpts { width: Size::px(240.), ..Default::default() }, |ui| {
            ui.money_input("amount", &mut m.amount, &cui_sys::cui_money_opts { label: s(ui, "Amount"), code: s(ui, "USD"), ..Default::default() });
        });
        ui.box_(&BoxOpts { width: Size::px(240.), ..Default::default() }, |ui| {
            ui.money_input(
                "salary",
                &mut m.salary,
                &cui_sys::cui_money_opts { label: s(ui, "Salary"), symbol: s(ui, "€"), code: s(ui, "EUR"), decimals: -1, hint: s(ui, "Gross per year"), ..Default::default() },
            );
        });
    });

    section(ui, "Prefix and suffix", Some("Any text field can carry muted text before and after the value."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        ui.input_with("url", &mut m.url, &Input { prefix: Some("https://"), width: Size::px(260.), ..Default::default() });
        ui.input_with("weight", &mut m.weight, &Input { suffix: Some("kg"), width: Size::px(120.), ..Default::default() });
    });

    section(ui, "Shortcut recorder", Some("Click, then press a combination. Backspace clears."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        ui.shortcut_recorder("palette-key", &mut m.combo, &cui_sys::cui_shortcut_recorder_opts::default());
        ui.shortcut_recorder("search-key", &mut m.combo2, &cui_sys::cui_shortcut_recorder_opts { placeholder: s(ui, "Record shortcut"), ..Default::default() });
    });

    section(ui, "Emoji picker", Some("Search by name, jump by group, pick a skin tone. Recent emoji come first."));
    ui.row(&Row { gap: 8., align: Align::Center, width: Size::FILL, max_width: 480., ..Default::default() }, |ui| {
        ui.input("message", &mut m.message);
        if ui.icon_button_with(cui::ICON_SMILE, &cui::Button { tooltip: Some("Emoji"), ..Default::default() }) {
            ui.popover_toggle("emoji-pop");
        }
        if ui.popover_begin(
            "emoji-pop",
            &cui_sys::cui_popover_opts { placement: cui_sys::cui_placement::CUI_PLACE_BOTTOM_END as i32, width: 340., pad: 0., ..Default::default() },
        ) {
            let o = cui_sys::cui_emoji_picker_opts {
                recent: m.recent.as_mut_ptr() as *mut c_char,
                recent_cap: m.recent.capacity(),
                tone: &mut m.tone,
                height: 260.,
                ..Default::default()
            };
            let e = ui.emoji_picker("emoji", &o);
            if !e.is_null() {
                let picked = unsafe { core::ffi::CStr::from_ptr(e) }.to_bytes();
                if m.message.len() + picked.len() < m.message.capacity() {
                    m.message.push_str(&String::from_utf8_lossy(picked));
                    ui.popover_close("emoji-pop");
                }
            }
            ui.popover_end();
        }
    });
}

// ---- pickers ---------------------------------------------------------------

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December",
];

fn page_pickers(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    section(ui, "Multi-select", Some("Chips for the chosen items, checkboxes in the list."));
    let items = ["C", "C++", "Rust", "Zig", "Go", "Swift", "Kotlin", "Odin"];
    ui.row(&Row { gap: 12., align: Align::Start, ..Default::default() }, |ui| {
        ui.box_(&BoxOpts { width: Size::px(320.), ..Default::default() }, |ui| {
            ui.multi_select(
                "langs",
                &items,
                &mut m.langs,
                &cui_sys::cui_multi_select_opts { label: s(ui, "Languages"), searchable: true, ..Default::default() },
            );
        });
    });

    section(ui, "Date range", Some("Presets on the left, two months to pick a start and an end."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        ui.date_range_picker("range", &mut m.range, &cui_sys::cui_date_range_opts { width: Size::px(300.).to_ffi(), ..Default::default() });
        ui.date_range_picker(
            "stay",
            &mut m.stay,
            &cui_sys::cui_date_range_opts { width: Size::px(300.).to_ffi(), no_presets: true, ..Default::default() },
        );
    });
    ui.row(&Row::default(), |ui| {
        ui.card(&Card { pad: 16., ..Default::default() }, |ui| {
            ui.range_calendar("inline-range", &mut m.stay, 2);
        });
    });

    section(ui, "Time", Some("A field with hour and minute columns, 24 hour or 12 hour."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        ui.time_picker("start", &mut m.hour, &mut m.minute, &cui_sys::cui_time_opts::default());
        ui.time_picker("end", &mut m.h2, &mut m.m2, &cui_sys::cui_time_opts { twelve_hour: true, minute_step: 15, ..Default::default() });
    });

    section(ui, "Wheels and dial", Some("Drag, scroll or use the arrow keys. Several wheels make a date reel."));
    ui.row(&Row { gap: 32., align: Align::Center, ..Default::default() }, |ui| {
        let days: Vec<String> = (1..=31).map(|d| d.to_string()).collect();
        let years: Vec<String> = (2020..2032).map(|y| y.to_string()).collect();
        ui.card(&Card { dir: Dir::Row, gap: 0., pad: 8., ..Default::default() }, |ui| {
            let day_refs: Vec<&str> = days.iter().map(|s| s.as_str()).collect();
            let month_refs: Vec<&str> = MONTH_NAMES.to_vec();
            let year_refs: Vec<&str> = years.iter().map(|s| s.as_str()).collect();
            ui.wheel_picker("day", &day_refs, &mut m.day, &cui_sys::cui_wheel_opts { width: Size::px(64.).to_ffi(), loop_: true, join_right: true, ..Default::default() });
            ui.wheel_picker("month", &month_refs, &mut m.month, &cui_sys::cui_wheel_opts { width: Size::px(128.).to_ffi(), loop_: true, join_left: true, join_right: true, ..Default::default() });
            ui.wheel_picker("year", &year_refs, &mut m.year, &cui_sys::cui_wheel_opts { width: Size::px(80.).to_ffi(), join_left: true, ..Default::default() });
        });
        ui.time_dial("dial", &mut m.dh, &mut m.dm, 232.);
    });

    section(ui, "Ruler and histogram", Some("A tape that slides under a fixed marker, and a range over its data."));
    let bins = [2., 4., 7., 12., 18., 26., 31., 29., 34., 40., 37., 30., 24., 22., 18., 15., 11., 9., 7., 5., 4., 3., 2., 1.];
    ui.row(&Row { gap: 24., align: Align::Start, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::FILL, gap: 20., ..Default::default() }, |ui| {
            ui.ruler_slider("Height", &mut m.body_height, 120., 220., &cui_sys::cui_ruler_opts { unit: s(ui, "cm"), ..Default::default() });
            ui.ruler_slider(
                "Temperature",
                &mut m.temp,
                10.,
                30.,
                &cui_sys::cui_ruler_opts { step: 0.5, major_every: 4, spacing: 14., format: s(ui, "%.1f"), unit: s(ui, "°C"), ..Default::default() },
            );
        });
        ui.card(&Card { width: Size::FILL, gap: 6., ..Default::default() }, |ui| {
            ui.histogram_slider("Price", &bins, &mut m.price_lo, &mut m.price_hi, 0., 240., &cui_sys::cui_histogram_opts { format: s(ui, "$%.0f"), ..Default::default() });
            ui.muted("1,204 listings in range");
        });
    });
}

// ---- capture ---------------------------------------------------------------

fn page_capture(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    section(ui, "Dropzone", Some("Drop files from your desktop, or click to browse."));
    let n = ui.dropzone(
        "files",
        &cui_sys::cui_dropzone_opts { description: s(ui, "PNG, JPG or PDF, up to 25 MB"), accept: s(ui, "png;jpg;jpeg;pdf"), ..Default::default() },
    );
    if n > 0 {
        if let Some(f) = ui.dropped_file(0) {
            m.last_dropped = f;
        }
    }
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        ui.box_(&BoxOpts { width: Size::FILL, ..Default::default() }, |ui| {
            ui.dropzone(
                "compact",
                &cui_sys::cui_dropzone_opts { compact: true, title: s(ui, "Attach a file"), description: s(ui, "Any type"), ..Default::default() },
            );
        });
        ui.box_(&BoxOpts { width: Size::FILL, ..Default::default() }, |ui| {
            ui.dropzone(
                "images",
                &cui_sys::cui_dropzone_opts {
                    compact: true,
                    title: s(ui, "Add images"),
                    description: s(ui, "PNG or JPG"),
                    icon: s(ui, cui::ICON_IMAGE),
                    accept: s(ui, "png;jpg;jpeg"),
                    ..Default::default()
                },
            );
        });
    });
    if !m.last_dropped.is_empty() {
        ui.muted(m.last_dropped.clone());
    }

    section(ui, "Avatar upload", Some("Drop an image on the avatar."));
    ui.row(&Row { gap: 20., align: Align::Center, ..Default::default() }, |ui| {
        ui.avatar_upload("avatar", "Maya Okafor", &mut m.avatar_tex, 88.);
        ui.column(&Column { gap: 4., ..Default::default() }, |ui| {
            ui.label("Profile photo");
            ui.secondary("At least 256 by 256 pixels.");
        });
    });

    section(ui, "Signature", Some("Draw with the mouse or a pen. Faster strokes are thinner, like ink."));
    ui.box_(&BoxOpts { width: Size::px(520.), ..Default::default() }, |ui| {
        ui.signature_pad("signature", &cui_sys::cui_signature_opts { height: 180., ..Default::default() });
    });
}

// ---- disclosure ------------------------------------------------------------

fn page_disclosure(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    if ui.banner(
        "CUI 1.1 is out: data grid, kanban and forty new components.",
        &mut m.banner_open,
        &cui_sys::cui_banner_opts { action: s(ui, "Read the notes"), dismissible: true, ..Default::default() },
    ) {
        ui.toast("Opening release notes");
    }
    if !m.banner_open && ui.button_with("Show the banner again", &cui::Button { size: ControlSize::Sm, ..Default::default() }) {
        m.banner_open = true;
    }

    section(ui, "Hover card", Some("Rest the pointer on a name."));
    ui.row(&Row { gap: 6., wrap: true, ..Default::default() }, |ui| {
        ui.text("Assigned to");
        ui.link("@jonas");
        if ui.hover_card_begin("grace-card", &cui_sys::cui_hover_card_opts { width: 300., ..Default::default() }) {
            ui.row(&Row { gap: 12., align: Align::Start, ..Default::default() }, |ui| {
                ui.avatar_with("Jonas Berg", &cui::Avatar { size: 44., status: Some("online"), ..Default::default() });
                ui.column(&Column { gap: 4., width: Size::FILL, ..Default::default() }, |ui| {
                    ui.text_with("Jonas Berg", &Text { font: cui::font::MEDIUM, ..Default::default() });
                    ui.text_with("Engineer on the rendering team. Joined March 2024.", &Text { size: 13., color: Color::TEXT_SECONDARY, ..Default::default() });
                    ui.row(&Row { gap: 6., pad_top: 4., wrap: true, ..Default::default() }, |ui| {
                        ui.icon(cui::ICON_CALENDAR, 12., Color::TEXT_MUTED);
                        ui.text_with("Usually replies within an hour", &Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
                    });
                });
            });
            ui.hover_card_end();
        }
        ui.text("and due Friday.");
    });

    section(ui, "Confirm popover", Some("Ask right where the action is."));
    ui.row(&Row { gap: 12., wrap: true, ..Default::default() }, |ui| {
        if ui.button_with("Delete branch", &cui::Button { icon: Some(cui::ICON_TRASH_2), ..Default::default() }) {
            ui.popover_open("confirm-delete");
        }
        if ui.confirm_popover(
            "confirm-delete",
            &cui_sys::cui_confirm_popover_opts {
                title: s(ui, "Delete feature/grid?"),
                description: s(ui, "The branch and its 12 commits are removed from the remote."),
                confirm_label: s(ui, "Delete branch"),
                danger: true,
                ..Default::default()
            },
        ) {
            m.checklist_deleted += 1;
        }
        if m.checklist_deleted != 0 {
            ui.muted("Branch deleted");
        }
    });

    section(ui, "Expandable text", Some("Long text clamps to a few lines with a fade."));
    ui.expandable_text(
        "about",
        "CUI is a component library for C. It draws everything itself, so an app looks the same on Windows, macOS and Linux, and it \
         ships the parts most apps need: inputs, pickers, menus, dialogs, tables, charts and a command palette. Every component \
         supports the keyboard, light and dark themes and reduced motion. The core is plain C99 with no dependencies, and it \
         renders through OpenGL or a built-in software rasterizer, so it also runs in virtual machines and over remote desktop.",
        &cui_sys::cui_expandable_text_opts { lines: 2, ..Default::default() },
    );

    section(ui, "Expandable cards", Some("A header that opens a body with a spring."));
    ui.column(&Column { gap: 10., ..Default::default() }, |ui| {
        if ui.expandable_card_begin(
            "Billing",
            &mut m.card1,
            &cui_sys::cui_expandable_card_opts {
                description: s(ui, "Pro plan, renews on December 1"),
                icon: s(ui, cui::ICON_CREDIT_CARD),
                badge: s(ui, "Active"),
                badge_tone: Tone::Success as i32,
                ..Default::default()
            },
        ) {
            ui.row(&Row { gap: 40., wrap: true, ..Default::default() }, |ui| {
                ui.column(&Column { gap: 2., ..Default::default() }, |ui| {
                    ui.muted("This month");
                    ui.text_with("$84.00", &Text { font: cui::font::DISPLAY, size: 22., tabular: true, ..Default::default() });
                });
                ui.column(&Column { gap: 2., ..Default::default() }, |ui| {
                    ui.muted("Seats");
                    ui.text_with("7 of 10", &Text { font: cui::font::DISPLAY, size: 22., tabular: true, ..Default::default() });
                });
            });
            ui.row(&Row { gap: 8., wrap: true, ..Default::default() }, |ui| {
                ui.button_with("Change plan", &cui::Button { size: ControlSize::Sm, ..Default::default() });
                ui.button_with("Invoices", &cui::Button { size: ControlSize::Sm, variant: cui::ButtonVariant::Ghost, ..Default::default() });
            });
            ui.expandable_card_end();
        }
        if ui.expandable_card_begin(
            "Security",
            &mut m.card2,
            &cui_sys::cui_expandable_card_opts { description: s(ui, "Two-factor authentication is on"), icon: s(ui, cui::ICON_SHIELD), ..Default::default() },
        ) {
            ui.text_with("Recovery codes were generated 3 months ago.", &Text { color: Color::TEXT_SECONDARY, ..Default::default() });
            ui.expandable_card_end();
        }
    });

    section(ui, "Countdown", Some("Digits roll as the time passes."));
    if m.countdown_target == 0. {
        m.countdown_target = ui.time() + 2. * 86400. + 5. * 3600. + 42. * 60. + 18.;
    }
    let now = ui.time();
    let (launch, sale) = (m.countdown_target - now, m.countdown_target - now - 86400. * 2.);
    ui.row(&Row { gap: 24., align: Align::Center, ..Default::default() }, |ui| {
        ui.countdown("launch", launch, &cui_sys::cui_countdown_opts::default());
        ui.countdown("sale", sale, &cui_sys::cui_countdown_opts { compact: true, hide_days: true, ..Default::default() });
    });

    section(ui, "Onboarding checklist", Some("Progress, with the next step open."));
    ui.box_(&BoxOpts { width: Size::px(460.), ..Default::default() }, |ui| {
        let a = ui.checklist("onboarding", &mut m.steps, &cui_sys::cui_checklist_opts { title: s(ui, "Set up your workspace"), ..Default::default() });
        if a >= 0 {
            let i = a as usize;
            m.steps[i].done = true;
            let title = unsafe { core::ffi::CStr::from_ptr(m.steps[i].title) }.to_string_lossy().into_owned();
            ui.toast_with(title, &Toast { tone: Tone::Success, ..Default::default() });
        }
    });

    section(ui, "Undo toast", Some("A draining line shows how long Undo stays."));
    if ui.button_with("Archive 3 conversations", &cui::Button { icon: Some(cui::ICON_ARCHIVE), ..Default::default() }) {
        ui.toast_with("3 conversations archived", &Toast { action: Some("Undo"), show_timer: true, ..Default::default() });
    }
}

// ---- reader (docs page with a table of contents) ---------------------------

fn page_reader(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    let toc = [
        ("Introduction", "intro", 0),
        ("Install", "install", 0),
        ("With CMake", "cmake", 1),
        ("Without CMake", "nocmake", 1),
        ("First window", "window", 0),
        ("Immediate mode", "imm", 0),
        ("Layout", "layout", 0),
        ("Next steps", "next", 0),
    ];
    let para = "Components are functions. They draw themselves, handle input and return what happened. Your variables are the \
                source of truth, so there is nothing to keep in sync, and conditional interface is just an if statement. ";
    ui.row(&Row { gap: 28., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.scroll(
            &cui::Scroll { id: Some("reader"), height: Size::px(560.), width: Size::FILL, gap: 14., fade_edges: true, ..Default::default() },
            |ui| {
                for (k, (title, anchor, level)) in toc.iter().enumerate() {
                    ui.toc_anchor(anchor);
                    ui.heading(*title, if *level != 0 { 4 } else { 3 });
                    for p in 0..3 {
                        let _p_id = ui.push_id_int((k * 10 + p) as i32);
                        ui.text_with(para, &Text { color: Color::TEXT_SECONDARY, ..Default::default() });
                    }
                }
                ui.gap(200.);
            },
        );
        ui.column(&Column { width: Size::px(200.), gap: 10., ..Default::default() }, |ui| {
            ui.text_with("On this page", &Text { size: 12., font: cui::font::MEDIUM, color: Color::TEXT_MUTED, ..Default::default() });
            let ffi_toc: Vec<cui_sys::cui_toc_item> = toc
                .iter()
                .map(|(title, anchor, level)| cui_sys::cui_toc_item { title: s(ui, title), anchor: s(ui, anchor), level: *level })
                .collect();
            ui.toc("reader", &ffi_toc);
        });
    });
}

// ---- data grid and code ----------------------------------------------------

const FIRST: [&str; 16] = ["Maya", "Priya", "Jonas", "Lena", "Sam", "Tomas", "Aiko", "Daniel", "Rosa", "Omar", "Elif", "Ben", "Nora", "Kwame", "Hana", "Felix"];
const LAST: [&str; 16] = ["Okafor", "Nair", "Berg", "Fischer", "Whitfield", "Rivera", "Tanaka", "Mensah", "Delgado", "Haddad", "Kaya", "Carter", "Lindqvist", "Asante", "Novak", "Moreau"];
const TEAMS: [&str; 5] = ["Engine", "Design", "Platform", "Docs", "Growth"];

fn hash32(k: u32) -> u32 {
    let mut h = k.wrapping_mul(2654435761);
    h ^= h >> 15;
    h = h.wrapping_mul(2246822519);
    h ^= h >> 13;
    h
}

fn make_people(m: &mut More) {
    for k in 0..2000 {
        let h = hash32(k as u32);
        let (f, l) = (FIRST[(h % 16) as usize], LAST[((h >> 4) % 16) as usize]);
        let name = format!("{f} {l}");
        let email = format!("{}{}{}.{}.{}@cui.dev", f.to_lowercase(), ".", l.to_lowercase().chars().next().unwrap_or('x'), ".", (h >> 8) % 90 + 10);
        m.people.push(Person {
            name,
            email,
            team: TEAMS[((h >> 12) % 5) as usize],
            commits: ((h >> 16) % 1400) as i32 + 12,
            score: ((h >> 3) % 1000) as f32 / 10.,
        });
        m.order_ix.push(k);
        m.people_sel.push(false);
    }
    m.people_sort = cui_sys::cui_table_sort { column: 3, descending: true };
}

fn sort_people(m: &mut More) {
    let col = m.people_sort.column;
    let desc = m.people_sort.descending;
    m.order_ix.sort_by(|&a, &b| {
        let (x, y) = (&m.people[a], &m.people[b]);
        let r = match col {
            0 => x.name.cmp(&y.name),
            2 => x.team.cmp(y.team),
            3 => x.commits.cmp(&y.commits),
            _ => x.score.partial_cmp(&y.score).unwrap_or(core::cmp::Ordering::Equal),
        };
        if desc {
            r.reverse()
        } else {
            r
        }
    });
}

fn page_grid(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    if m.people.is_empty() {
        make_people(m);
    }
    if m.people_sort.column != m.grid_last_sort.column || m.people_sort.descending != m.grid_last_sort.descending {
        sort_people(m);
        m.grid_last_sort = m.people_sort;
    }
    let fields = [
        filter_field("Team", cui::ICON_USERS, &["Engine", "Design", "Platform", "Docs", "Growth"]),
        filter_field("Status", cui::ICON_CIRCLE_DOT, &["Active", "Away", "Invited"]),
        filter_field("Role", cui::ICON_KEY_ROUND, &["Owner", "Admin", "Member"]),
    ];
    section(ui, "Filter bar", Some("Chips for active filters, values in a list, and a search."));
    ui.filter_bar("filters", &fields, &mut m.filters[0], &mut m.grid_search);

    section(ui, "Data grid", Some("2,000 rows, virtualized. Sort, resize columns, select with Shift and Ctrl, edit names with a double click."));
    let cols = [
        cui_sys::cui_grid_column { title: s(ui, "Name"), width: 200., align: TextAlign::Left as i32, sortable: true, ..Default::default() },
        cui_sys::cui_grid_column { title: s(ui, "Email"), width: 196., align: TextAlign::Left as i32, ..Default::default() },
        cui_sys::cui_grid_column { title: s(ui, "Team"), width: 110., align: TextAlign::Left as i32, sortable: true, ..Default::default() },
        cui_sys::cui_grid_column { title: s(ui, "Commits"), width: 96., align: TextAlign::Right as i32, sortable: true, ..Default::default() },
        cui_sys::cui_grid_column { title: s(ui, "Score"), width: 130., align: TextAlign::Right as i32, sortable: true, ..Default::default() },
    ];
    let n_sel = m.people_sel.iter().filter(|&&s| s).count();
    let (mut f0, mut l0) = (0, 0);
    let o = cui_sys::cui_grid_opts {
        height: Size::px(420.).to_ffi(),
        selected: m.people_sel.as_mut_ptr(),
        sort: &mut m.people_sort,
        ..Default::default()
    };
    if ui.data_grid_begin("people", &cols, 2000, &mut f0, &mut l0, &o) {
        for r in f0..l0 {
            let ix = m.order_ix[r as usize];
            ui.data_grid_row(r);
            ui.data_grid_cell();
            ui.avatar_with(&m.people[ix].name, &cui::Avatar { size: 24., ..Default::default() });
            ui.data_grid_edit(&mut m.people[ix].name);
            ui.data_grid_cell_text(m.people[ix].email.clone());
            ui.data_grid_cell();
            ui.badge(m.people[ix].team);
            ui.data_grid_cell_text(m.people[ix].commits.to_string());
            ui.data_grid_cell();
            ui.progress_with(m.people[ix].score / 100., &cui::Progress { width: Size::px(70.), height: 5., ..Default::default() });
            ui.text_with(format!("{:.1}", m.people[ix].score), &Text { tabular: true, size: 13., color: Color::TEXT_SECONDARY, ..Default::default() });
        }
        ui.data_grid_end();
    }
    ui.muted(if n_sel > 0 {
        "Rows selected. Press Esc to clear."
    } else {
        "Click a row, Shift+click to select a range, Ctrl+A for all."
    });

    section(ui, "Tree table", Some("Tree cells inside a table: disclosure, indentation, totals."));
    ui.card(&Card { pad: 6., ..Default::default() }, |ui| {
        let (mut open_src, mut open_ui) = (true, false);
        ui.table_begin(
            "tree",
            &[
                cui::TableColumn { title: "Name", width: Size::FILL, ..Default::default() },
                cui::TableColumn { title: "Files", width: Size::px(80.), align: TextAlign::Right, ..Default::default() },
                cui::TableColumn { title: "Size", width: Size::px(100.), align: TextAlign::Right, ..Default::default() },
            ],
            &mut cui::Table::default(),
        );
        ui.table_row(false);
        ui.table_cell();
        ui.tree_cell("src", 0, true, &mut open_src, cui::ICON_FOLDER);
        ui.table_cell_text("42");
        ui.table_cell_text("1.2 MB");
        if open_src {
            ui.table_row(false);
            ui.table_cell();
            ui.tree_cell("ui", 1, true, &mut open_ui, cui::ICON_FOLDER);
            ui.table_cell_text("18");
            ui.table_cell_text("640 KB");
            if open_ui {
                ui.table_row(false);
                ui.table_cell();
                let mut leaf = false;
                ui.tree_cell("button.c", 2, false, &mut leaf, cui::ICON_FILE);
                ui.table_cell_text("1");
                ui.table_cell_text("24 KB");
            }
            ui.table_row(false);
            ui.table_cell();
            let mut leaf = false;
            ui.tree_cell("main.c", 1, false, &mut leaf, cui::ICON_FILE);
            ui.table_cell_text("1");
            ui.table_cell_text("8 KB");
        }
        ui.table_row(false);
        ui.table_cell();
        let mut leaf = false;
        ui.tree_cell("README.md", 0, false, &mut leaf, cui::ICON_FILE_TEXT);
        ui.table_cell_text("1");
        ui.table_cell_text("12 KB");
        ui.table_end();
    });
}

fn page_code(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    section(ui, "Code block", Some("Highlighting for C, C++, JavaScript, TypeScript, Python, Rust, Go, JSON and shell."));
    ui.code_block(
        "c-sample",
        "#include <cui/cui.h>\n\n/* A counter with a button */\nstatic int clicks = 0;\n\nvoid frame(cui_ctx *ui) {\n    if (cui_button(ui, \"Click me\", .variant = CUI_BUTTON_PRIMARY)) clicks++;\n    cui_textf(ui, \"Clicked %d times\", clicks);\n    float ratio = clicks / 10.0f;   // grows with each click\n}\n",
        &cui_sys::cui_code_opts { lang: s(ui, "c"), title: s(ui, "counter.c"), line_numbers: true, highlight_from: 7, ..Default::default() },
    );
    ui.row(&Row { gap: 16., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.box_(&BoxOpts { width: Size::grow(1.), ..Default::default() }, |ui| {
            ui.code_block(
                "py",
                "def fib(n: int) -> int:\n    # the slow way, on purpose\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\n\nprint([fib(k) for k in range(10)])\n",
                &cui_sys::cui_code_opts { lang: s(ui, "python"), title: s(ui, "fib.py"), ..Default::default() },
            );
        });
        ui.box_(&BoxOpts { width: Size::grow(1.), ..Default::default() }, |ui| {
            ui.code_block(
                "sh",
                "git clone https://github.com/yappologistic/CUI.git\ncmake -S CUI -B build\ncmake --build build\n./build/examples/gallery  # run it\n",
                &cui_sys::cui_code_opts { lang: s(ui, "shell"), ..Default::default() },
            );
        });
    });
    section(ui, "JSON viewer", Some("Collapsible, with counts. Click a brace to fold it."));
    ui.json_view(
        "json",
        "{\"name\": \"cui\", \"version\": \"1.1.0\", \"private\": false, \"downloads\": 128400,\n \"platforms\": [\"windows\", \"macos\", \"linux\"],\n \"renderer\": {\"backend\": \"opengl\", \"fallback\": \"software\", \"vsync\": true, \"msaa\": null},\n \"authors\": [{\"name\": \"Maya\", \"commits\": 412}, {\"name\": \"Jonas\", \"commits\": 388}]}",
        &cui_sys::cui_json_opts { expand_depth: 2, ..Default::default() },
    );
    ui.json_view("broken", "{\"ok\": true, \"oops\": [1, 2,}", &cui_sys::cui_json_opts::default());
}

// ---- social and media ------------------------------------------------------

/// Paints a synthetic landscape into an RGBA8 buffer (the demo texture).
fn make_scene(w: usize, h: usize, color: bool) -> Vec<u8> {
    let mut px = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 / w as f32;
            let fy = y as f32 / h as f32;
            let (mut r, mut g, mut b) = (0.56 + fy * 0.34, 0.72 + fy * 0.2, 0.88 + fy * 0.06);
            let sun = (fx - 0.72).powi(2) * 15.4 + (fy - 0.24).powi(2) * 6.0;
            if sun < 0.024 {
                r = 1.0;
                g = 0.97;
                b = 0.86;
            }
            let m1 = 0.5 + 0.1 * (fx * 6.0 + 0.6).sin() + 0.03 * (fx * 23.0).sin();
            let m2 = 0.68 + 0.06 * (fx * 4.0 + 2.1).sin() + 0.02 * (fx * 31.0).sin();
            if fy > m1 {
                r = 0.5;
                g = 0.6;
                b = 0.62 - fy * 0.08;
            }
            if fy > m2 {
                r = 0.3 - (fy - m2) * 0.2;
                g = 0.45 - (fy - m2) * 0.25;
                b = 0.3;
            }
            if !color {
                let l = r * 0.3 + g * 0.59 + b * 0.11;
                let l = l * 0.85 + 0.05;
                r = l;
                g = l;
                b = l;
            }
            let p = &mut px[(y * w + x) * 4..(y * w + x) * 4 + 4];
            p[0] = (r.clamp(0., 1.) * 255.) as u8;
            p[1] = (g.clamp(0., 1.) * 255.) as u8;
            p[2] = (b.clamp(0., 1.) * 255.) as u8;
            p[3] = 255;
        }
    }
    px
}

fn page_social(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    // the demo textures are made once and live for the app's lifetime
    if m.textures.is_none() {
        let a = make_scene(640, 400, false);
        let b = make_scene(640, 400, true);
        let before = ui.image_create(640, 400, &a);
        let after = ui.image_create(640, 400, &b);
        m.textures = Some((before, after));
    }
    let (before, after) = m.textures.unwrap();

    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Chat", Some("Grouped bubbles, read receipts and a typing indicator."));
            ui.card(&Card { gap: 0., pad: 16., ..Default::default() }, |ui| {
                ui.chat_divider("Today");
                ui.chat_message_with(
                    "Did the data grid make it into 1.1?",
                    &ChatMessage { author: Some("Jonas Berg"), time: Some("9:41"), last_in_group: false, ..Default::default() },
                );
                ui.chat_message_with(
                    "I need it for the billing page.",
                    &ChatMessage { author: Some("Jonas Berg"), grouped: true, last_in_group: true, ..Default::default() },
                );
                ui.chat_message_with("It did. 2,000 rows scroll at 60 fps on my laptop.", &ChatMessage { mine: true, ..Default::default() });
                ui.chat_message_with(
                    "Shift and Ctrl clicks select like a spreadsheet.",
                    &ChatMessage { mine: true, grouped: true, last_in_group: true, status: Some("Read"), time: Some("9:43"), ..Default::default() },
                );
                ui.typing_indicator("Jonas");
            });
        });
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Comments", Some("Threads with replies, likes and badges."));
            ui.card(&Card { gap: 4., pad: 14., ..Default::default() }, |ui| {
                let o1 = cui_sys::cui_comment_opts {
                    time: s(ui, "2h"),
                    likes: &mut m.likes[0],
                    liked: &mut m.liked[0],
                    badge: s(ui, "Author"),
                    ..Default::default()
                };
                ui.comment("c1", "Tomas Rivera", "The new numbers match the staging run. I would still test the import on a large file before Friday.", &o1);
                let o2 = cui_sys::cui_comment_opts { time: s(ui, "1h"), depth: 1, likes: &mut m.likes[1], liked: &mut m.liked[1], ..Default::default() };
                ui.comment("c2", "Priya Nair", "Agreed. I have a 2 GB export we can use.", &o2);
                let o3 = cui_sys::cui_comment_opts { time: s(ui, "40m"), depth: 1, likes: &mut m.likes[2], liked: &mut m.liked[2], edited: true, ..Default::default() };
                ui.comment("c3", "Maya Okafor", "Scheduled it for 3 pm.", &o3);
            });
        });
    });
    section(ui, "Presence", Some("Who is here now. Hover to spread them out."));
    ui.row(&Row { gap: 24., align: Align::Center, ..Default::default() }, |ui| {
        let who = [
            presence_user("Maya Okafor", 0x7c5cff),
            presence_user("Jonas Berg", 0x22c58b),
            presence_user("Priya Nair", 0xf59e0b),
            presence_user("Lena Fischer", 0x38bdf8),
            presence_user("Aiko Tanaka", 0xec4899),
            presence_user("Daniel Mensah", 0x10b981),
            presence_user("Rosa Delgado", 0xa855f7),
        ];
        ui.presence("here", &who, 5, 30.);
        ui.counter("rev", m.revenue, &cui_sys::cui_counter_opts { format: s(ui, "$%.0f"), size: 34., group_thousands: true, ..Default::default() });
        if ui.button_with("Add a sale", &cui::Button { size: ControlSize::Sm, icon: Some(cui::ICON_PLUS), ..Default::default() }) {
            m.revenue += 1287.;
        }
    });

    section(ui, "Ticker", Some("A continuous strip of figures. It pauses under the pointer."));
    let tick = [
        ticker_item(ui, "Revenue", "$48.2k", 6.9, false),
        ticker_item(ui, "Active users", "12,408", 3.1, false),
        ticker_item(ui, "Churn", "1.7%", -0.6, true),
        ticker_item(ui, "Latency", "182 ms", -8.4, true),
        ticker_item(ui, "Builds", "1,204", 12.0, false),
        ticker_item(ui, "Uptime", "99.98%", 0.0, false),
    ];
    ui.ticker("tick", &tick, &cui_sys::cui_ticker_opts::default());

    section(ui, "Carousel", Some("Drag, flick, scroll sideways or use the arrows and dots."));
    // SAFETY: `m.slide` lives in the gallery's state for the whole run, past
    // the `carousel_end` a few lines below.
    if unsafe { ui.carousel_begin("tour", 4, &mut m.slide, &cui_sys::cui_carousel_opts { height: 200., auto_advance: 6., ..Default::default() }) } {
        let titles = ["Faster exports", "ARM64 runners", "Shortcuts everywhere", "Smaller downloads"];
        let descs = [
            "A 900 file project now exports in 40 seconds, down from four minutes.",
            "Linux and macOS jobs can run on ARM64 machines in every plan.",
            "Every action in the editor has a key, listed under Help, Shortcuts.",
            "The desktop app is 38 MB lighter after dropping the bundled browser.",
        ];
        let tops = [0x1f2a37u32, 0x0f3d3e, 0x3a2e22, 0x2b2f3a];
        for k in 0..4 {
            ui.carousel_slide_begin();
            ui.box_(
                &BoxOpts {
                    width: Size::FILL,
                    height: Size::FILL,
                    bg: Color::hex(tops[k]),
                    radius: 16.,
                    pad: 28.,
                    gap: 8.,
                    justify: Justify::End,
                    ..Default::default()
                },
                |ui| {
                    ui.text_with(titles[k], &Text { font: cui::font::DISPLAY, size: 24., color: Color::rgb(255, 255, 255), ..Default::default() });
                    ui.text_with(descs[k], &Text { color: Color::hexa(0xffffffc0), ..Default::default() });
                },
            );
            ui.carousel_slide_end();
        }
        ui.carousel_end();
    }

    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, wrap: true, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(3.), min_width: 280., gap: 10., ..Default::default() }, |ui| {
            section(ui, "Image viewer", Some("Wheel to zoom around the pointer, drag to pan, double-click for 100%."));
            ui.image_viewer("viewer", after, 640, 400, &mut m.image_view, &cui_sys::cui_image_viewer_opts { height: Size::px(260.).to_ffi(), ..Default::default() });
        });
        ui.column(&Column { width: Size::grow(2.), min_width: 240., gap: 10., ..Default::default() }, |ui| {
            section(ui, "Cropper", Some("Drag the corners; the square keeps its shape."));
            ui.cropper("cropper", after, 640, 400, &mut m.crop, &cui_sys::cui_cropper_opts { aspect: 1., height: Size::px(260.).to_ffi(), ..Default::default() });
        });
    });
    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Before and after", Some("Drag the divider."));
            ui.image_compare("cmp", before, after, &mut m.split, &cui_sys::cui_compare_opts { height: 230., aspect: 640. / 400., ..Default::default() });
        });
        ui.column(&Column { width: Size::grow(1.), gap: 10., align: Align::Center, ..Default::default() }, |ui| {
            section(ui, "Card stack", Some("Flick the top card away, left or right."));
            let names = ["Kyoto", "Lisbon", "Reykjavik", "Oaxaca", "Hanoi"];
            let cols = [0xc8553du32, 0x2d3047, 0x5b8e7d, 0xb7791f, 0x1f3a5f];
            ui.card_stack_begin("deck", 3, &cui_sys::cui_card_stack_opts { width: Size::px(260.).to_ffi(), height: 200., ..Default::default() });
            for k in 0..3 {
                let idx = (m.deck + k) % 5;
                ui.card_stack_item_begin(k as i32);
                ui.box_(
                    &BoxOpts {
                        width: Size::FILL,
                        height: Size::FILL,
                        bg: Color::hex(cols[idx]),
                        radius: 25.,
                        pad: 20.,
                        justify: Justify::End,
                        gap: 4.,
                        ..Default::default()
                    },
                    |ui| {
                        ui.text_with(names[idx], &Text { font: cui::font::DISPLAY, size: 22., color: Color::rgb(255, 255, 255), ..Default::default() });
                        ui.text_with("Swipe right to save", &Text { size: 13., font: cui::font::MEDIUM, color: Color::hexa(0xfffffff0), ..Default::default() });
                    },
                );
                ui.card_stack_item_end();
            }
            let dir = ui.card_stack_end();
            if dir != 0 {
                m.deck = (m.deck + 1) % 5;
                ui.toast(if dir > 0 { "Saved" } else { "Skipped" });
            }
        });
    });
}

fn presence_user(name: &'static str, hex: u32) -> cui_sys::cui_presence_user {
    cui_sys::cui_presence_user {
        name: str_ptr(name),
        color: cui_sys::cui_color { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8, a: 255 },
        ..Default::default()
    }
}

fn ticker_item(ui: &Ui, label: &'static str, value: &'static str, delta: f32, lower: bool) -> cui_sys::cui_ticker_item {
    cui_sys::cui_ticker_item { label: s(ui, label), value: s(ui, value), delta, lower_is_better: lower }
}

// ---- planning --------------------------------------------------------------

struct KbCard {
    id: i32,
    title: &'static str,
    tag: &'static str,
    who: &'static str,
    tone: Tone,
    comments: i32,
    files: i32,
    progress: f32,
}

const KB_CARDS: [KbCard; 8] = [
    KbCard { id: 1, title: "Audit the color tokens for contrast", tag: "Design", who: "Maya Okafor", tone: Tone::Violet, comments: 3, files: 0, progress: 0. },
    KbCard { id: 2, title: "Virtualize the activity feed", tag: "Perf", who: "Jonas Berg", tone: Tone::Teal, comments: 1, files: 2, progress: 0. },
    KbCard { id: 3, title: "Dark mode screenshots for docs", tag: "Docs", who: "Aiko Tanaka", tone: Tone::Rose, comments: 0, files: 4, progress: 0. },
    KbCard { id: 4, title: "Keyboard nav in the date range picker", tag: "A11y", who: "Priya Nair", tone: Tone::Info, comments: 5, files: 0, progress: 0.6 },
    KbCard { id: 5, title: "IME composition on Windows", tag: "Platform", who: "Daniel Mensah", tone: Tone::Warning, comments: 2, files: 1, progress: 0.3 },
    KbCard { id: 6, title: "Spring presets", tag: "Motion", who: "Rosa Delgado", tone: Tone::Violet, comments: 4, files: 0, progress: 1. },
    KbCard { id: 7, title: "Glyph atlas eviction", tag: "Perf", who: "Lena Fischer", tone: Tone::Teal, comments: 0, files: 0, progress: 1. },
    KbCard { id: 8, title: "Retina screenshots", tag: "Docs", who: "Aiko Tanaka", tone: Tone::Rose, comments: 1, files: 3, progress: 1. },
];

fn kb_apply(m: &mut More, mv: &cui_sys::cui_kanban_move) {
    let (from, to) = (mv.from_column as usize, mv.to_column as usize);
    let mut at = None;
    for k in 0..m.kb_len[from] as usize {
        if m.kb_cols[from][k] == mv.card {
            at = Some(k);
        }
    }
    let Some(at) = at else { return };
    if mv.to_column != mv.from_column && m.kb_len[to] >= 8 {
        return;
    }
    for k in at..m.kb_len[from] as usize - 1 {
        m.kb_cols[from][k] = m.kb_cols[from][k + 1];
    }
    m.kb_len[from] -= 1;
    let idx = (mv.to_index.clamp(0, m.kb_len[to])) as usize;
    for k in (idx..m.kb_len[to] as usize).rev() {
        m.kb_cols[to][k + 1] = m.kb_cols[to][k];
    }
    m.kb_cols[to][idx] = mv.card;
    m.kb_len[to] += 1;
}

fn page_planning(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let col_names = ["Backlog", "In progress", "Review", "Done"];
    let col_dots = [0x94a3b8u32, 0x3b82f6, 0xf59e0b, 0x22c55e];
    section(ui, "Kanban", Some("Drag cards within and between columns. Columns can carry a work-in-progress limit."));
    ui.kanban_begin("board");
    for c in 0..4 {
        let o = cui_sys::cui_kanban_column_opts {
            _pad: 0,
            dot: cui_sys::cui_color { r: (col_dots[c] >> 16) as u8, g: (col_dots[c] >> 8) as u8, b: col_dots[c] as u8, a: 255 },
            limit: if c == 1 { 3 } else { 0 },
            width: 264.,
        };
        if ui.kanban_column_begin(col_names[c], m.kb_len[c], &o) {
            ui.toast_with("New card", &Toast { description: Some(col_names[c]), ..Default::default() });
        }
        for k in 0..m.kb_len[c] as usize {
            let id = m.kb_cols[c][k];
            let Some(cd) = KB_CARDS.iter().find(|x| x.id == id) else { continue };
            if ui.kanban_card_begin(cd.id) {
                ui.toast(cd.title);
            }
            ui.badge_with(cd.tag, &Badge { tone: cd.tone, size: ControlSize::Sm, ..Default::default() });
            ui.text_with(cd.title, &Text { font: cui::font::MEDIUM, size: 13.5, line_height: 1.35, ..Default::default() });
            if cd.progress > 0. && cd.progress < 1. {
                ui.progress_with(cd.progress, &cui::Progress { height: 4., ..Default::default() });
            }
            ui.row(&Row { gap: 10., align: Align::Center, width: Size::FILL, ..Default::default() }, |ui| {
                ui.avatar_with(cd.who, &cui::Avatar { size: 22., ..Default::default() });
                ui.spacer();
                if cd.files != 0 {
                    ui.icon(cui::ICON_PAPERCLIP, 12., Color::TEXT_MUTED);
                    ui.text_with(cd.files.to_string(), &Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
                }
                if cd.comments != 0 {
                    ui.icon(cui::ICON_MESSAGE_SQUARE, 12., Color::TEXT_MUTED);
                    ui.text_with(cd.comments.to_string(), &Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
                }
                if cd.progress >= 1. {
                    ui.icon(cui::ICON_CIRCLE_CHECK, 14., Color::SUCCESS);
                }
            });
            ui.kanban_card_end();
        }
        ui.kanban_column_end();
    }
    let mut mv = cui_sys::cui_kanban_move::default();
    if ui.kanban_end(&mut mv) {
        kb_apply(m, &mv);
    }

    section(ui, "Week calendar", Some("Drag events to move them in 15 minute steps, or pull the bottom edge to resize."));
    let mut events = [
        cal_event("Standup", "Zoom", 0, 9.5, 9.75, Tone::Info),
        cal_event("Design review", "Room 2", 0, 11., 12.5, Tone::Violet),
        cal_event("Lunch with Maya", "", 1, 12.5, 13.5, Tone::Success),
        cal_event("Standup", "Zoom", 1, 9.5, 9.75, Tone::Info),
        cal_event("Release planning", "Room 1", 2, 10., 11.5, Tone::Warning),
        cal_event("1:1 Jonas", "", 2, 10.5, 11., Tone::Rose),
        cal_event("Focus time", "No meetings", 2, 14., 17., Tone::Teal),
        cal_event("Standup", "Zoom", 3, 9.5, 9.75, Tone::Info),
        cal_event("Customer call", "Northlight", 3, 15., 16., Tone::Danger),
        cal_event("Demo day", "All hands", 4, 16., 17.5, Tone::Violet),
        cal_event("Gym", "", 5, 10., 11.5, Tone::Success),
    ];
    let today = cui::date_today();
    let td = cui::date_to_days(today);
    let monday = td - (td + 3).rem_euclid(7);
    let ev = ui.week_calendar(
        "week",
        cui::date_from_days(monday).to_ffi(),
        &mut events,
        &cui_sys::cui_week_opts { start_hour: 8., end_hour: 19., height: Size::px(420.).to_ffi(), ..Default::default() },
    );
    if ev >= 0 {
        let e = &events[ev as usize];
        let detail = unsafe { core::ffi::CStr::from_ptr(e.detail) }.to_string_lossy().into_owned();
        let title: &'static str = Box::leak(unsafe { core::ffi::CStr::from_ptr(e.title) }.to_string_lossy().into_owned().into_boxed_str());
        let desc: Option<&'static str> = if detail.is_empty() { None } else { Some(Box::leak(detail.into_boxed_str())) };
        ui.toast_with(title, &Toast { description: desc, ..Default::default() });
    }

    section(ui, "Gantt", Some("Drag bars to reschedule, pull either end to change the length. Arrows follow dependencies."));
    let mut tasks = [
        gantt_task("Research", "Maya Okafor", 0, 5, 1., Tone::Teal, -1, false),
        gantt_task("Wireframes", "Aiko Tanaka", 4, 10, 1., Tone::Teal, 0, false),
        gantt_task("Design system", "Rosa Delgado", 8, 17, 0.7, Tone::Teal, 1, false),
        gantt_task("Core engine", "Jonas Berg", 6, 20, 0.55, Tone::Info, 0, false),
        gantt_task("Platform backends", "Daniel Mensah", 14, 26, 0.2, Tone::Teal, 3, false),
        gantt_task("Beta", "", 27, 27, 0., Tone::Warning, 4, true),
        gantt_task("Docs and launch", "Priya Nair", 28, 36, 0., Tone::Success, 5, false),
    ];
    let origin = cui::date_from_days(monday - 7);
    let tk = ui.gantt("plan", &mut tasks, origin.to_ffi(), 42, &cui_sys::cui_gantt_opts { today: 7 + (td - monday), ..Default::default() });
    if tk >= 0 {
        let t = &tasks[tk as usize];
        let dur = format!("{} days", t.end - t.start);
        let name: &'static str = Box::leak(unsafe { core::ffi::CStr::from_ptr(t.name) }.to_string_lossy().into_owned().into_boxed_str());
        let desc: &'static str = if t.milestone { "Milestone" } else { Box::leak(dur.into_boxed_str()) };
        ui.toast_with(name, &Toast { description: Some(desc), ..Default::default() });
    }
    section(ui, "Month calendar", Some("Bars for days off and trips, timed events in time order, +N more when a day is full."));
    let now = cui::date_today();
    let base = cui::date_to_days(cui::Date { year: now.year, month: now.month, day: 1 });
    let titles = ["Standup", "Design review", "Offsite", "1:1", "Release", "Planning", "Customer call", "Lunch", "Retro", "Holiday", "Demo", "Board meeting"];
    let mut mev: Vec<cui_sys::cui_month_event> = Vec::with_capacity(12);
    for (k, title) in titles.iter().enumerate() {
        let span = k == 2 || k == 9;
        mev.push(cui_sys::cui_month_event {
            title: s(ui, title),
            start: cui::date_from_days(base + (k as i32 * 5) % 27).to_ffi(),
            end: if span { cui::date_from_days(base + (k as i32 * 5) % 27 + 2).to_ffi() } else { cui_sys::cui_date::default() },
            time: 9.0 + (k % 8) as f32,
            all_day: span || k == 4,
            tone: (1 + k % 8) as i32,
        });
    }
    let mut month = cui_sys::cui_date { year: now.year, month: now.month, day: 1 };
    let mut picked = cui_sys::cui_date::default();
    let mo = cui_sys::cui_month_opts { selected: &mut picked, ..Default::default() };
    let mr = ui.month_calendar("month", &mut month, &mev, &mo);
    if mr.event >= 0 {
        ui.toast_with(titles[mr.event as usize], &Toast { description: Some("Opened from the month calendar"), ..Default::default() });
    }
}

fn cal_event(title: &'static str, detail: &'static str, day: i32, start: f32, end: f32, tone: Tone) -> cui_sys::cui_cal_event {
    cui_sys::cui_cal_event { title: str_ptr(title), detail: str_ptr(detail), day, start, end, tone: tone as i32 }
}

#[allow(clippy::too_many_arguments)]
fn gantt_task(name: &'static str, owner: &'static str, start: i32, end: i32, progress: f32, tone: Tone, depends_on: i32, milestone: bool) -> cui_sys::cui_gantt_task {
    cui_sys::cui_gantt_task {
        name: str_ptr(name),
        owner: str_ptr(owner),
        start,
        end,
        progress,
        tone: tone as i32,
        depends_on,
        milestone,
    }
}

// ---- more charts -----------------------------------------------------------

fn viz(label: &'static str, value: f32) -> cui_sys::cui_viz_item {
    cui_sys::cui_viz_item { label: str_ptr(label), value, color: cui_sys::cui_color::default() }
}

fn page_viz(ui: &mut Ui, g: &mut Gallery) {
    let _m = &mut g.more;
    // a week of traffic by hour, and a year of activity
    let mut traffic = [0f32; 7 * 24];
    for d in 0..7 {
        for h in 0..24 {
            let work = if d < 5 { 1.0 } else { 0.45 };
            let day = (-(((h as f32) - 13.5) / 4.2).powi(2)).exp() * 90. * work + (-(((h as f32) - 20.5) / 2.0).powi(2)).exp() * 35.;
            traffic[d * 24 + h] = day + ((d * 31 + h * 17) % 11) as f32;
        }
    }
    let mut days = [0f32; 364];
    let mut seed: u32 = 7;
    for (k, day) in days.iter_mut().enumerate() {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let wd = (k as i32 + (cui::date_to_days(cui::date_today()) - 363).rem_euclid(7)) as usize % 7;
        let base = (if wd >= 5 { 0.6 } else { 3.2 }) * (0.6 + 0.4 * (k as f32 / 40.).sin() * (k as f32 / 40.).sin());
        let r = ((seed >> 16) % 1000) as f32 / 1000.;
        *day = if r < 0.18 { 0. } else { (base * r * 4.).floor() };
    }
    let wd = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let hours: [&str; 24] = ["0h", "", "", "3h", "", "", "6h", "", "", "9h", "", "", "12h", "", "", "15h", "", "", "18h", "", "", "21h", "", ""];
    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Heatmap", Some("Requests by weekday and hour."));
            ui.card(&Card { pad: 18., width: Size::FILL, ..Default::default() }, |ui| {
                ui.heatmap(
                    "traffic",
                    &traffic,
                    7,
                    24,
                    &cui_sys::cui_heatmap_opts { row_labels: ui.arena().cstr_array(&wd) as *mut _, col_labels: ui.arena().cstr_array(&hours) as *mut _, format: s(ui, "%.0f req"), ..Default::default() },
                );
            });
        });
    });
    section(ui, "Calendar heatmap", Some("A year of activity, one square a day."));
    ui.card(&Card { pad: 18., ..Default::default() }, |ui| {
        let start = cui::date_from_days(cui::date_to_days(cui::date_today()) - 363);
        ui.calendar_heatmap("activity", start.to_ffi(), &days, 364, &cui_sys::cui_calendar_heatmap_opts { unit: s(ui, "contributions"), ..Default::default() });
    });
    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.3), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Treemap", Some("Bundle size by module."));
            let mods = [viz("Layout", 55.), viz("Text", 42.), viz("Draw", 35.), viz("Controls", 31.), viz("Charts", 23.), viz("Overlays", 19.), viz("Platform", 16.), viz("Theme", 9.), viz("Icons", 7.), viz("Grid", 6.)];
            ui.treemap("bundle", &mods, &cui_sys::cui_treemap_opts { height: 280., format: s(ui, "%.0f KB"), ..Default::default() });
        });
        ui.column(&Column { width: Size::grow(1.), gap: 10., align: Align::Center, ..Default::default() }, |ui| {
            section(ui, "Radar", Some("Two releases across six qualities."));
            const RA: [f32; 6] = [82., 64., 90., 58., 75., 88.];
            const RB: [f32; 6] = [70., 85., 72., 80., 60., 79.];
            let axes = ["Speed", "Size", "Polish", "Docs", "Platforms", "Tests"];
            let rs = [
                Series { name: Some("1.0"), values: &RA, ..Default::default() },
                Series { name: Some("1.1"), values: &RB, ..Default::default() },
            ];
            ui.radar_chart("radar", &rs, &axes, &cui_sys::cui_radar_opts { size: 220., max: 100., ..Default::default() });
        });
    });
    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Funnel", Some("Sign up conversion."));
            let stages = [viz("Visited", 12840.), viz("Signed up", 4210.), viz("Activated", 2380.), viz("Subscribed", 860.)];
            ui.funnel("funnel", &stages, &cui_sys::cui_funnel_opts::default());
        });
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Split bar", Some("Where the time goes."));
            let parts = [viz("Layout", 38.), viz("Text", 24.), viz("Paint", 21.), viz("Input", 9.), viz("Other", 8.)];
            ui.split_bar("split", &parts, &cui_sys::cui_split_bar_opts { format: s(ui, "%.0f ms"), ..Default::default() });
            ui.gap(18.);
            section(ui, "Waffle", Some("Platforms of the last 100 installs."));
            let plats = [viz("Windows", 46.), viz("macOS", 31.), viz("Linux", 23.)];
            ui.waffle("waffle", &plats, &cui_sys::cui_waffle_opts { cell: 14., ..Default::default() });
        });
    });
    section(ui, "Sankey", Some("Where sign ups come from and where they end up."));
    let nodes = ["Search", "Social", "Referral", "Sign up", "Bounced", "Trial", "Paid", "Churned"];
    let links = [
        cui_sys::cui_sankey_link { from: 0, to: 3, value: 420. },
        cui_sys::cui_sankey_link { from: 1, to: 3, value: 260. },
        cui_sys::cui_sankey_link { from: 2, to: 3, value: 180. },
        cui_sys::cui_sankey_link { from: 0, to: 4, value: 300. },
        cui_sys::cui_sankey_link { from: 1, to: 4, value: 220. },
        cui_sys::cui_sankey_link { from: 3, to: 5, value: 860. },
        cui_sys::cui_sankey_link { from: 5, to: 6, value: 580. },
        cui_sys::cui_sankey_link { from: 5, to: 7, value: 280. },
    ];
    ui.card(&Card { pad: 18., width: Size::FILL, ..Default::default() }, |ui| {
        ui.sankey("flow", &nodes, &links, &cui_sys::cui_sankey_opts { height: 280., ..Default::default() });
    });
    ui.row(&Row { gap: 24., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.4), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Bar list", Some("Ranked, keyed by label: press Shuffle and the rows slide to their new rank."));
            let mut pages = vec![viz("/docs", 4820.), viz("/", 3920.), viz("/gallery", 2410.), viz("/blog/fast-ui-in-c", 1830.), viz("/download", 1210.), viz("/pricing", 640.)];
            ui.bar_list("pages", &pages, &cui_sys::cui_bar_list_opts::default());
            if ui.button_with("Shuffle", &cui::Button { size: ControlSize::Sm, icon: Some(cui::ICON_SHUFFLE), variant: cui::ButtonVariant::Outline, ..Default::default() }) {
                for p in pages.iter_mut() {
                    p.value = (500 + (p.value as u32 % 7).wrapping_mul(7919) % 5000) as f32;
                }
            }
        });
        ui.column(&Column { width: Size::grow(1.), gap: 10., align: Align::Center, ..Default::default() }, |ui| {
            section(ui, "Activity rings", Some("Past one, a ring keeps winding."));
            let rings = [viz("Move", 0.82), viz("Exercise", 1.24), viz("Stand", 0.58)];
            ui.activity_rings("rings", &rings, &cui_sys::cui_rings_opts { size: 170., center: s(ui, "82%"), caption: s(ui, "of goal"), ..Default::default() });
        });
    });
    section(ui, "Scatter", Some("Frame time against node count, sized by draw calls."));
    let (mut a, mut b): (Vec<cui_sys::cui_vec2>, Vec<cui_sys::cui_vec2>) = (Vec::with_capacity(40), Vec::with_capacity(40));
    let (mut sa, mut sb) = (Vec::with_capacity(40), Vec::with_capacity(40));
    for k in 0..40 {
        let n = 200. + k as f32 * 95.;
        a.push(cui_sys::cui_vec2 { x: n, y: 0.08 + n * 0.00006 + ((k * 37) % 13) as f32 * 0.004 });
        b.push(cui_sys::cui_vec2 { x: n * 0.9 + 150., y: 0.15 + n * 0.00011 + ((k * 53) % 17) as f32 * 0.006 });
        sa.push((10 + (k * 7) % 40) as f32);
        sb.push((20 + (k * 11) % 50) as f32);
    }
    let ss = [
        cui_sys::cui_scatter_series { name: s(ui, "Software"), points: a.as_ptr(), sizes: sa.as_ptr(), count: 40, color: cui_sys::cui_color::default() },
        cui_sys::cui_scatter_series { name: s(ui, "OpenGL"), points: b.as_ptr(), sizes: sb.as_ptr(), count: 40, color: cui_sys::cui_color::default() },
    ];
    ui.card(&Card { pad: 18., width: Size::FILL, ..Default::default() }, |ui| {
        ui.scatter_chart(
            "scatter",
            &ss,
            &cui_sys::cui_scatter_opts { height: 300., x_label: s(ui, "Nodes"), y_label: s(ui, "Frame time (ms)"), y_format: s(ui, "%.2f"), ..Default::default() },
        );
    });
    let bands = [
        cui_sys::cui_gauge_band { to: 60., tone: Tone::Success as i32 },
        cui_sys::cui_gauge_band { to: 85., tone: Tone::Warning as i32 },
        cui_sys::cui_gauge_band { to: 100., tone: Tone::Danger as i32 },
    ];
    ui.card(
        &Card { pad: 18., width: Size::FILL, dir: Dir::Row, wrap: true, gap: 24., align: Align::Center, ..Default::default() },
        |ui| {
            ui.radial_gauge("cpu", 64., 0., 100., &cui_sys::cui_radial_gauge_opts { unit: s(ui, "%"), label: s(ui, "CPU"), bands: bands.as_ptr(), band_count: 3, size: 180., ..Default::default() });
            ui.radial_gauge("temp", 71., 20., 100., &cui_sys::cui_radial_gauge_opts { unit: s(ui, "C"), label: s(ui, "GPU temperature"), show_target: true, target: 85., size: 180., ..Default::default() });
            ui.radial_gauge("mem", 0.62 * 32., 0., 32., &cui_sys::cui_radial_gauge_opts { format: s(ui, "%.1f"), unit: s(ui, "GB"), label: s(ui, "Memory"), no_needle: true, ticks: -1, size: 150., ..Default::default() });
        },
    );
    let mut candles = Vec::with_capacity(2000);
    let mut price = 64f32;
    for k in 0..2000 {
        let o = price;
        let c = o * (1. + (k as f32 * 0.37).sin() * 0.012 + (k as f32 * 0.043).cos() * 0.008);
        candles.push(cui_sys::cui_candle {
            open: o,
            high: o.max(c) * 1.006,
            low: o.min(c) * 0.994,
            close: c,
            volume: 2000. + ((k as u32 * 7919) % 900) as f32 * 10.,
        });
        price = c;
    }
    ui.card(&Card { pad: 18., width: Size::FILL, gap: 10., ..Default::default() }, |ui| {
        ui.label("Price, 2,000 periods");
        ui.candlestick("price", &candles, &cui_sys::cui_candlestick_opts { height: 280., ..Default::default() });
    });
    let _ = &ss; // the scatter series borrow the frame arena; keep them alive till here
}

// ---- motion ----------------------------------------------------------------

fn page_motion(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    if m.peaks[0] == 0. {
        for (k, peak) in m.peaks.iter_mut().enumerate() {
            *peak = 0.18 + 0.8 * ((k as f32 * 0.37).sin() * (k as f32 * 0.11 + 1.3).sin()).abs();
        }
    }
    section(ui, "Text effects", Some("A shimmer for pending work, a typewriter for streamed text, words that settle into place."));
    ui.card(&Card { gap: 18., pad: 20., width: Size::FILL, ..Default::default() }, |ui| {
        ui.row(&Row { gap: 10., align: Align::Center, ..Default::default() }, |ui| {
            ui.loader(&cui_sys::cui_loader_opts { style: cui_sys::CUI_LOADER_DOTS, size: 22., ..Default::default() });
            ui.shimmer_text("Thinking about your question…", &cui_sys::cui_text_fx_opts { size: 15., ..Default::default() });
        });
        let _replay_id = ui.push_id_int(m.replay);
        ui.typewriter("tw", "CUI draws a full frame in a fraction of a millisecond, so text can stream in as fast as it arrives.", &TextFx { caret: true, size: 15., ..Default::default() });
        ui.text_reveal("rv", "The release build finished in 3 minutes 42 seconds.", &cui_sys::cui_text_fx_opts { size: 20., font: cui::font::DISPLAY, ..Default::default() });
        ui.row(&Row { gap: 8., align: Align::Center, ..Default::default() }, |ui| {
            ui.text_with("Deploying to", &Text { font: cui::font::DISPLAY, size: 26., ..Default::default() });
            let words = ["staging.", "eu-west.", "us-east.", "all regions."];
            ui.word_rotate("rot", &words, 2.2, &TextFx { font: cui::font::DISPLAY, size: 26., ..Default::default() });
        });
        if ui.button_with("Replay", &cui::Button { size: ControlSize::Sm, icon: Some(cui::ICON_ROTATE_CCW), variant: cui::ButtonVariant::Outline, ..Default::default() }) {
            m.replay += 1;
        }
    });
    section(ui, "Gradient text and spotlight cards", Some("A light follows the pointer across a card and along its edge. A beam marks the one card that is live."));
    ui.gradient_text("Kiln 2.4", &cui_sys::cui_gradient_text_opts { font: cui::font::DISPLAY, size: 34., animate: true, ..Default::default() });
    ui.row(&Row { gap: 14., align: Align::Stretch, width: Size::FILL, ..Default::default() }, |ui| {
        let titles = ["Software", "OpenGL 3.3", "Your own"];
        let blurbs = ["Plain C into a pixel buffer. No GPU needed.", "The default on desktop. Deploying now.", "The core emits triangles; bring any GPU API."];
        for (k, title) in titles.iter().enumerate() {
            let _spot_id = ui.push_id_int(k as i32);
            ui.spotlight_begin("spot", &cui_sys::cui_spotlight_opts { width: Size::grow(1.).to_ffi(), beam: k == 1, ..Default::default() });
            ui.text_with(*title, &Text { font: cui::font::MEDIUM, size: 15., ..Default::default() });
            ui.text_with(blurbs[k], &Text { size: 13., color: Color::TEXT_SECONDARY, ..Default::default() });
            ui.spotlight_end();
        }
    });
    section(ui, "Loaders", Some("Dots, bars, pulse, orbit and grid."));
    ui.row(&Row { gap: 36., align: Align::Center, ..Default::default() }, |ui| {
        for k in 0..5 {
            ui.loader(&cui_sys::cui_loader_opts { style: k, size: 28., ..Default::default() });
        }
    });
    section(ui, "Dock", Some("Icons grow as the pointer passes."));
    ui.row(&Row { width: Size::FILL, justify: Justify::Center, height: Size::px(110.), align: Align::End, ..Default::default() }, |ui| {
        ui.dock_begin("dock", &cui_sys::cui_dock_opts::default());
        let o1 = cui_sys::cui_dock_item_opts { color: Color::hex(0x3b82f6).to_ffi(), running: true, ..Default::default() };
        if ui.dock_item(cui::ICON_FOLDER, "Files", &o1) {
            ui.toast("Files");
        }
        let o2 = cui_sys::cui_dock_item_opts { color: Color::hex(0x0ea5e9).to_ffi(), badge: 3, running: true, ..Default::default() };
        ui.dock_item(cui::ICON_MAIL, "Mail", &o2);
        let o3 = cui_sys::cui_dock_item_opts { color: Color::hex(0xef4444).to_ffi(), ..Default::default() };
        ui.dock_item(cui::ICON_CALENDAR, "Calendar", &o3);
        let o4 = cui_sys::cui_dock_item_opts { color: Color::hex(0xec4899).to_ffi(), running: true, ..Default::default() };
        ui.dock_item(cui::ICON_MUSIC, "Music", &o4);
        let o5 = cui_sys::cui_dock_item_opts { color: Color::hex(0x27272a).to_ffi(), ..Default::default() };
        ui.dock_item(cui::ICON_SQUARE_TERMINAL, "Terminal", &o5);
        let o6 = cui_sys::cui_dock_item_opts { color: Color::hex(0x8b5cf6).to_ffi(), ..Default::default() };
        ui.dock_item(cui::ICON_PALETTE, "Design", &o6);
        ui.dock_separator();
        ui.dock_item(cui::ICON_SETTINGS, "Settings", &cui_sys::cui_dock_item_opts::default());
        ui.dock_item(cui::ICON_TRASH_2, "Trash", &cui_sys::cui_dock_item_opts::default());
        ui.dock_end();
    });
    ui.row(&Row { gap: 28., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Control tiles", Some("Big toggles for quick settings."));
            ui.box_(&BoxOpts { dir: Dir::Grid, columns: 2, gap: 10., width: Size::px(300.), ..Default::default() }, |ui| {
                let wifi_detail = if m.tile_wifi { "Home" } else { "Off" };
                ui.control_tile("Wi-Fi", &mut m.tile_wifi, &cui_sys::cui_tile_opts { icon: s(ui, cui::ICON_WIFI), detail: s(ui, wifi_detail), ..Default::default() });
                ui.control_tile("Hotspot", &mut m.tile_hotspot, &cui_sys::cui_tile_opts { icon: s(ui, cui::ICON_RADIO), ..Default::default() });
                let focus_detail = if m.tile_focus { "Until 6 pm" } else { "Off" };
                ui.control_tile("Focus", &mut m.tile_focus, &cui_sys::cui_tile_opts { icon: s(ui, cui::ICON_MOON), detail: s(ui, focus_detail), ..Default::default() });
                let air_detail = if m.tile_airshare { "Everyone" } else { "Contacts only" };
                ui.control_tile("AirShare", &mut m.tile_airshare, &cui_sys::cui_tile_opts { icon: s(ui, cui::ICON_SHARE_2), detail: s(ui, air_detail), ..Default::default() });
            });
            ui.control_tile("Dark appearance", &mut m.tile_dark, &cui_sys::cui_tile_opts { icon: s(ui, cui::ICON_SUN), wide: true, ..Default::default() });
        });
        ui.column(&Column { width: Size::grow(1.), gap: 10., ..Default::default() }, |ui| {
            section(ui, "Now playing", Some("Artwork, a scrubber and transport."));
            let ev = ui.now_playing("np", &mut m.np, &cui_sys::cui_now_playing_opts::default());
            if ev & cui_sys::CUI_PLAYER_NEXT != 0 {
                ui.toast("Next track");
            }
            if m.np.playing {
                m.np.position += ui.delta_time();
                if m.np.position > m.np.duration {
                    m.np.position = 0.;
                }
            }
            section(ui, "Waveform", Some("Click or drag to seek."));
            ui.card(&Card { pad: 14., gap: 8., width: Size::px(360.), ..Default::default() }, |ui| {
                ui.row(&Row { gap: 12., align: Align::Center, width: Size::FILL, ..Default::default() }, |ui| {
                    ui.icon_button_with(cui::ICON_PLAY, &cui::Button { size: ControlSize::Sm, variant: cui::ButtonVariant::Secondary, ..Default::default() });
                    ui.box_(&BoxOpts { width: Size::FILL, ..Default::default() }, |ui| {
                        ui.waveform("wave", &m.peaks, &mut m.wave_progress, &cui_sys::cui_waveform_opts { height: 34., ..Default::default() });
                    });
                    ui.text_with("0:42", &Text { size: 12., tabular: true, color: Color::TEXT_MUTED, ..Default::default() });
                });
            });
            section(ui, "Tab bar", Some("For compact layouts."));
            let tabs = [
                cui_sys::cui_tab_bar_item { icon: s(ui, cui::ICON_HOUSE), label: s(ui, "Home"), badge: 0 },
                cui_sys::cui_tab_bar_item { icon: s(ui, cui::ICON_SEARCH), label: s(ui, "Search"), badge: 0 },
                cui_sys::cui_tab_bar_item { icon: s(ui, cui::ICON_INBOX), label: s(ui, "Inbox"), badge: 4 },
                cui_sys::cui_tab_bar_item { icon: s(ui, cui::ICON_USER), label: s(ui, "Profile"), badge: 0 },
            ];
            ui.tab_bar("tabs", &tabs, &mut m.motion_tab, &cui_sys::cui_tab_bar_opts { width: Size::px(360.).to_ffi(), ..Default::default() });
        });
    });
}

// ---- AI --------------------------------------------------------------------

fn page_ai(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let now = ui.time();
    if m.busy && m.started < 0. {
        m.started = now;
    }
    if !m.busy {
        m.started = -1.;
    }
    let elapsed = if m.busy { (now - m.started) as f32 } else { 0. };
    if m.busy && elapsed > 9. {
        m.busy = false;
    }
    ui.row(&Row { gap: 28., align: Align::Start, width: Size::FILL, ..Default::default() }, |ui| {
        ui.column(&Column { width: Size::grow(1.6), gap: 14., ..Default::default() }, |ui| {
            section(ui, "Conversation", Some("Reasoning, tool calls, an answer with sources, and the composer."));
            ui.chat_message_with("Why do immediate mode toolkits feel so fast? Cite a few sources.", &ChatMessage { mine: true, ..Default::default() });
            ui.reasoning("think", &Reasoning { thinking: m.busy, seconds: if m.busy { elapsed } else { 8. }, ..Default::default() }, |ui| {
                ui.text_with("The question is about perceived speed. Two parts: how little work a frame does, and how quickly input shows up on screen.", &Text { size: 13.5, color: Color::TEXT_SECONDARY, ..Default::default() });
                ui.text_with("Look for sources on retained versus immediate mode, text rendering cost and motion.", &Text { size: 13.5, color: Color::TEXT_SECONDARY, ..Default::default() });
            });
            ui.tool_call(
                "search",
                "web_search",
                &ToolCall {
                    icon: Some(cui::ICON_GLOBE),
                    detail: Some("immediate mode gui performance"),
                    status: TaskStatus::Done,
                    seconds: 1.4,
                    input: Some("{\"query\": \"immediate mode gui performance\", \"limit\": 5}"),
                    output: Some("{\"results\": [{\"title\": \"Immediate mode GUIs\", \"url\": \"https://fieldnotes.dev/immediate-mode\"}], \"took_ms\": 1380}"),
                },
            );
            ui.tool_call(
                "read",
                "read_file",
                &ToolCall {
                    icon: Some(cui::ICON_FILE_TEXT),
                    detail: Some("quarterly-report.pdf"),
                    status: if m.busy { TaskStatus::Running } else { TaskStatus::Done },
                    seconds: if m.busy { elapsed } else { 2.1 },
                    ..Default::default()
                },
            );
            if m.busy {
                ui.agent_status("Reading quarterly-report.pdf", &AgentStatus { status: TaskStatus::Running, seconds: elapsed, ..Default::default() });
            }
            ui.typewriter("answer", "They do very little work per frame: the interface is rebuilt from your data, laid out once and drawn in a handful of batches, so there is no tree of objects to keep in sync. Text is the expensive part, which is why glyphs are cached in an atlas, and motion feels quick when it is driven by springs instead of fixed curves.", &TextFx { size: 15., ..Default::default() });
            ui.row(&Row { gap: 6., align: Align::Center, ..Default::default() }, |ui| {
                ui.text_with("Sources", &Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
                for k in 0..3 {
                    ui.citation(k + 1, &Source {
                        title: Some(match k {
                            0 => "Immediate mode GUIs and why they keep coming back",
                            1 => "Notes on subpixel text positioning",
                            _ => "Spring physics for interface motion",
                        }),
                        url: match k {
                            0 => "https://fieldnotes.dev/immediate-mode",
                            1 => "https://glyphlab.io/notes/subpixel",
                            _ => "https://motionbook.dev/springs",
                        },
                        snippet: Some(match k {
                            0 => "A look at retained and immediate mode toolkits and the trade offs between them.",
                            1 => "Shaping, hinting and subpixel positioning explained with examples.",
                            _ => "Why spring physics feel more natural than cubic beziers for interface motion.",
                        }),
                    });
                }
            });
            let sources = [
                Source {
                    title: Some("Immediate mode GUIs and why they keep coming back"),
                    url: "https://fieldnotes.dev/immediate-mode",
                    snippet: Some("A look at retained and immediate mode toolkits and the trade offs between them."),
                },
                Source {
                    title: Some("Notes on subpixel text positioning"),
                    url: "https://glyphlab.io/notes/subpixel",
                    snippet: Some("Shaping, hinting and subpixel positioning explained with examples."),
                },
                Source {
                    title: Some("Spring physics for interface motion"),
                    url: "https://motionbook.dev/springs",
                    snippet: Some("Why spring physics feel more natural than cubic beziers for interface motion."),
                },
            ];
            ui.source_list("sources", &sources);
            ui.row(&Row { gap: 8., align: Align::Center, width: Size::FILL, ..Default::default() }, |ui| {
                let mut ra = cui::ResponseActions { copy_text: Some("They do very little work per frame."), vote: Some(&mut m.vote), ..Default::default() };
                let flags = ui.response_actions("actions", &mut ra);
                if flags.regenerated() {
                    m.busy = true;
                }
                ui.spacer();
                let parts = [
                    cui::VizItem { label: "System", value: 3200., color: cui::Color::TRANSPARENT },
                    cui::VizItem { label: "Files", value: 41800., color: cui::Color::TRANSPARENT },
                    cui::VizItem { label: "Messages", value: 68400., color: cui::Color::TRANSPARENT },
                ];
                ui.context_meter("ctx", 113400., 200000., &cui::ContextMeter { parts: &parts, ..Default::default() });
            });
            let models = [
                Model { name: "Atlas", detail: Some("Longer answers, reads files and code"), badge: Some("New"), icon: Some(cui::ICON_LAYERS) },
                Model { name: "Atlas Fast", detail: Some("Short answers in about a second"), badge: None, icon: Some(cui::ICON_ZAP) },
                Model { name: "Atlas Mini", detail: Some("Runs on your own machine"), badge: None, icon: Some(cui::ICON_BOX) },
            ];
            let files: Vec<&str> = m.files.iter().take(m.file_count as usize).copied().collect();
            let mut composer = Composer {
                placeholder: Some("Ask a follow up"),
                busy: m.busy,
                attachments: &[],
                removed: Some(&mut m.removed),
                models: &models,
                model: Some(&mut m.model),
                ..Default::default()
            };
            composer.attachments = Box::leak(files.clone().into_boxed_slice());
            let ev = ui.chat_composer("composer", &mut m.draft, &mut composer);
            if ev.sent() {
                m.draft.clear();
                m.busy = true;
            }
            if ev.stopped() {
                m.busy = false;
            }
            if ev.removed() && m.removed >= 0 && m.removed < m.file_count {
                let r = m.removed as usize;
                m.files.copy_within(r + 1.., r);
                m.file_count -= 1;
            }
            if ev.attach() {
                ui.toast_with("Attach files", &Toast { description: Some("Drop files on the window, too."), ..Default::default() });
            }
        });
        ui.column(&Column { width: Size::grow(1.), gap: 14., ..Default::default() }, |ui| {
            section(ui, "Agent plan", Some("Steps and their status."));
            let steps = [
                PlanStep { title: "Search for sources", detail: Some("3 results"), status: TaskStatus::Done },
                PlanStep {
                    title: "Read the attached report",
                    detail: Some(if m.busy { "Page 12 of 40" } else { "40 pages" }),
                    status: if m.busy { TaskStatus::Running } else { TaskStatus::Done },
                },
                PlanStep { title: "Compare the numbers with last quarter", detail: None, status: if m.busy { TaskStatus::Pending } else { TaskStatus::Done } },
                PlanStep { title: "Draft the summary", detail: None, status: if m.busy { TaskStatus::Pending } else { TaskStatus::Running } },
                PlanStep { title: "Check the citations", detail: None, status: TaskStatus::Pending },
            ];
            ui.agent_plan("plan", &steps, &AgentPlan { title: Some("Research plan") });
            section(ui, "Voice", Some("Idle, listening, thinking and speaking."));
            ui.column(&Column { align: Align::Center, gap: 12., width: Size::FILL, ..Default::default() }, |ui| {
                let lvl = 0.35 + 0.35 * (now as f32 * 5.1).sin() * (now as f32 * 1.7).sin();
                if ui.voice_orb("orb", m.orb_state, lvl, &cui_sys::cui_voice_orb_opts { size: 170., ..Default::default() }) {
                    m.orb_state = if m.orb_state == 1 { 0 } else { 1 };
                }
                let states = ["Idle", "Listening", "Thinking", "Speaking"];
                ui.segmented_with("orb-state", &states, &mut m.orb_state, &cui::Segmented { size: ControlSize::Sm, ..Default::default() });
            });
            section(ui, "Suggestions", Some("Ways to start."));
            let sugg = [
                PromptSuggestion { icon: Some(cui::ICON_CODE), title: "Explain this code", detail: Some("Walk through what it does") },
                PromptSuggestion { icon: Some(cui::ICON_CHART_BAR), title: "Analyze a dataset", detail: Some("Find trends and outliers") },
                PromptSuggestion { icon: Some(cui::ICON_PENCIL), title: "Draft an email", detail: Some("Clear and friendly") },
                PromptSuggestion { icon: Some(cui::ICON_BUG), title: "Debug an error", detail: Some("Paste the stack trace") },
            ];
            let sg = ui.prompt_suggestions("sugg", &sugg, &PromptSuggestions { columns: 1 });
            if sg >= 0 {
                m.draft.clear();
                m.draft.push_str(sugg[sg as usize].title);
                m.draft.push_str(": ");
            }
        });
    });
}

// ---- audio -----------------------------------------------------------------

fn page_audio(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    m.tick += 1;
    let names = ["Kick", "Snare", "Bass", "Keys", "Vocals", "FX"];
    let marks = [6., 0., -6., -12., -24., -48.];
    let mut lv = [[0f32; 2]; 6];
    for (k, ch) in lv.iter_mut().enumerate() {
        for (c, level) in ch.iter_mut().enumerate() {
            let t = (m.tick as f32 + c as f32 * 5.) / 60.;
            let beat = (t * (1.8 + 0.2 * k as f32) + k as f32 * 0.31) % 1.0;
            *level = (0.3 + 0.1 * k as f32) * (0.2 + 0.8 * (-beat * (2.5 + k as f32)).exp());
        }
    }
    if ui.button_with("Take the tour", &cui::Button { icon: Some(cui::ICON_SPARKLES), variant: cui::ButtonVariant::Outline, ..Default::default() }) {
        m.tour = 0;
    }
    ui.row(&Row { gap: 16., wrap: true, align: Align::Start, ..Default::default() }, |ui| {
        ui.card(&Card { gap: 16., min_width: 280., ..Default::default() }, |ui| {
            ui.heading("Synth", 4);
            ui.row(&Row { gap: 14., wrap: true, ..Default::default() }, |ui| {
                ui.knob("Cutoff", &mut m.cutoff, 20., 20000., &cui_sys::cui_knob_opts { unit: s(ui, "Hz"), log_scale: true, reset: 1000., ..Default::default() });
                m.tour_steps[0].target = ui.last_id();
                ui.knob("Resonance", &mut m.res, 0., 1., &cui_sys::cui_knob_opts { format: s(ui, "%.2f"), ..Default::default() });
                ui.knob("Attack", &mut m.attack, 1., 2000., &cui_sys::cui_knob_opts { unit: s(ui, "ms"), log_scale: true, reset: 10., ..Default::default() });
                ui.knob("Pan", &mut m.pan, -1., 1., &cui_sys::cui_knob_opts { bipolar: true, format: s(ui, "%+.2f"), ..Default::default() });
            });
            let master = [lv[0][0] * 1.4, lv[4][1] * 1.3];
            ui.level_meter("master", &master, 2, &cui_sys::cui_level_meter_opts { horizontal: true, length: 240., scale: true, segmented: true, ..Default::default() });
            m.tour_steps[1].target = ui.last_id();
        });
        ui.card(&Card { dir: Dir::Row, gap: 8., wrap: true, ..Default::default() }, |ui| {
            for (k, name) in names.iter().enumerate() {
                let _fader_id = ui.push_id_int(k as i32);
                let ticks = if k == 0 { marks.as_ptr() } else { core::ptr::null() };
                ui.fader(
                    name,
                    &mut m.faders[k],
                    -48.,
                    6.,
                    &cui_sys::cui_fader_opts { unit: s(ui, "dB"), length: 180., ticks, tick_count: if k == 0 { 6 } else { 0 }, meter: lv[k].as_ptr(), meter_channels: 2, ..Default::default() },
                );
                if k == 2 {
                    m.tour_steps[2].target = ui.last_id();
                }
            }
        });
    });
    if m.tour >= 0 {
        ui.tour("audio-tour", &m.tour_steps, &mut m.tour, &cui_sys::cui_tour_opts::default());
    }
}

// ---- files -----------------------------------------------------------------

/// The local disk, or a few made-up files where there is none.
extern "C" fn files_list(_user: *mut c_void, path: *const c_char, out: *mut cui_sys::cui_file_entry, cap: i32, names: *mut c_char, names_cap: usize, hidden: bool) -> i32 {
    let n = unsafe { cui_sys::cui_fs_list(_user, path, out, cap, names, names_cap, hidden) };
    if n >= 0 || unsafe { core::ffi::CStr::from_ptr(path) } != c"." {
        return n;
    }
    let demo = [("assets", true), ("src", true), ("README.md", false), ("notes.txt", false), ("cover.png", false), ("report.pdf", false)];
    let mut w = 0usize;
    unsafe {
        for (k, (name, dir)) in demo.iter().enumerate() {
            if k as i32 >= cap {
                break;
            }
            let e = &mut *out.add(k);
            let mut bytes = name.as_bytes().to_vec();
            bytes.push(0);
            if w + bytes.len() > names_cap {
                break;
            }
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), names.add(w).cast(), bytes.len());
            e.name = names.add(w) as *const c_char;
            e.dir = *dir;
            e.size = (k * 13100) as u64;
            e.modified = 1730000000 + k as i64 * 86400;
            w += bytes.len();
        }
    }
    6
}

fn term_write(term: &mut cui_sys::cui_terminal_buffer, text: &str) {
    if let Ok(c) = std::ffi::CString::new(text.replace('\0', "")) {
        unsafe { cui_sys::cui_terminal_write(term, c.as_ptr(), -1) };
    }
}

fn page_files(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    section(ui, "File browser", Some("Double-click to open; type to jump; Backspace goes up."));
    let o = cui_sys::cui_file_browser_opts { list: Some(files_list), height: Size::px(380.).to_ffi(), ..Default::default() };
    let r = ui.file_browser("files", &mut m.folder, &o);
    if !r.picked.is_null() {
        let picked = unsafe { core::ffi::CStr::from_ptr(r.picked) }.to_string_lossy().into_owned();
        m.chosen.clear();
        m.chosen.push_str(&picked);
    }
    ui.row(&Row { gap: 8., align: Align::Center, wrap: true, ..Default::default() }, |ui| {
        if ui.button_with("Open…", &cui::Button { icon: Some(cui::ICON_FOLDER_OPEN), ..Default::default() }) {
            m.open_dlg = true;
        }
        if ui.button_with("Save as…", &cui::Button { icon: Some(cui::ICON_DOWNLOAD), ..Default::default() }) {
            m.save_dlg = true;
        }
        if !m.chosen.is_empty() {
            ui.muted(m.chosen.clone());
        }
    });
    ui.file_picker("open", &mut m.open_dlg, &mut m.chosen, &cui_sys::cui_file_picker_opts { list: Some(files_list), ..Default::default() });
    ui.file_picker(
        "save",
        &mut m.save_dlg,
        &mut m.chosen,
        &cui_sys::cui_file_picker_opts { list: Some(files_list), save: true, name: s(ui, "untitled.txt"), ..Default::default() },
    );

    // a small shell over the same folder
    if m.term.line_cap == 0 {
        // SAFETY: the buffers live in `m` next to `m.term` for the whole
        // run and nothing else touches them.
        unsafe { ui.terminal_init(&mut m.term, &mut m.term_text, &mut m.term_lines) };
        term_write(&mut m.term, "Type \x1b[1mhelp\x1b[0m for the commands.\n");
    }
    section(ui, "Terminal", Some("Colors, \\r progress, history on Up and Down, Ctrl+C, Ctrl+L, drag to select."));
    let tr = ui.terminal("shell", &mut m.term, &cui_sys::cui_terminal_opts { height: 300., prompt: s(ui, "> "), ..Default::default() });
    if !tr.command.is_null() {
        let c = unsafe { core::ffi::CStr::from_ptr(tr.command) }.to_string_lossy().into_owned();
        if c == "help" {
            term_write(&mut m.term, "  \x1b[36mls\x1b[0m      the folder above\n  \x1b[36mcolors\x1b[0m  the 16 colors\n  \x1b[36mbuild\x1b[0m   a progress bar\n  \x1b[36mecho\x1b[0m    prints its words\n  \x1b[36mclear\x1b[0m   empties the screen\n");
        } else if c == "ls" {
            let mut ents = [cui_sys::cui_file_entry::default(); 256];
            let mut names = [0u8; 16384];
            let n = files_list(core::ptr::null_mut(), m.folder.as_ptr() as *const c_char, ents.as_mut_ptr(), 256, names.as_mut_ptr() as *mut c_char, names.len(), false);
            for e in ents.iter().take(n.max(0) as usize) {
                let name = unsafe { core::ffi::CStr::from_ptr(e.name) }.to_string_lossy();
                if e.dir {
                    term_write(&mut m.term, &format!("\x1b[1;34m{name}/\x1b[0m\n"));
                } else {
                    term_write(&mut m.term, &format!("{name}\n"));
                }
            }
            if n < 0 {
                term_write(&mut m.term, "\x1b[31mls: cannot read this folder\x1b[0m\n");
            }
        } else if c == "colors" {
            let mut line = String::new();
            for k in 0..16 {
                line.push_str(&format!("\x1b[{}m██\x1b[0m{}", if k < 8 { 30 + k } else { 82 + k }, if k == 7 || k == 15 { "\n" } else { " " }));
            }
            term_write(&mut m.term, &line);
        } else if c == "build" {
            for k in 0..=20 {
                let bar: String = (0..20).map(|j| if j < k { "█" } else { "░" }).collect();
                term_write(&mut m.term, &format!("\r\x1b[32m{bar}\x1b[0m {:3}%", k * 5));
            }
            term_write(&mut m.term, "\n\x1b[32mdone\x1b[0m\n");
        } else if let Some(rest) = c.strip_prefix("echo") {
            term_write(&mut m.term, &format!("{}\n", rest.trim_start()));
        } else if c == "clear" {
            unsafe { cui_sys::cui_terminal_clear(&mut m.term) };
        } else if !c.is_empty() {
            term_write(&mut m.term, &format!("\x1b[31m{c}: command not found\x1b[0m\n"));
        }
    }
    if tr.interrupt {
        ui.toast("Interrupted");
    }
}

// ---- developer tools -------------------------------------------------------

fn init_kv(pairs: &mut [cui_sys::cui_kv_pair; 12]) -> i32 {
    let mut kv: [cui_sys::cui_kv_pair; 12] = unsafe { core::mem::zeroed() };
    let set = |k: &mut cui_sys::cui_kv_pair, key: &str, value: &str, enabled: bool, secret: bool| {
        let kb = key.as_bytes();
        let vb = value.as_bytes();
        k.key[..kb.len()].copy_from_slice(unsafe { core::mem::transmute::<&[u8], &[i8]>(kb) });
        k.value[..vb.len()].copy_from_slice(unsafe { core::mem::transmute::<&[u8], &[i8]>(vb) });
        k.enabled = enabled;
        k.secret = secret;
    };
    set(&mut kv[0], "Authorization", "Bearer 4f9c2e81d0b7a6", true, true);
    set(&mut kv[1], "Accept", "application/json", true, false);
    set(&mut kv[2], "X-Request-Id", "7d1c0a", false, false);
    set(&mut kv[3], "Accept", "text/html", true, false);
    pairs.clone_from(&kv);
    4
}

fn devtools_kv(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    if m.kv_n < 0 {
        m.kv_n = init_kv(&mut m.kv_pairs);
    }
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 680., ..Default::default() }, |ui| {
        ui.text_with("Headers", &Text { font: cui::font::MEDIUM, ..Default::default() });
        ui.kv_editor("headers", &mut m.kv_pairs, &mut m.kv_n, &cui_sys::cui_kv_editor_opts::default());
    });
}

fn devtools_problems(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    let diags = [
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_ERROR, message: str_ptr("use of undeclared identifier 'frame_time'"), file: str_ptr("src/render/timing.c"), line: 42, column: 17, source: str_ptr("clang"), code: str_ptr("undeclared-var") },
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_WARNING, message: str_ptr("unused variable 'scale'"), file: str_ptr("src/render/timing.c"), line: 88, column: 11, source: str_ptr("clang"), code: str_ptr("-Wunused-variable") },
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_ERROR, message: str_ptr("incompatible pointer types passing 'float *' to 'double *'"), file: str_ptr("src/core/layout.c"), line: 210, column: 33, source: str_ptr("clang"), code: str_ptr("-Wincompatible-pointer-types") },
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_WARNING, message: str_ptr("implicit conversion loses floating-point precision"), file: str_ptr("src/core/layout.c"), line: 318, column: 22, source: str_ptr("clang"), code: str_ptr("-Wfloat-conversion") },
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_INFO, message: str_ptr("function 'measure' could be declared static"), file: str_ptr("src/core/layout.c"), line: 402, column: 6, source: str_ptr("clang-tidy"), code: str_ptr("misc-use-internal-linkage") },
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_HINT, message: str_ptr("loop variable copied on each iteration; consider a pointer"), file: str_ptr("src/app/main.c"), line: 51, column: 9, source: str_ptr("clang-tidy"), code: str_ptr("performance-for-range-copy") },
        cui_sys::cui_diagnostic { severity: cui_sys::CUI_SEVERITY_WARNING, message: str_ptr("comparison of integers of different signs"), file: str_ptr("src/app/main.c"), line: 77, column: 18, source: str_ptr("gcc"), code: str_ptr("-Wsign-compare") },
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 760., ..Default::default() }, |ui| {
        ui.problems("problems", &diags, &cui_sys::cui_problems_opts { height: 300., ..Default::default() });
    });
}

fn devtools_pipeline(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let stages = ["Build", "Test", "Package", "Deploy"];
    let jobs = [
        pipeline_job("linux-x64", 0, cui_sys::CUI_JOB_PASSED, 142., false),
        pipeline_job("macos-arm64", 0, cui_sys::CUI_JOB_PASSED, 201., false),
        pipeline_job("windows-x64", 0, cui_sys::CUI_JOB_PASSED, 236., false),
        pipeline_job("unit", 1, cui_sys::CUI_JOB_PASSED, 64., false),
        pipeline_job("integration", 1, cui_sys::CUI_JOB_RUNNING, 95., false),
        pipeline_job("lint", 1, cui_sys::CUI_JOB_FAILED, 12., true),
        pipeline_job("docker image", 2, cui_sys::CUI_JOB_PENDING, -1., false),
        pipeline_job("release notes", 2, cui_sys::CUI_JOB_SKIPPED, -1., false),
        pipeline_job("staging", 3, cui_sys::CUI_JOB_PENDING, -1., false),
        pipeline_job("production", 3, cui_sys::CUI_JOB_PENDING, -1., false),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 920., ..Default::default() }, |ui| {
        ui.pipeline("run", &stages, &jobs, &cui_sys::cui_pipeline_opts { selected: &mut m.pl_sel, ..Default::default() });
    });
}

fn pipeline_job(name: &'static str, stage: i32, status: i32, duration: f32, allow_failure: bool) -> cui_sys::cui_pipeline_job {
    cui_sys::cui_pipeline_job { name: str_ptr(name), stage, status, duration, allow_failure }
}

fn devtools_query(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 720., ..Default::default() }, |ui| {
        ui.query_builder("filter", &m.qb_fields, &mut m.qb_nodes, &mut m.qb_n, &cui_sys::cui_query_opts::default());
        let mut text = String::with_capacity(256);
        ui.query_text(&m.qb_fields, &m.qb_nodes, &mut text);
        ui.text_with(text, &Text { size: 12., color: Color::TEXT_MUTED, ..Default::default() });
    });
}

fn devtools_schedule(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 560., ..Default::default() }, |ui| {
        ui.schedule_picker("backup", &mut m.sc_cron, &cui_sys::cui_schedule_opts { now: 1794211200, utc: true, next_count: 3, ..Default::default() });
    });
}

fn devtools_keymap(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let binds: Vec<cui_sys::cui_keymap_binding> = [
        ((Mods::PRIMARY, Key::S), "Save", 0),
        ((Mods::PRIMARY, Key::O), "Open", 0),
        ((Mods::PRIMARY, Key::N), "New file", 0),
        ((Mods::PRIMARY, Key::W), "Close tab", 0),
        ((Mods::PRIMARY, Key::P), "Go to file", 1),
        ((Mods::PRIMARY, Key::G), "Go to line", 1),
        ((Mods::PRIMARY, Key::F), "Find", 2),
        ((Mods::PRIMARY, Key::H), "Replace", 2),
        ((Mods::PRIMARY, Key::Z), "Undo", 2),
        ((Mods::PRIMARY, Key::Y), "Redo", 2),
        ((Mods::PRIMARY, Key::B), "Build", 3),
        ((Mods::PRIMARY, Key::R), "Run", 3),
        ((Mods::PRIMARY, Key::D), "Debug", 3),
        ((Mods::PRIMARY, Key::Slash), "Comment", 2),
        ((Mods::PRIMARY, Key::E), "Recent files", 1),
        ((Mods::PRIMARY, Key::R), "Rename symbol", 2),
        ((Mods(0), Key::F5), "Continue", 3),
        ((Mods(0), Key::F9), "Breakpoint", 3),
        ((Mods(0), Key::F10), "Step over", 3),
        ((Mods(0), Key::F11), "Step into", 3),
        ((Mods(0), Key::F2), "Rename", 2),
        ((Mods(0), Key::F12), "Definition", 1),
        ((Mods::PRIMARY | Mods::SHIFT, Key::P), "Commands", 1),
        ((Mods::PRIMARY | Mods::SHIFT, Key::F), "Find in files", 2),
    ]
    .iter()
    .map(|((mods, key), action, category)| cui_sys::cui_keymap_binding {
        combo: cui_sys::cui_key_combo { mods: mods.0, key: key.to_ffi() as i32 },
        action: str_ptr(action),
        category: *category,
    })
    .collect();
    let cats = ["File", "Navigate", "Edit", "Run and debug"];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 900., ..Default::default() }, |ui| {
        let o = cui_sys::cui_keymap_opts {
            categories: ui.arena().cstr_array(&cats) as *mut _,
            category_count: cats.len() as i32,
            mods: &mut m.km_mods,
            ..Default::default()
        };
        ui.keymap("keys", &binds, &o);
    });
}

fn commit(hash: &'static str, message: &'static str, author: &'static str, date: &'static str, refs: Option<&'static str>, parents: [i32; 2]) -> cui_sys::cui_commit {
    cui_sys::cui_commit {
        hash: str_ptr(hash),
        message: str_ptr(message),
        author: str_ptr(author),
        date: str_ptr(date),
        refs: match refs {
            Some(r) => str_ptr(r),
            None => core::ptr::null(),
        },
        parent: parents,
    }
}

fn devtools_commits(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let commits = [
        commit("a91f3c2e", "Merge branch 'feature/trace-viewer'", "Maya Okafor", "2 hours ago", Some("main, origin/main"), [1, 4]),
        commit("7be04d19", "Docs: the trace viewer and the waterfall", "Maya Okafor", "3 hours ago", None, [2, -1]),
        commit("c3d8a7f0", "Fix the ruler's last tick on narrow panels", "Jonas Berg", "5 hours ago", None, [3, -1]),
        commit("1f22b9e4", "Release 1.2", "Jonas Berg", "yesterday", Some("tag: v1.2.0"), [6, -1]),
        commit("e8a0c55b", "Trace viewer: measure a range with Shift+drag", "Priya Nair", "yesterday", Some("feature/trace-viewer"), [5, -1]),
        commit("4d9e1a77", "Trace viewer: lanes, spans and markers", "Priya Nair", "2 days ago", None, [6, -1]),
        commit("90b7f3c1", "Merge pull request #21 from fix/atlas", "Maya Okafor", "3 days ago", None, [7, 8]),
        commit("52ce8d0a", "Faster soft shadows in software", "Lena Fischer", "3 days ago", None, [9, -1]),
        commit("b6f1e2d3", "Glyph atlas: compact after a zoom", "Aiko Tanaka", "4 days ago", Some("fix/atlas"), [9, -1]),
        commit("0aa4c9e8", "Right to left layout", "Omid Saadat", "5 days ago", None, [-1, -1]),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 920., ..Default::default() }, |ui| {
        ui.commit_graph("history", &commits, &cui_sys::cui_commit_graph_opts { selected: &mut m.cg_sel, height: 320., ..Default::default() });
    });
}

fn devtools_graph(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let nodes = [
        cui_sys::cui_graph_node { label: str_ptr("app"), detail: str_ptr("executable"), tone: Tone::Violet as i32 },
        cui_sys::cui_graph_node { label: str_ptr("ui"), detail: str_ptr("libcui.a"), tone: Tone::Info as i32 },
        cui_sys::cui_graph_node { label: str_ptr("net"), detail: str_ptr("http client"), tone: Tone::Info as i32 },
        cui_sys::cui_graph_node { label: str_ptr("storage"), detail: str_ptr("sqlite"), tone: Tone::Teal as i32 },
        cui_sys::cui_graph_node { label: str_ptr("render"), detail: str_ptr("software, OpenGL"), tone: Tone::Info as i32 },
        cui_sys::cui_graph_node { label: str_ptr("text"), detail: str_ptr("fonts, shaping"), tone: Tone::Info as i32 },
        cui_sys::cui_graph_node { label: str_ptr("tls"), detail: str_ptr("mbedtls"), tone: Tone::Warning as i32 },
        cui_sys::cui_graph_node { label: str_ptr("zlib"), detail: core::ptr::null(), tone: Tone::Neutral as i32 },
        cui_sys::cui_graph_node { label: str_ptr("platform"), detail: str_ptr("X11, Win32, Cocoa"), tone: Tone::Info as i32 },
    ];
    let edges = [
        cui_sys::cui_graph_edge { from: 0, to: 1 },
        cui_sys::cui_graph_edge { from: 0, to: 2 },
        cui_sys::cui_graph_edge { from: 0, to: 3 },
        cui_sys::cui_graph_edge { from: 1, to: 4 },
        cui_sys::cui_graph_edge { from: 1, to: 5 },
        cui_sys::cui_graph_edge { from: 1, to: 8 },
        cui_sys::cui_graph_edge { from: 2, to: 6 },
        cui_sys::cui_graph_edge { from: 2, to: 7 },
        cui_sys::cui_graph_edge { from: 3, to: 7 },
        cui_sys::cui_graph_edge { from: 5, to: 7 },
        cui_sys::cui_graph_edge { from: 4, to: 8 },
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 1000., ..Default::default() }, |ui| {
        ui.graph_view("deps", &nodes, &edges, &cui_sys::cui_graph_opts { selected: &mut m.gv_sel, height: 340., ..Default::default() });
    });
}

fn devtools_waterfall(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let reqs = [
        net_req("https://shop.example.com/", "GET", 200, "document", 18400., 0., [0.002, 0.018, 0.024, 0.031, 0.142, 0.012]),
        net_req("https://shop.example.com/assets/app.css", "GET", 200, "stylesheet", 42100., 0.232, [0.004, 0., 0., 0., 0.061, 0.022]),
        net_req("https://shop.example.com/assets/app.js", "GET", 200, "script", 186000., 0.233, [0.005, 0., 0., 0., 0.068, 0.094]),
        net_req("https://cdn.example.net/fonts/inter.woff2", "GET", 200, "font", 48200., 0.241, [0.003, 0.021, 0.019, 0.028, 0.044, 0.018]),
        net_req("https://shop.example.com/api/session", "POST", 401, "fetch", 312., 0.402, [0.002, 0., 0., 0., 0.083, 0.001]),
        net_req("https://shop.example.com/api/products?page=1", "GET", 200, "fetch", 9600., 0.404, [0.002, 0., 0., 0., 0.121, 0.006]),
        net_req("https://shop.example.com/img/hero.avif", "GET", 200, "image", 212000., 0.410, [0.012, 0., 0., 0., 0.052, 0.188]),
        net_req("https://shop.example.com/img/p/1042.webp", "GET", 304, "image", 0., 0.538, [0.006, 0., 0., 0., 0.038, 0.]),
        net_req("https://shop.example.com/img/p/1043.webp", "GET", 200, "image", 31200., 0.540, [0.008, 0., 0., 0., 0.041, 0.024]),
        net_req("https://analytics.example.org/collect", "POST", 0, "ping", 0., 0.702, [0.004, 0.026, 0.031, 0.036, 0.12, 0.]),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 1000., ..Default::default() }, |ui| {
        ui.waterfall("net", &reqs, &cui_sys::cui_waterfall_opts { selected: &mut m.wf_sel, height: 340., dom_loaded: 0.395, loaded: 0.68, ..Default::default() });
    });
}

fn net_req(name: &'static str, method: &'static str, status: i32, ty: &'static str, size: f64, start: f64, timings: [f64; 6]) -> cui_sys::cui_net_request {
    let mut timing = [0f32; 6];
    for (t, v) in timing.iter_mut().zip(timings) {
        *t = v as f32;
    }
    cui_sys::cui_net_request { name: str_ptr(name), method: str_ptr(method), status, type_: str_ptr(ty), size, start, timing }
}

fn trace_track(name: &'static str) -> cui_sys::cui_trace_track {
    cui_sys::cui_trace_track { name: str_ptr(name), collapsed: false }
}

fn trace_span(name: &'static str, start: f64, end: f64, track: i32, depth: i32) -> cui_sys::cui_trace_span {
    cui_sys::cui_trace_span { name: str_ptr(name), start, end, track, depth, color_key: 0 }
}

fn build_trace(spans: &mut Vec<cui_sys::cui_trace_span>) {
    for f in 0..3 {
        let t = f as f64 * 0.0166;
        let mut add = |n: &'static str, s: f64, e: f64, tr: i32, d: i32| spans.push(trace_span(n, s, e, tr, d));
        add("Frame", t + 0.0004, t + 0.0151, 0, 0);
        add("Input", t + 0.0006, t + 0.0019, 0, 1);
        add("Update", t + 0.0021, t + 0.0092, 0, 1);
        add("Physics", t + 0.0023, t + 0.0051, 0, 2);
        add("Animation", t + 0.0053, t + 0.0068, 0, 2);
        add("Scripts", t + 0.0070, t + 0.0090, 0, 2);
        add("Layout", t + 0.0094, t + 0.0121, 0, 1);
        add("Measure", t + 0.0095, t + 0.0108, 0, 2);
        add("Paint", t + 0.0123, t + 0.0149, 0, 1);
        add("Record", t + 0.0124, t + 0.0139, 0, 2);
        add("Submit", t + 0.0152, t + 0.0163, 1, 0);
        add("Upload", t + 0.0153, t + 0.0157, 1, 1);
        add("Encode", t + 0.0157, t + 0.0162, 1, 1);
        add("Decode image", t + 0.0012, t + 0.0074, 2, 0);
        add("Inflate", t + 0.0013, t + 0.0049, 2, 1);
        add("Parse JSON", t + 0.0081, t + 0.0118, 3, 0);
        add("Draw", t + 0.0164, t + 0.0166 + 0.0105, 4, 0);
    }
    unsafe { cui_sys::cui_trace_sort(spans.as_mut_ptr(), spans.len() as i32) };
}

fn devtools_trace(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    if m.tr_spans.is_empty() {
        m.tr_tracks = vec![
            trace_track("Main thread"),
            trace_track("Render"),
            trace_track("Worker 1"),
            trace_track("Worker 2"),
            trace_track("GPU"),
        ];
        build_trace(&mut m.tr_spans);
    }
    if m.tr_sel < 0 {
        for (i, span) in m.tr_spans.iter().enumerate() {
            let name = unsafe { core::ffi::CStr::from_ptr(span.name) }.to_bytes() == b"Update";
            if name && span.start > 0.016 && span.start < 0.03 {
                m.tr_sel = i as i32;
                break;
            }
        }
    }

    let marks = [
        cui_sys::cui_trace_marker { name: str_ptr("vsync"), time: 0.0, tone: Tone::Neutral as i32 },
        cui_sys::cui_trace_marker { name: str_ptr("vsync"), time: 0.0166, tone: Tone::Neutral as i32 },
        cui_sys::cui_trace_marker { name: str_ptr("GC pause"), time: 0.0273, tone: Tone::Warning as i32 },
        cui_sys::cui_trace_marker { name: str_ptr("vsync"), time: 0.0332, tone: Tone::Neutral as i32 },
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 1000., ..Default::default() }, |ui| {
        let o = cui_sys::cui_trace_opts { markers: marks.as_ptr(), marker_count: marks.len() as i32, height: 320., ..Default::default() };
        ui.trace_view("trace", &mut m.tr_tracks, &m.tr_spans, &mut m.tr_view, &mut m.tr_sel, &o);
    });
}

fn page_devtools(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Trace viewer", Some("Wheel zooms, drag pans, Shift+drag measures, a lane's name folds it."));
    devtools_trace(ui, g);
    section(ui, "Network waterfall", Some("Hover a bar for its timing; Up and Down move the selection."));
    devtools_waterfall(ui, g);
    section(ui, "Dependency graph", Some("Hover a node to light up its neighbours; wheel zooms, F fits."));
    devtools_graph(ui, g);
    section(ui, "Commit graph", Some("Branch lanes, merges and refs; only the rows in view are built."));
    devtools_commits(ui, g);
    section(ui, "Pipeline", Some("Stages of jobs; a stage sums up its jobs."));
    devtools_pipeline(ui, g);
    section(ui, "Problems", Some("Grouped by file; filter by severity or text, Enter opens."));
    devtools_problems(ui, g);
    section(ui, "Key-value editor", Some("Type in the empty row to add a pair; the lock hides a value."));
    devtools_kv(ui, g);
    section(ui, "Query builder", Some("Nested all and any groups."));
    devtools_query(ui, g);
    section(ui, "Schedule picker", Some("Writes a cron expression and shows the next runs."));
    devtools_schedule(ui, g);
    section(ui, "Keymap", Some("Shortcuts on a keyboard; keys bound twice are marked."));
    devtools_keymap(ui, g);
}

// ---- more developer tools --------------------------------------------------

fn devtools2_tests(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    let items = [
        test_item("layout", 0, -1, 0., None),
        test_item("row distributes extra space", 1, cui_sys::CUI_JOB_PASSED, 0.004, None),
        test_item("wraps to the next line", 1, cui_sys::CUI_JOB_PASSED, 0.006, None),
        test_item("mirrors under right to left", 1, cui_sys::CUI_JOB_FAILED, 0.012, Some("expected x = 412, got 388 (padding not mirrored)")),
        test_item("text", 0, -1, 0., None),
        test_item("shapes Persian", 1, cui_sys::CUI_JOB_PASSED, 0.210, None),
        test_item("ellipsis keeps whole characters", 1, cui_sys::CUI_JOB_PASSED, 0.003, None),
        test_item("bidi brackets", 1, cui_sys::CUI_JOB_SKIPPED, -1., None),
        test_item("renderer", 0, -1, 0., None),
        test_item("software", 1, -1, 0., None),
        test_item("rounded corners", 2, cui_sys::CUI_JOB_PASSED, 0.031, None),
        test_item("soft shadows", 2, cui_sys::CUI_JOB_RUNNING, -1., None),
        test_item("OpenGL", 1, -1, 0., None),
        test_item("atlas growth", 2, cui_sys::CUI_JOB_PENDING, -1., None),
        test_item("slow gradient path", 2, cui_sys::CUI_JOB_PASSED, 1.42, None),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 760., ..Default::default() }, |ui| {
        ui.test_explorer("tests", &items, &cui_sys::cui_test_explorer_opts { height: 500., ..Default::default() });
    });
}

fn test_item(name: &'static str, depth: i32, status: i32, duration: f32, message: Option<&'static str>) -> cui_sys::cui_test_item {
    cui_sys::cui_test_item {
        name: str_ptr(name),
        depth,
        status,
        duration,
        message: match message { Some(m) => str_ptr(m), None => core::ptr::null() },
    }
}

fn devtools2_stack(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    let src1: Vec<&str> = vec![
        "function total(cart) {",
        "  let sum = 0;",
        "  for (const item of cart.items) {",
        "    sum += item.price * item.qty;",
        "  }",
        "  return sum;",
    ];
    let src2: Vec<&str> = vec![
        "export function checkout(user) {",
        "  const cart = carts.get(user.id);",
        "  const amount = total(cart);",
        "  return charge(user, amount);",
    ];
    let frames = [
        stack_frame("total", "src/cart.js", 3, 29, false, Some(&src1), 1),
        stack_frame("checkout", "src/checkout.js", 3, 18, false, Some(&src2), 1),
        stack_frame("handle", "node_modules/express/lib/router/layer.js", 95, 5, true, None, 0),
        stack_frame("next", "node_modules/express/lib/router/route.js", 149, 13, true, None, 0),
        stack_frame("dispatch", "node_modules/express/lib/router/index.js", 284, 15, true, None, 0),
        stack_frame("onRequest", "src/server.js", 41, 9, false, None, 0),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 760., ..Default::default() }, |ui| {
        ui.stack_trace(
            "trace",
            &frames,
            &cui_sys::cui_stack_trace_opts { title: s(ui, "TypeError"), message: s(ui, "Cannot read properties of undefined (reading 'items')"), lang: s(ui, "js"), ..Default::default() },
        );
    });
}

fn stack_frame(function: &'static str, file: &'static str, line: i32, column: i32, external: bool, source: Option<&Vec<&'static str>>, source_first: i32) -> cui_sys::cui_stack_frame {
    let mut f = cui_sys::cui_stack_frame {
        function: str_ptr(function),
        file: str_ptr(file),
        line,
        column,
        external,
        source: core::ptr::null(),
        source_count: 0,
        source_first: 0,
    };
    if let Some(lines) = source {
        // leak the source lines; they live as long as the process
        let leaked: &'static mut [*const c_char] = Box::leak(lines.iter().map(|l| str_ptr(l) as *const c_char).collect::<Vec<_>>().into_boxed_slice());
        f.source = leaked.as_ptr();
        f.source_count = leaked.len() as i32;
        f.source_first = source_first;
    }
    f
}

fn devtools2_blame(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    let lines = [
        "#include \"cui_internal.h\"", "", "/* the width a component asks for */", "static float measure(cui_node *n) {", "    float w = n->pad[1] + n->pad[3];",
        "    for (int c = n->first; c >= 0; c = next(c))", "        w += width_of(c) + n->gap;", "    return w - n->gap;", "}", "",
        "void cui__layout(cui_ctx *ui) {", "    for (int i = 0; i < ui->node_count; i++)", "        measure(&ui->nodes[i]);", "    place(ui, 0);  /* the root */", "}",
    ];
    let line_commit = [0, 0, 1, 1, 2, 2, 2, 1, 1, 0, 3, 3, 4, 3, 3];
    let commits = [
        cui_sys::cui_blame_commit { hash: str_ptr("a1b2c3d4e5"), author: str_ptr("Maya Okafor"), summary: str_ptr("First layout pass"), time: 1730000000 },
        cui_sys::cui_blame_commit { hash: str_ptr("b2c3d4e5f6"), author: str_ptr("Jonas Berg"), summary: str_ptr("Measure: count padding once"), time: 1760000000 },
        cui_sys::cui_blame_commit { hash: str_ptr("c3d4e5f6a7"), author: str_ptr("Priya Nair"), summary: str_ptr("Gaps between children"), time: 1790000000 },
        cui_sys::cui_blame_commit { hash: str_ptr("d4e5f6a7b8"), author: str_ptr("Omid Saadat"), summary: str_ptr("Layout entry point"), time: 1793000000 },
        cui_sys::cui_blame_commit { hash: str_ptr("e5f6a7b8c9"), author: str_ptr("Aiko Tanaka"), summary: str_ptr("Measure every node"), time: 1794100000 },
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 900., ..Default::default() }, |ui| {
        ui.blame("blame", &lines, &line_commit, &commits, &cui_sys::cui_blame_opts { lang: s(ui, "c"), now: 1794211200, height: 320., first_line: 120, ..Default::default() });
    });
}

fn devtools2_coverage(ui: &mut Ui, g: &mut Gallery) {
    let _ = g;
    let files = [
        cov_file("src/core/layout.c", 1840, 1702, 620, 541, 88, 86),
        cov_file("src/core/state.c", 420, 401, 96, 90, 31, 31),
        cov_file("src/core/draw.c", 960, 610, 300, 160, 54, 40),
        cov_file("src/text/shape.c", 2210, 1490, 802, 411, 120, 88),
        cov_file("src/text/atlas.c", 640, 598, 140, 131, 30, 30),
        cov_file("src/render/gl.c", 1300, 410, 260, 60, 70, 22),
        cov_file("src/render/software.c", 2400, 2210, 900, 799, 140, 136),
        cov_file("src/main.c", 120, 120, 10, 10, 4, 4),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 820., ..Default::default() }, |ui| {
        ui.coverage("cov", &files, &cui_sys::cui_coverage_opts { height: 372., ..Default::default() });
    });
}

fn cov_file(path: &'static str, lines: i32, lines_hit: i32, branches: i32, branches_hit: i32, functions: i32, functions_hit: i32) -> cui_sys::cui_coverage_file {
    cui_sys::cui_coverage_file { path: str_ptr(path), lines, lines_hit, branches, branches_hit, functions, functions_hit }
}

fn devtools2_regex(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 760., ..Default::default() }, |ui| {
        ui.regex_tester("rx", &mut m.rx_pat, &mut m.rx_text, &cui_sys::cui_regex_tester_opts { height: 110., ..Default::default() });
    });
}

fn devtools2_request(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    if m.rq_n < 0 {
        m.rq_n = unsafe { cui_sys::cui_url_params(m.rq_url.as_ptr() as *const c_char, m.rq_params.as_mut_ptr(), m.rq_params.len() as i32) };
    }
    ui.column(&Column { gap: 12., width: Size::FILL, max_width: 760., ..Default::default() }, |ui| {
        let r = ui.request_bar("req", &mut m.rq_method, &mut m.rq_url, &cui_sys::cui_request_bar_opts::default());
        if r.changed {
            m.rq_n = unsafe { cui_sys::cui_url_params(m.rq_url.as_ptr() as *const c_char, m.rq_params.as_mut_ptr(), m.rq_params.len() as i32) };
        }
        let mut n = m.rq_n;
        if ui.kv_editor("params", &mut m.rq_params, &mut n, &cui_sys::cui_kv_editor_opts::default()) {
            m.rq_n = n;
            ui.url_set_params(&mut m.rq_url, &m.rq_params[..n.max(0) as usize]);
        }
        ui.response_status(201, 0.142, 18400.);
    });
}

fn devtools2_uptime(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let names = ["API", "Website", "Database", "CDN"];
    let sv = [
        cui_sys::cui_uptime_service { name: s(ui, names[0]), days: m.up_api.as_ptr(), day_count: 90 },
        cui_sys::cui_uptime_service { name: s(ui, names[1]), days: m.up_web.as_ptr(), day_count: 90 },
        cui_sys::cui_uptime_service { name: s(ui, names[2]), days: m.up_db.as_ptr(), day_count: 90 },
        cui_sys::cui_uptime_service { name: s(ui, names[3]), days: m.up_cdn.as_ptr(), day_count: 90 },
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 860., ..Default::default() }, |ui| {
        ui.uptime("status", &sv, &cui_sys::cui_uptime_opts { last_day: cui_sys::cui_date { year: 2026, month: 11, day: 9 }, ..Default::default() });
    });
}

fn devtools2_histogram(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let se = [
        cui_sys::cui_histogram_series { name: s(ui, "Info"), counts: m.th_info.as_ptr(), tone: Tone::Info as i32 },
        cui_sys::cui_histogram_series { name: s(ui, "Warning"), counts: m.th_warn.as_ptr(), tone: Tone::Warning as i32 },
        cui_sys::cui_histogram_series { name: s(ui, "Error"), counts: m.th_err.as_ptr(), tone: Tone::Danger as i32 },
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 900., ..Default::default() }, |ui| {
        ui.time_histogram("logs", 1794240000., 900., 96, &se, &mut m.th_s0, &mut m.th_s1, &cui_sys::cui_time_histogram_opts { utc: true, height: 150., ..Default::default() });
    });
}

fn devtools2_api(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let p1 = [
        cui_sys::cui_api_param { name: str_ptr("id"), in_: str_ptr("path"), type_: str_ptr("string"), required: true, description: str_ptr("The user's id") },
        cui_sys::cui_api_param { name: str_ptr("fields"), in_: str_ptr("query"), type_: str_ptr("string"), required: false, description: str_ptr("Comma separated fields to return") },
    ];
    let p2 = [
        cui_sys::cui_api_param { name: str_ptr("limit"), in_: str_ptr("query"), type_: str_ptr("integer"), required: false, description: str_ptr("At most this many, default 20") },
        cui_sys::cui_api_param { name: str_ptr("cursor"), in_: str_ptr("query"), type_: str_ptr("string"), required: false, description: str_ptr("From a previous page") },
    ];
    let eps = [
        api_ep("GET", "/users", "List users", "Users", false, None, 0),
        api_ep("POST", "/users", "Create a user", "Users", false, None, 0),
        api_ep("GET", "/users/{id}", "Get a user", "Users", false, Some(&p1), 2),
        api_ep("PATCH", "/users/{id}", "Update a user", "Users", false, Some(&p1), 1),
        api_ep("DELETE", "/users/{id}", "Delete a user", "Users", false, Some(&p1), 1),
        api_ep("GET", "/orders", "List orders", "Orders", false, Some(&p2), 2),
        api_ep("PUT", "/orders/{id}/status", "Set an order's status", "Orders", false, None, 0),
        api_ep("GET", "/v1/orders", "List orders (old)", "Orders", true, None, 0),
    ];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 820., ..Default::default() }, |ui| {
        ui.api_explorer("api", &eps, &cui_sys::cui_api_explorer_opts { selected: &mut m.ax_sel, ..Default::default() });
    });
}

fn api_ep(method: &'static str, path: &'static str, summary: &'static str, tag: &'static str, deprecated: bool, params: Option<&[cui_sys::cui_api_param]>, param_count: i32) -> cui_sys::cui_api_endpoint {
    cui_sys::cui_api_endpoint {
        method: str_ptr(method),
        path: str_ptr(path),
        summary: str_ptr(summary),
        tag: str_ptr(tag),
        deprecated,
        params: match params {
            Some(p) => p.as_ptr(),
            None => core::ptr::null(),
        },
        param_count,
    }
}

fn page_devtools2(ui: &mut Ui, g: &mut Gallery) {
    section(ui, "Test explorer", Some("Suites fold; hover a row to run it; filter by status or name."));
    devtools2_tests(ui, g);
    section(ui, "Stack trace", Some("Library frames fold; a frame with source opens; copy the trace."));
    devtools2_stack(ui, g);
    section(ui, "Blame", Some("Hover the gutter to mark a commit's lines; click it to open the commit."));
    devtools2_blame(ui, g);
    section(ui, "Coverage", Some("Folders sum up their files; click a column to sort."));
    devtools2_coverage(ui, g);
    section(ui, "Regex tester", Some("Matches and groups are marked as you type."));
    devtools2_regex(ui, g);
    section(ui, "HTTP request bar", Some("The parameters table and the URL stay in step."));
    devtools2_request(ui, g);
    section(ui, "API explorer", Some("Click an endpoint for its parameters."));
    devtools2_api(ui, g);
    section(ui, "Feature flags", Some("A switch per environment; click a share to change it."));
    let envs = ["Development", "Staging", "Production"];
    ui.column(&Column { gap: 10., width: Size::FILL, max_width: 860., ..Default::default() }, |ui| {
        ui.feature_flags("flags", &mut g.more.ff_flags, &envs, &cui_sys::cui_feature_flags_opts::default());
    });
    section(ui, "Uptime", Some("One bar per day; hover for the day."));
    devtools2_uptime(ui, g);
    section(ui, "Time histogram", Some("Drag to select a range; click to clear it."));
    devtools2_histogram(ui, g);
}

// ---- instruments -----------------------------------------------------------

/// One dose of caffeine `t` hours after it, in mg: absorbed in about 45
/// minutes, then a half-life of five hours.
fn dose(t: f64) -> f32 {
    if t <= 0. {
        return 0.;
    }
    let ke = (2.0f64).ln() / 5.0;
    let ka = (2.0f64).ln() / 0.25;
    (95. * ka / (ka - ke) * ((-ke * t).exp() - (-ka * t).exp())) as f32
}

fn page_instruments(ui: &mut Ui, g: &mut Gallery) {
    let m = &mut g.more;
    let (mut peak, mut peak_at, mut bed) = (0f64, 6f64, 0f64);
    for k in 0..97 {
        let at = 6. + 24. * k as f64 / 96.;
        let mut total = 0f64;
        for c in 0..m.cup_count as usize {
            m.curve[c][k] = dose(at - m.cups[c]);
            total += m.curve[c][k] as f64;
        }
        if total > peak {
            peak = total;
            peak_at = at;
        }
    }
    for c in 0..m.cup_count as usize {
        bed += dose(23. - m.cups[c]) as f64;
    }
    let peak_text = format!("PEAK {peak:.0} MG");
    let peak_mg = format!("{peak:.0}");
    let bed_text = format!("{bed:.0} MG STILL IN YOU");

    section(ui, "Readouts", Some("A label, a value with its unit, and a line of context, in the monospaced font."));
    ui.row(&Row { gap: 40., wrap: true, align: Align::End, ..Default::default() }, |ui| {
        ui.readout_with(
            "AT BED",
            format!("{bed:.0}"),
            &Readout { unit: Some("MG"), size: ControlSize::Lg, accent: bed > 50., detail: Some(if bed > 50. { "OVER THE 50 MG LINE" } else { "UNDER THE 50 MG LINE" }), ..Default::default() },
        );
        ui.readout_with("PEAK", &peak_mg, &Readout { unit: Some("MG"), detail: Some("AT 14:12"), ..Default::default() });
        ui.readout_with("CUPS", if m.cup_count >= 5 { "5+".to_string() } else { m.cup_count.to_string() }, &Readout { size: ControlSize::Sm, detail: Some("95 MG EACH"), ..Default::default() });
    });
    section(ui, "Dot-matrix chart and event track", Some("Drag a cup along the axis, click the chart to add one, double-click a cup to remove it."));
    let series: Vec<DotSeries> = (0..m.cup_count as usize)
        .map(|c| DotSeries { values: &m.curve[c], name: None, accent: c == m.cup_selected as usize })
        .collect();
    let notes = [
        cui::Annotation { kind: cui::AnnotationKind::Y, x: 0., y: 50., label: Some("50"), detail: None, accent: false },
        cui::Annotation { kind: cui::AnnotationKind::X, x: 23., y: 0., label: Some("BED 23:00"), detail: Some(&bed_text), accent: false },
        cui::Annotation { kind: cui::AnnotationKind::X, x: peak_at, y: 0., label: Some(&peak_text), detail: None, accent: false },
    ];
    ui.column(&Column { width: Size::FILL, gap: 0., ..Default::default() }, |ui| {
        let r = ui.dot_chart(
            "caffeine",
            &series,
            97,
            &DotChart {
                height: 240.,
                x_min: 6.,
                x_max: 30.,
                x_units: Axis::Hours,
                min: 0.,
                max: 260.,
                format: Some("MG"),
                no_axis: true,
                annotations: if m.cup_count != 0 { &notes } else { &notes[..1] },
                ..Default::default()
            },
        );
        if r.clicked && m.cup_count < 8 {
            let t = (r.x * 4. + 0.5).floor() / 4.;
            let mut at = m.cup_count as usize;
            while at > 0 && m.cups[at - 1] > t {
                m.cups[at] = m.cups[at - 1];
                at -= 1;
            }
            m.cups[at] = t;
            m.cup_count += 1;
            m.cup_selected = at as i32;
        }
        let mut track = EventTrack {
            step: 0.25,
            units: Axis::Hours,
            label_step: 6.,
            selected: Some(&mut m.cup_selected),
            ..Default::default()
        };
        ui.event_track("cups", &mut m.cups, &mut m.cup_count, 6., 30., &mut track);
    });
    section(ui, "Dot-matrix display", Some("Text in a 5 by 7 dot font; a long line can scroll."));
    ui.dot_display_with(
        if bed > 50. { "OVER THE LINE AT BEDTIME" } else { "UNDER THE LINE AT BEDTIME" },
        &DotDisplay { pitch: 3., accent: bed > 50., ..Default::default() },
    );
}
