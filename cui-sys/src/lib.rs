//! Raw FFI bindings to CUI. The C sources are the amalgamated release
//! assets of the upstream version pinned in `UPSTREAM.toml`, committed in
//! `c/` and digest-verified at build time (see `build.rs` and `vendor.sh`).
//!
//! Only the `*_ex` functions and plain structs are declared here — the C
//! named-argument macros are sugar over these. For the safe, idiomatic API
//! use the `cui` crate instead; this crate mirrors `include/cui/cui.h` and
//! `include/cui/cui_app.h` one-to-one.
//!
//! C enums are mapped to `#[repr(u32)]` field-less enums (all values are
//! non-negative). Opts structs keep their C layout, including the leading `_pad` (C: `int _;`) — the leading
//! `int _;` padding field; a zeroed struct means "all defaults", exactly as
//! in C.
#![no_std]
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ffi::{c_char, c_int, c_void};

pub type cui_ctx = core::ffi::c_void;
pub type cui_app = core::ffi::c_void;
pub type cui_id = u32;
pub type cui_texture = usize;
pub type cui_font = c_int;

/// The upstream CUI version the compiled C code was generated from,
/// e.g. `"1.4.0"`. Extracted from the release-asset header at build time,
/// so it always describes the C code that was actually compiled. The
/// digest-verified pin lives in `UPSTREAM.toml`.
pub const CUI_VERSION_MAJOR: i32 = 1;
pub const CUI_VERSION_MINOR: i32 = 4;
pub const CUI_VERSION_STRING: &str = env!("CUI_VERSION_STRING");

/* ==== Basic types (cui.h) ==== */

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct cui_vec2 {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct cui_rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct cui_color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// cui_token — semantic color tokens (CUI_TOKEN_*).
pub type cui_token = c_int;
pub const CUI_TOKEN_NONE: cui_token = 0;
pub const CUI_TOKEN_BACKGROUND: cui_token = 1;
pub const CUI_TOKEN_SURFACE: cui_token = 2;
pub const CUI_TOKEN_SURFACE_RAISED: cui_token = 3;
pub const CUI_TOKEN_SURFACE_MUTED: cui_token = 4;
pub const CUI_TOKEN_FOREGROUND: cui_token = 5;
pub const CUI_TOKEN_TEXT_SECONDARY: cui_token = 6;
pub const CUI_TOKEN_TEXT_MUTED: cui_token = 7;
pub const CUI_TOKEN_BORDER: cui_token = 8;
pub const CUI_TOKEN_BORDER_SUBTLE: cui_token = 9;
pub const CUI_TOKEN_BORDER_STRONG: cui_token = 10;
pub const CUI_TOKEN_ACCENT: cui_token = 11;
pub const CUI_TOKEN_ACCENT_STRONG: cui_token = 12;
pub const CUI_TOKEN_ACCENT_SUBTLE: cui_token = 13;
pub const CUI_TOKEN_ACCENT_FOREGROUND: cui_token = 14;
pub const CUI_TOKEN_SUCCESS: cui_token = 15;
pub const CUI_TOKEN_WARNING: cui_token = 16;
pub const CUI_TOKEN_DANGER: cui_token = 17;
pub const CUI_TOKEN_INFO: cui_token = 18;
pub const CUI_TOKEN_CONTROL_ON: cui_token = 19;
pub const CUI_TOKEN_CONTROL_GLYPH: cui_token = 20;
pub const CUI_TOKEN_CONTROL_TRACK: cui_token = 21;
pub const CUI_TOKEN_CONTROL_TRACK_HOVER: cui_token = 22;
pub const CUI_TOKEN_CONTROL_THUMB: cui_token = 23;
pub const CUI_TOKEN_CONTROL_THUMB_ON: cui_token = 24;
pub const CUI_TOKEN_HOVER: cui_token = 25;
pub const CUI_TOKEN_ACTIVE: cui_token = 26;
pub const CUI_TOKEN_SCRIM: cui_token = 27;
pub const CUI_TOKEN_FOCUS_RING: cui_token = 28;
pub const CUI_TOKEN_SERIES_1: cui_token = 29;
pub const CUI_TOKEN_SERIES_2: cui_token = 30;
pub const CUI_TOKEN_SERIES_3: cui_token = 31;
pub const CUI_TOKEN_SERIES_4: cui_token = 32;
pub const CUI_TOKEN_SERIES_5: cui_token = 33;
pub const CUI_TOKEN_SERIES_6: cui_token = 34;
pub const CUI_TOKEN_COUNT: cui_token = 35;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_size_kind {
    #[default]
    CUI_SIZE_FIT = 0,
    CUI_SIZE_GROW,
    CUI_SIZE_FIXED,
    CUI_SIZE_PERCENT,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct cui_size {
    pub kind: cui_size_kind,
    pub value: f32,
}

/// cui_dir — container direction.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_dir {
    #[default]
    CUI_COLUMN = 0,
    CUI_ROW,
    CUI_STACK,
    CUI_GRID,
}

/// cui_align — cross axis alignment.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_align {
    #[default]
    CUI_ALIGN_DEFAULT = 0,
    CUI_ALIGN_START,
    CUI_ALIGN_CENTER,
    CUI_ALIGN_END,
    CUI_ALIGN_STRETCH,
}

/// cui_justify — main axis distribution.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_justify {
    #[default]
    CUI_JUSTIFY_START = 0,
    CUI_JUSTIFY_CENTER,
    CUI_JUSTIFY_END,
    CUI_JUSTIFY_BETWEEN,
    CUI_JUSTIFY_AROUND,
}

/// cui_elevation.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_elevation {
    #[default]
    CUI_ELEVATION_NONE = 0,
    CUI_ELEVATION_RESTING,
    CUI_ELEVATION_RAISED,
    CUI_ELEVATION_FLOATING,
}

/// cui_direction.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_direction {
    #[default]
    CUI_DIRECTION_INHERIT = 0,
    CUI_DIRECTION_LTR,
    CUI_DIRECTION_RTL,
    CUI_DIRECTION_AUTO,
}

/// cui_theme_mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_theme_mode {
    #[default]
    CUI_THEME_LIGHT = 0,
    CUI_THEME_DARK = 1,
}

/// cui_accent.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_accent {
    #[default]
    CUI_ACCENT_NEUTRAL = 0,
    CUI_ACCENT_VIOLET,
    CUI_ACCENT_BLUE,
    CUI_ACCENT_GREEN,
    CUI_ACCENT_AMBER,
    CUI_ACCENT_ORANGE,
    CUI_ACCENT_CORAL,
    CUI_ACCENT_ROSE,
}

/// cui_text_align.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_text_align {
    #[default]
    CUI_TEXT_START = 0,
    CUI_TEXT_CENTER,
    CUI_TEXT_RIGHT,
    CUI_TEXT_LEFT,
    CUI_TEXT_END,
}

/// cui_button_variant.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_button_variant {
    #[default]
    CUI_BUTTON_DEFAULT = 0,
    CUI_BUTTON_SECONDARY,
    CUI_BUTTON_PRIMARY,
    CUI_BUTTON_OUTLINE,
    CUI_BUTTON_GHOST,
    CUI_BUTTON_DANGER,
    CUI_BUTTON_LINK,
}

/// cui_control_size.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_control_size {
    #[default]
    CUI_SIZE_MD = 0,
    CUI_SIZE_SM,
    CUI_SIZE_LG,
}

/// cui_tone.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_tone {
    #[default]
    CUI_TONE_NEUTRAL = 0,
    CUI_TONE_INFO,
    CUI_TONE_SUCCESS,
    CUI_TONE_WARNING,
    CUI_TONE_DANGER,
    CUI_TONE_VIOLET,
    CUI_TONE_TEAL,
    CUI_TONE_ROSE,
    CUI_TONE_ACCENT,
}

/// cui_side.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_side {
    #[default]
    CUI_SIDE_RIGHT = 0,
    CUI_SIDE_LEFT,
    CUI_SIDE_BOTTOM,
    CUI_SIDE_TOP,
}

/// cui_placement.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_placement {
    #[default]
    CUI_PLACE_BOTTOM_START = 0,
    CUI_PLACE_BOTTOM,
    CUI_PLACE_BOTTOM_END,
    CUI_PLACE_TOP_START,
    CUI_PLACE_TOP,
    CUI_PLACE_TOP_END,
    CUI_PLACE_RIGHT_START,
    CUI_PLACE_LEFT_START,
    CUI_PLACE_RIGHT,
    CUI_PLACE_LEFT,
}

/// cui_tabs_variant.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_tabs_variant {
    #[default]
    CUI_TABS_UNDERLINE = 0,
    CUI_TABS_PILL,
}

/// cui_mouse_button.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_mouse_button {
    #[default]
    CUI_MOUSE_LEFT = 0,
    CUI_MOUSE_RIGHT = 1,
    CUI_MOUSE_MIDDLE = 2,
}

/// cui_key.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum cui_key {
    CUI_KEY_NONE = 0,
    CUI_KEY_TAB,
    CUI_KEY_ENTER,
    CUI_KEY_ESCAPE,
    CUI_KEY_BACKSPACE,
    CUI_KEY_DELETE,
    CUI_KEY_SPACE,
    CUI_KEY_LEFT,
    CUI_KEY_RIGHT,
    CUI_KEY_UP,
    CUI_KEY_DOWN,
    CUI_KEY_HOME,
    CUI_KEY_END,
    CUI_KEY_PAGE_UP,
    CUI_KEY_PAGE_DOWN,
    CUI_KEY_A,
    CUI_KEY_B,
    CUI_KEY_C,
    CUI_KEY_D,
    CUI_KEY_E,
    CUI_KEY_F,
    CUI_KEY_G,
    CUI_KEY_H,
    CUI_KEY_I,
    CUI_KEY_J,
    CUI_KEY_K,
    CUI_KEY_L,
    CUI_KEY_M,
    CUI_KEY_N,
    CUI_KEY_O,
    CUI_KEY_P,
    CUI_KEY_Q,
    CUI_KEY_R,
    CUI_KEY_S,
    CUI_KEY_T,
    CUI_KEY_U,
    CUI_KEY_V,
    CUI_KEY_W,
    CUI_KEY_X,
    CUI_KEY_Y,
    CUI_KEY_Z,
    CUI_KEY_0,
    CUI_KEY_1,
    CUI_KEY_2,
    CUI_KEY_3,
    CUI_KEY_4,
    CUI_KEY_5,
    CUI_KEY_6,
    CUI_KEY_7,
    CUI_KEY_8,
    CUI_KEY_9,
    CUI_KEY_MINUS,
    CUI_KEY_EQUAL,
    CUI_KEY_COMMA,
    CUI_KEY_PERIOD,
    CUI_KEY_SLASH,
    CUI_KEY_F1,
    CUI_KEY_F2,
    CUI_KEY_F3,
    CUI_KEY_F4,
    CUI_KEY_F5,
    CUI_KEY_F6,
    CUI_KEY_F7,
    CUI_KEY_F8,
    CUI_KEY_F9,
    CUI_KEY_F10,
    CUI_KEY_F11,
    CUI_KEY_F12,
}

/// cui_mod flags.
pub type cui_mod = c_int;
pub const CUI_MOD_SHIFT: c_int = 1 << 0;
pub const CUI_MOD_CTRL: c_int = 1 << 1;
pub const CUI_MOD_ALT: c_int = 1 << 2;
pub const CUI_MOD_SUPER: c_int = 1 << 3;

/// Text input event flags (return value of cui_input_ex and friends).
pub type cui_input_flags = c_int;
pub const CUI_CHANGED: c_int = 1;
pub const CUI_SUBMITTED: c_int = 2;
pub const CUI_FOCUSED: c_int = 4;
pub const CUI_BLURRED: c_int = 8;

/* ==== Opts structs (cui.h) ==== */

/// Options for every container (cui_box).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_box {
    pub _pad: c_int,
    pub id: *const c_char,
    pub dir: cui_dir,
    pub width: cui_size,
    pub height: cui_size,
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub pad: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub pad_top: f32,
    pub pad_right: f32,
    pub pad_bottom: f32,
    pub pad_left: f32,
    pub gap: f32,
    pub columns: c_int,
    pub min_column_width: f32,
    pub wrap: bool,
    pub align: cui_align,
    pub justify: cui_justify,
    pub bg: cui_color,
    pub border_color: cui_color,
    pub border: f32,
    pub radius: f32,
    pub radius_tl: f32,
    pub radius_tr: f32,
    pub radius_br: f32,
    pub radius_bl: f32,
    pub elevation: cui_elevation,
    pub scroll_x: bool,
    pub scroll_y: bool,
    pub clip: bool,
    pub opacity: f32,
    pub direction: cui_direction,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_card_opts {
    pub _pad: c_int,
    pub id: *const c_char,
    pub width: cui_size,
    pub height: cui_size,
    pub min_width: f32,
    pub max_width: f32,
    pub pad: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub gap: f32,
    pub dir: cui_dir,
    pub align: cui_align,
    pub wrap: bool,
    pub hoverable: bool,
    pub muted: bool,
    pub selected: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_scroll_opts {
    pub _pad: c_int,
    pub id: *const c_char,
    pub width: cui_size,
    pub height: cui_size,
    pub horizontal: bool,
    pub pad: f32,
    pub gap: f32,
    pub fade_edges: bool,
    pub fade_color: cui_color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_text_opts {
    pub _pad: c_int,
    pub font: cui_font,
    pub size: f32,
    pub color: cui_color,
    pub align: cui_text_align,
    pub nowrap: bool,
    pub tabular: bool,
    pub selectable: bool,
    pub width: cui_size,
    pub line_height: f32,
    pub direction: cui_direction,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_button_opts {
    pub _pad: c_int,
    pub variant: cui_button_variant,
    pub size: cui_control_size,
    pub icon: *const c_char,
    pub icon_right: *const c_char,
    pub disabled: bool,
    pub loading: bool,
    pub full_width: bool,
    pub selected: bool,
    pub tooltip: *const c_char,
    pub shortcut: *const c_char,
    pub width: cui_size,
    pub color: cui_color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_segmented_opts {
    pub _pad: c_int,
    pub size: cui_control_size,
    pub full_width: bool,
    pub icons: *const *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_check_opts {
    pub _pad: c_int,
    pub description: *const c_char,
    pub disabled: bool,
    pub indeterminate: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_slider_opts {
    pub _pad: c_int,
    pub step: f32,
    pub show_value: bool,
    pub format: *const c_char,
    pub disabled: bool,
    pub width: cui_size,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_chip_opts {
    pub _pad: c_int,
    pub icon: *const c_char,
    pub disabled: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_input_opts {
    pub _pad: c_int,
    pub label: *const c_char,
    pub placeholder: *const c_char,
    pub icon: *const c_char,
    pub hint: *const c_char,
    pub error: *const c_char,
    pub password: bool,
    pub disabled: bool,
    pub clearable: bool,
    pub autofocus: bool,
    pub read_only: bool,
    pub size: cui_control_size,
    pub width: cui_size,
    pub height: f32,
    pub prefix: *const c_char,
    pub suffix: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_number_opts {
    pub _pad: c_int,
    pub label: *const c_char,
    pub step: f64,
    pub decimals: c_int,
    pub disabled: bool,
    pub width: cui_size,
    pub suffix: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_select_opts {
    pub _pad: c_int,
    pub label: *const c_char,
    pub placeholder: *const c_char,
    pub icons: *const *const c_char,
    pub searchable: bool,
    pub disabled: bool,
    pub width: cui_size,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_tabs_opts {
    pub _pad: c_int,
    pub variant: cui_tabs_variant,
    pub icons: *const *const c_char,
    pub full_width: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_nav_opts {
    pub _pad: c_int,
    pub icon: *const c_char,
    pub badge: *const c_char,
    pub selected: bool,
    pub group: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_dialog_opts {
    pub _pad: c_int,
    pub description: *const c_char,
    pub width: f32,
    pub no_close_button: bool,
    pub persistent: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_sheet_opts {
    pub _pad: c_int,
    pub side: cui_side,
    pub size: f32,
    pub description: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_toast_opts {
    pub _pad: c_int,
    pub description: *const c_char,
    pub tone: cui_tone,
    pub duration: f32,
    pub action: *const c_char,
    pub show_timer: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_badge_opts {
    pub _pad: c_int,
    pub tone: cui_tone,
    pub dot: bool,
    pub solid: bool,
    pub icon: *const c_char,
    pub size: cui_control_size,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_empty_opts {
    pub _pad: c_int,
    pub icon: *const c_char,
    pub description: *const c_char,
    pub action: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_metric_opts {
    pub _pad: c_int,
    pub delta: f32,
    pub delta_label: *const c_char,
    pub spark: *const f32,
    pub spark_count: c_int,
    pub icon: *const c_char,
    pub width: cui_size,
    pub lower_is_better: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_table_column {
    pub title: *const c_char,
    pub width: cui_size,
    pub align: cui_text_align,
    pub sortable: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct cui_table_sort {
    pub column: c_int,
    pub descending: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_table_opts {
    pub _pad: c_int,
    pub sort: *mut cui_table_sort,
    pub height: cui_size,
    pub striped: bool,
    pub compact: bool,
    pub selectable: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_series {
    pub name: *const c_char,
    pub values: *const f32,
    pub count: c_int,
    pub color: cui_color,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_annotation_kind {
    #[default]
    CUI_ANNOTATION_X = 0,
    CUI_ANNOTATION_Y,
    CUI_ANNOTATION_POINT,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_chart_annotation {
    pub kind: cui_annotation_kind,
    pub x: f64,
    pub y: f64,
    pub label: *const c_char,
    pub detail: *const c_char,
    pub accent: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_chart_opts {
    pub _pad: c_int,
    pub width: cui_size,
    pub height: f32,
    pub labels: *const *const c_char,
    pub label_count: c_int,
    pub area: bool,
    pub no_grid: bool,
    pub no_legend: bool,
    pub stacked: bool,
    pub smooth: bool,
    pub format: *const c_char,
    pub min: f32,
    pub max: f32,
    pub annotations: *const cui_chart_annotation,
    pub annotation_count: c_int,
}

/* ==== Context and frame (cui.h) ==== */

extern "C" {
    pub fn cui_create(desc: *const c_void) -> *mut cui_ctx; // cui_ctx_desc
    pub fn cui_destroy(ui: *mut cui_ctx);
    pub fn cui_set_viewport(ui: *mut cui_ctx, width: f32, height: f32, scale: f32);
    pub fn cui_begin_frame(ui: *mut cui_ctx, dt: f32);
    pub fn cui_end_frame(ui: *mut cui_ctx);
    pub fn cui_viewport_size(ui: *mut cui_ctx) -> cui_vec2;
    pub fn cui_dpi_scale(ui: *mut cui_ctx) -> f32;
    pub fn cui_needs_redraw(ui: *mut cui_ctx) -> bool;
    pub fn cui_request_redraw(ui: *mut cui_ctx);
    pub fn cui_time(ui: *mut cui_ctx) -> f64;
    pub fn cui_delta_time(ui: *mut cui_ctx) -> f32;
    pub fn cui_frame_count(ui: *mut cui_ctx) -> u64;
}

/* ==== Input queries (cui.h) ==== */

extern "C" {
    pub fn cui_key_pressed(ui: *mut cui_ctx, key: cui_key) -> bool;
    pub fn cui_key_down(ui: *mut cui_ctx, key: cui_key) -> bool;
    pub fn cui_shortcut(ui: *mut cui_ctx, mods: c_int, key: cui_key) -> bool;
    pub fn cui_key_mods(ui: *mut cui_ctx) -> c_int;
    pub fn cui_mouse_pos(ui: *mut cui_ctx) -> cui_vec2;
    pub fn cui_mouse_down(ui: *mut cui_ctx, b: cui_mouse_button) -> bool;
    pub fn cui_mouse_clicked(ui: *mut cui_ctx, b: cui_mouse_button) -> bool;
    pub fn cui_wants_keyboard(ui: *mut cui_ctx) -> bool;
    pub fn cui_wants_mouse(ui: *mut cui_ctx) -> bool;
    pub fn cui_dropped_file_count(ui: *mut cui_ctx) -> c_int;
    pub fn cui_dropped_file(ui: *mut cui_ctx, index: c_int) -> *const c_char;
}

/* ==== Theme (cui.h) ==== */

extern "C" {
    pub fn cui_set_theme(ui: *mut cui_ctx, mode: cui_theme_mode);
    pub fn cui_get_theme_mode(ui: *mut cui_ctx) -> cui_theme_mode;
    pub fn cui_set_accent(ui: *mut cui_ctx, accent: cui_accent);
    pub fn cui_get_accent(ui: *mut cui_ctx) -> cui_accent;
    pub fn cui_set_reduced_motion(ui: *mut cui_ctx, reduced: bool);
    pub fn cui_color_of(ui: *mut cui_ctx, c: cui_color) -> cui_color;
    pub fn cui_token_color(ui: *mut cui_ctx, token: cui_token) -> cui_color;
    pub fn cui_oklch(l: f32, c: f32, h_degrees: f32, alpha: f32) -> cui_color;
    pub fn cui_mix(a: cui_color, b: cui_color, t: f32) -> cui_color;
    pub fn cui_alpha(c: cui_color, alpha: f32) -> cui_color;
    pub fn cui_accent_name(accent: cui_accent) -> *const c_char;
}

/* ==== Fonts (cui.h) ==== */

extern "C" {
    pub fn cui_font_load_memory(ui: *mut cui_ctx, data: *const c_void, size: usize, copy: bool) -> cui_font;
    pub fn cui_font_load_file(ui: *mut cui_ctx, path: *const c_char) -> cui_font;
    pub fn cui_font_set_role(ui: *mut cui_ctx, role: cui_font, font: cui_font);
    pub fn cui_font_add_fallback(ui: *mut cui_ctx, font: cui_font, fallback: cui_font);
}

pub const CUI_FONT_BODY: cui_font = 0;
pub const CUI_FONT_MEDIUM: cui_font = 1;
pub const CUI_FONT_DISPLAY: cui_font = 2;
pub const CUI_FONT_MONO: cui_font = 3;
pub const CUI_FONT_ICONS: cui_font = 4;

/* ==== Layout (cui.h) ==== */

extern "C" {
    pub fn cui_box_begin(ui: *mut cui_ctx, opts: *const cui_box) -> cui_id;
    pub fn cui_box_end(ui: *mut cui_ctx);
    pub fn cui_row_begin(ui: *mut cui_ctx, opts: *const cui_box) -> cui_id;
    pub fn cui_column_begin(ui: *mut cui_ctx, opts: *const cui_box) -> cui_id;
    pub fn cui_spacer(ui: *mut cui_ctx);
    pub fn cui_gap(ui: *mut cui_ctx, size: f32);
    pub fn cui_separator(ui: *mut cui_ctx);
    pub fn cui_card_begin(ui: *mut cui_ctx, opts: *const cui_card_opts) -> bool;
    pub fn cui_card_end(ui: *mut cui_ctx);
    pub fn cui_scroll_begin(ui: *mut cui_ctx, opts: *const cui_scroll_opts);
    pub fn cui_scroll_end(ui: *mut cui_ctx);
    pub fn cui_scroll_to(ui: *mut cui_ctx, scroll_id: *const c_char, y: f32);
}

/* ==== Identity (cui.h) ==== */

extern "C" {
    pub fn cui_make_id(ui: *mut cui_ctx, s: *const c_char) -> cui_id;
    pub fn cui_push_id(ui: *mut cui_ctx, s: *const c_char);
    pub fn cui_push_id_int(ui: *mut cui_ctx, value: c_int);
    pub fn cui_pop_id(ui: *mut cui_ctx);
    pub fn cui_last_id(ui: *mut cui_ctx) -> cui_id;
    pub fn cui_last_rect(ui: *mut cui_ctx) -> cui_rect;
    pub fn cui_get_rect(ui: *mut cui_ctx, id: cui_id) -> cui_rect;
    pub fn cui_last_hovered(ui: *mut cui_ctx) -> bool;
    pub fn cui_is_disabled(ui: *mut cui_ctx) -> bool;
    pub fn cui_push_disabled(ui: *mut cui_ctx, disabled: bool);
    pub fn cui_pop_disabled(ui: *mut cui_ctx);
}

/* ==== Text (cui.h) ==== */

extern "C" {
    pub fn cui_text_ex(ui: *mut cui_ctx, text: *const c_char, opts: *const cui_text_opts);
    pub fn cui_heading(ui: *mut cui_ctx, text: *const c_char, level: c_int);
    pub fn cui_label(ui: *mut cui_ctx, text: *const c_char);
    pub fn cui_secondary(ui: *mut cui_ctx, text: *const c_char);
    pub fn cui_muted(ui: *mut cui_ctx, text: *const c_char);
    pub fn cui_code(ui: *mut cui_ctx, text: *const c_char);
    pub fn cui_kbd(ui: *mut cui_ctx, keys: *const c_char);
    pub fn cui_link(ui: *mut cui_ctx, text: *const c_char) -> bool;
    pub fn cui_icon(ui: *mut cui_ctx, icon: *const c_char, size: f32, color: cui_color);
}

/* ==== Buttons (cui.h) ==== */

extern "C" {
    pub fn cui_button_ex(ui: *mut cui_ctx, label: *const c_char, opts: *const cui_button_opts) -> bool;
    pub fn cui_icon_button_ex(ui: *mut cui_ctx, icon: *const c_char, opts: *const cui_button_opts) -> bool;
    pub fn cui_segmented_ex(
        ui: *mut cui_ctx,
        id: *const c_char,
        items: *const *const c_char,
        count: c_int,
        index: *mut c_int,
        opts: *const cui_segmented_opts,
    ) -> bool;
}

/* ==== Selection controls (cui.h) ==== */

extern "C" {
    pub fn cui_checkbox_ex(ui: *mut cui_ctx, label: *const c_char, value: *mut bool, opts: *const cui_check_opts) -> bool;
    pub fn cui_switch_ex(ui: *mut cui_ctx, label: *const c_char, value: *mut bool, opts: *const cui_check_opts) -> bool;
    pub fn cui_radio_ex(ui: *mut cui_ctx, label: *const c_char, value: *mut c_int, option: c_int, opts: *const cui_check_opts) -> bool;
    pub fn cui_radio_group(
        ui: *mut cui_ctx,
        id: *const c_char,
        items: *const *const c_char,
        count: c_int,
        index: *mut c_int,
    ) -> bool;
    pub fn cui_slider_ex(
        ui: *mut cui_ctx,
        label: *const c_char,
        value: *mut f32,
        min: f32,
        max: f32,
        opts: *const cui_slider_opts,
    ) -> bool;
    pub fn cui_chip_ex(ui: *mut cui_ctx, label: *const c_char, selected: *mut bool, opts: *const cui_chip_opts) -> bool;
}

/* ==== Text input (cui.h) ==== */

extern "C" {
    pub fn cui_input_ex(ui: *mut cui_ctx, id: *const c_char, buf: *mut c_char, cap: usize, opts: *const cui_input_opts) -> c_int;
    pub fn cui_textarea_ex(ui: *mut cui_ctx, id: *const c_char, buf: *mut c_char, cap: usize, opts: *const cui_input_opts) -> c_int;
    pub fn cui_search_ex(ui: *mut cui_ctx, id: *const c_char, buf: *mut c_char, cap: usize, opts: *const cui_input_opts) -> c_int;
    pub fn cui_number_ex(
        ui: *mut cui_ctx,
        id: *const c_char,
        value: *mut f64,
        min: f64,
        max: f64,
        opts: *const cui_number_opts,
    ) -> bool;
    pub fn cui_select_ex(
        ui: *mut cui_ctx,
        id: *const c_char,
        items: *const *const c_char,
        count: c_int,
        index: *mut c_int,
        opts: *const cui_select_opts,
    ) -> bool;
}

/* ==== Navigation (cui.h) ==== */

extern "C" {
    pub fn cui_tabs_ex(
        ui: *mut cui_ctx,
        id: *const c_char,
        items: *const *const c_char,
        count: c_int,
        index: *mut c_int,
        opts: *const cui_tabs_opts,
    ) -> bool;
    pub fn cui_nav_item_ex(ui: *mut cui_ctx, label: *const c_char, opts: *const cui_nav_opts) -> bool;
    pub fn cui_nav_section(ui: *mut cui_ctx, title: *const c_char);
}

/* ==== Dialogs, toasts, tooltips (cui.h) ==== */

extern "C" {
    pub fn cui_dialog_begin(ui: *mut cui_ctx, title: *const c_char, open: *mut bool, opts: *const cui_dialog_opts) -> bool;
    pub fn cui_dialog_end(ui: *mut cui_ctx);
    pub fn cui_dialog_footer_begin(ui: *mut cui_ctx);
    pub fn cui_dialog_footer_end(ui: *mut cui_ctx);
    pub fn cui_sheet_begin(ui: *mut cui_ctx, title: *const c_char, open: *mut bool, opts: *const cui_sheet_opts) -> bool;
    pub fn cui_sheet_end(ui: *mut cui_ctx);
    pub fn cui_toast_ex(ui: *mut cui_ctx, title: *const c_char, opts: *const cui_toast_opts) -> c_int;
    pub fn cui_toast_action_clicked(ui: *mut cui_ctx, handle: c_int) -> bool;
    pub fn cui_toast_dismiss(ui: *mut cui_ctx, handle: c_int);
    pub fn cui_tooltip(ui: *mut cui_ctx, text: *const c_char);
}

/* ==== Feedback and display (cui.h) ==== */

extern "C" {
    pub fn cui_badge_ex(ui: *mut cui_ctx, text: *const c_char, opts: *const cui_badge_opts);
    pub fn cui_empty_state_ex(ui: *mut cui_ctx, title: *const c_char, opts: *const cui_empty_opts) -> bool;
    pub fn cui_metric_ex(ui: *mut cui_ctx, label: *const c_char, value: *const c_char, opts: *const cui_metric_opts);
    pub fn cui_progress_ex(ui: *mut cui_ctx, value: f32, opts: *const cui_progress_opts);
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_progress_opts {
    pub _pad: c_int,
    pub indeterminate: bool,
    pub tone: cui_tone,
    pub height: f32,
    pub label: *const c_char,
    pub show_value: bool,
    pub width: cui_size,
}

/* ==== Data display: table and charts (cui.h) ==== */

extern "C" {
    pub fn cui_table_begin_ex(
        ui: *mut cui_ctx,
        id: *const c_char,
        columns: *const cui_table_column,
        column_count: c_int,
        opts: *const cui_table_opts,
    ) -> bool;
    pub fn cui_table_row(ui: *mut cui_ctx, selected: bool) -> bool;
    pub fn cui_table_cell(ui: *mut cui_ctx);
    pub fn cui_table_cell_text(ui: *mut cui_ctx, text: *const c_char);
    pub fn cui_table_end(ui: *mut cui_ctx);

    pub fn cui_line_chart_ex(ui: *mut cui_ctx, id: *const c_char, series: *const cui_series, count: c_int, opts: *const cui_chart_opts);
    pub fn cui_bar_chart_ex(ui: *mut cui_ctx, id: *const c_char, series: *const cui_series, count: c_int, opts: *const cui_chart_opts);
    pub fn cui_donut_chart_ex(
        ui: *mut cui_ctx,
        id: *const c_char,
        values: *const f32,
        labels: *const *const c_char,
        count: c_int,
        opts: *const cui_chart_opts,
    );
    pub fn cui_sparkline(ui: *mut cui_ctx, values: *const f32, count: c_int, width: cui_size, height: f32, color: cui_color);
}

/* ==== App layer (cui_app.h) ==== */

/// cui_renderer_kind.
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_renderer_kind {
    #[default]
    CUI_RENDERER_AUTO = 0,
    CUI_RENDERER_OPENGL,
    CUI_RENDERER_SOFTWARE,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_app_desc {
    pub title: *const c_char,
    pub width: c_int,
    pub height: c_int,
    pub min_width: c_int,
    pub min_height: c_int,
    pub fixed_size: bool,
    pub continuous: bool,
    pub no_vsync: bool,
    pub hidden: bool,
    pub renderer: cui_renderer_kind,
    pub theme: cui_theme_mode,
    pub accent: cui_accent,
    pub follow_system_theme: bool,
    pub app_id: *const c_char,
}

extern "C" {
    pub fn cui_app_create(desc: *const cui_app_desc) -> *mut cui_app;
    pub fn cui_app_destroy(app: *mut cui_app);
    pub fn cui_app_frame(app: *mut cui_app) -> bool;
    pub fn cui_app_ui(app: *mut cui_app) -> *mut cui_ctx;
    pub fn cui_app_run(desc: *const cui_app_desc, frame: Option<extern "C" fn(ui: *mut cui_ctx, user: *mut c_void)>, user: *mut c_void) -> c_int;
    pub fn cui_app_quit(app: *mut cui_app);
    pub fn cui_app_set_title(app: *mut cui_app, title: *const c_char);
    pub fn cui_app_set_size(app: *mut cui_app, width: c_int, height: c_int);
    pub fn cui_app_size(app: *mut cui_app) -> cui_vec2;
    pub fn cui_app_is_software(app: *mut cui_app) -> bool;
    pub fn cui_app_native_window(app: *mut cui_app) -> *mut c_void;
    pub fn cui_app_backend(app: *mut cui_app) -> *const c_char;
    pub fn cui_command_palette(ui: *mut cui_ctx, open: *mut bool, commands: *const cui_command, count: c_int) -> c_int;
    pub fn cui_app_open_url(url: *const c_char) -> bool;
    pub fn cui_app_screenshot(app: *mut cui_app, rgba: *mut c_void, width: c_int, height: c_int) -> bool;
}


/* ==== Menus, split, conversation, instruments, canvas (cui.h) ==== */

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_menu_opts {
    pub _pad: c_int,
    pub icon: *const c_char,
    pub variant: cui_button_variant,
    pub placement: cui_placement,
    pub width: f32,
    pub icon_only: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_menu_item_opts {
    pub _pad: c_int,
    pub icon: *const c_char,
    pub shortcut: *const c_char,
    pub danger: bool,
    pub disabled: bool,
    pub checked: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_split_opts {
    pub _pad: c_int,
    pub vertical: bool,
    pub min_first: f32,
    pub min_second: f32,
    pub width: cui_size,
    pub height: cui_size,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_avatar_opts {
    pub _pad: c_int,
    pub size: f32,
    pub image: cui_texture,
    pub status: *const c_char,
    pub square: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_list_item_opts {
    pub _pad: c_int,
    pub icon: *const c_char,
    pub subtitle: *const c_char,
    pub trailing: *const c_char,
    pub selected: bool,
    pub avatar: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_user_menu_opts {
    pub _pad: c_int,
    pub detail: *const c_char,
    pub status: *const c_char,
    pub avatar: cui_texture,
    pub compact: bool,
    pub placement: cui_placement,
    pub width: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_timeline_opts {
    pub _pad: c_int,
    pub time: *const c_char,
    pub description: *const c_char,
    pub icon: *const c_char,
    pub tone: cui_tone,
    pub last: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_text_fx_opts {
    pub _pad: c_int,
    pub font: cui_font,
    pub size: f32,
    pub color: cui_color,
    pub width: cui_size,
    pub nowrap: bool,
    pub speed: f32,
    pub delay: f32,
    pub caret: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_chat_opts {
    pub _pad: c_int,
    pub mine: bool,
    pub author: *const c_char,
    pub time: *const c_char,
    pub status: *const c_char,
    pub grouped: bool,
    pub last_in_group: bool,
    pub avatar: cui_texture,
    pub markdown: bool,
    pub markdown_result: *mut c_void,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_task_status {
    #[default]
    CUI_TASK_PENDING = 0,
    CUI_TASK_RUNNING,
    CUI_TASK_DONE,
    CUI_TASK_FAILED,
    CUI_TASK_SKIPPED,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_reasoning_opts {
    pub _pad: c_int,
    pub thinking: bool,
    pub seconds: f32,
    pub title: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_tool_call_opts {
    pub _pad: c_int,
    pub status: cui_task_status,
    pub icon: *const c_char,
    pub detail: *const c_char,
    pub input: *const c_char,
    pub output: *const c_char,
    pub seconds: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_plan_step {
    pub title: *const c_char,
    pub detail: *const c_char,
    pub status: cui_task_status,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_agent_plan_opts {
    pub _pad: c_int,
    pub title: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_agent_status_opts {
    pub _pad: c_int,
    pub status: cui_task_status,
    pub seconds: f32,
    pub icon: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_source {
    pub title: *const c_char,
    pub url: *const c_char,
    pub snippet: *const c_char,
}

pub const CUI_RESPONSE_COPY: c_int = 1;
pub const CUI_RESPONSE_REGENERATE: c_int = 2;
pub const CUI_RESPONSE_UP: c_int = 4;
pub const CUI_RESPONSE_DOWN: c_int = 8;
pub const CUI_RESPONSE_SHARE: c_int = 16;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_response_actions_opts {
    pub _pad: c_int,
    pub copy_text: *const c_char,
    pub vote: *mut c_int,
    pub no_regenerate: bool,
    pub no_share: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_prompt_suggestion {
    pub icon: *const c_char,
    pub title: *const c_char,
    pub detail: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_prompt_suggestions_opts {
    pub _pad: c_int,
    pub columns: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_model {
    pub name: *const c_char,
    pub detail: *const c_char,
    pub badge: *const c_char,
    pub icon: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_model_picker_opts {
    pub _pad: c_int,
    pub ghost: bool,
    pub placement: cui_placement,
}

pub const CUI_COMPOSER_SEND: c_int = 1;
pub const CUI_COMPOSER_STOP: c_int = 2;
pub const CUI_COMPOSER_ATTACH: c_int = 4;
pub const CUI_COMPOSER_REMOVE: c_int = 8;
pub const CUI_COMPOSER_MODEL: c_int = 16;
pub const CUI_COMPOSER_RECORD: c_int = 32;
pub const CUI_COMPOSER_CANCEL: c_int = 64;
pub const CUI_COMPOSER_VOICE: c_int = 128;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_composer_opts {
    pub _pad: c_int,
    pub placeholder: *const c_char,
    pub busy: bool,
    pub attachments: *const *const c_char,
    pub attachment_count: c_int,
    pub removed: *mut c_int,
    pub models: *const cui_model,
    pub model_count: c_int,
    pub model: *mut c_int,
    pub max_lines: c_int,
    pub width: cui_size,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_viz_item {
    pub label: *const c_char,
    pub value: f32,
    pub color: cui_color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_context_opts {
    pub _pad: c_int,
    pub parts: *const cui_viz_item,
    pub part_count: c_int,
    pub unit: *const c_char,
}

/* ==== Instruments: dot charts, event track, readouts (cui.h) ==== */

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_axis_units {
    #[default]
    CUI_AXIS_NUMBER = 0,
    CUI_AXIS_HOURS,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_dot_series {
    pub values: *const f32,
    pub name: *const c_char,
    pub accent: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_dot_chart_opts {
    pub _pad: c_int,
    pub width: cui_size,
    pub height: f32,
    pub x_min: f64,
    pub x_max: f64,
    pub x_units: cui_axis_units,
    pub x_format: *const c_char,
    pub x_step: f64,
    pub min: f32,
    pub max: f32,
    pub format: *const c_char,
    pub unit: *const c_char,
    pub pitch: f32,
    pub no_grid: bool,
    pub no_axis: bool,
    pub no_readout: bool,
    pub annotations: *const cui_chart_annotation,
    pub annotation_count: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_dot_chart_result {
    pub hovered: c_int,
    pub x: f64,
    pub value: f32,
    pub clicked: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_event_track_opts {
    pub _pad: c_int,
    pub width: cui_size,
    pub step: f64,
    pub units: cui_axis_units,
    pub format: *const c_char,
    pub label_step: f64,
    pub selected: *mut c_int,
    pub no_add: bool,
    pub no_remove: bool,
    pub no_axis: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_event_track_result {
    pub changed: bool,
    pub added: c_int,
    pub removed: c_int,
    pub selected: c_int,
    pub dragging: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_readout_opts {
    pub _pad: c_int,
    pub unit: *const c_char,
    pub detail: *const c_char,
    pub size: cui_control_size,
    pub tone: cui_tone,
    pub accent: bool,
    pub align: cui_text_align,
    pub width: cui_size,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_dot_display_opts {
    pub _pad: c_int,
    pub pitch: f32,
    pub color: cui_color,
    pub accent: bool,
    pub no_dim: bool,
    pub marquee: bool,
    pub align: cui_text_align,
    pub width: cui_size,
}

/* ==== Command palette, canvas, theme, file dialog ==== */

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct cui_command {
    pub title: *const c_char,
    pub icon: *const c_char,
    pub group: *const c_char,
    pub shortcut: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct cui_corners {
    pub tl: f32,
    pub tr: f32,
    pub br: f32,
    pub bl: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_canvas_result {
    pub rect: cui_rect,
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub mouse: cui_vec2,
}

pub type cui_paint_fn = Option<extern "C" fn(ui: *mut cui_ctx, rect: cui_rect, user: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_shadow {
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub alpha: f32,
}

#[repr(C)]
pub struct cui_theme {
    pub colors: [cui_color; 35],
    pub tone_sky: cui_color,
    pub tone_teal: cui_color,
    pub tone_green: cui_color,
    pub tone_amber: cui_color,
    pub tone_rose: cui_color,
    pub tone_orchid: cui_color,
    pub tone_violet: cui_color,
    pub radius_control: f32,
    pub radius_panel: f32,
    pub radius_surface: f32,
    pub control_sm: f32,
    pub control_md: f32,
    pub control_lg: f32,
    pub text_xs: f32,
    pub text_sm: f32,
    pub text_body: f32,
    pub text_lg: f32,
    pub heading: [f32; 4],
    pub shadow_resting: [cui_shadow; 2],
    pub shadow_raised: [cui_shadow; 2],
    pub shadow_floating: [cui_shadow; 2],
    pub motion: f32,
    pub font_scale: f32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum cui_file_dialog_kind {
    #[default]
    CUI_FILE_OPEN = 0,
    CUI_FILE_SAVE,
    CUI_FILE_FOLDER,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_file_dialog_opts {
    pub kind: cui_file_dialog_kind,
    pub title: *const c_char,
    pub filter_name: *const c_char,
    pub filter: *const c_char,
    pub default_name: *const c_char,
}

extern "C" {
    pub fn cui_menu_begin(ui: *mut cui_ctx, label: *const c_char, opts: *const cui_menu_opts) -> bool;
    pub fn cui_menu_end(ui: *mut cui_ctx);
    pub fn cui_context_menu_begin(ui: *mut cui_ctx, id: *const c_char) -> bool;
    pub fn cui_context_menu_end(ui: *mut cui_ctx);
    pub fn cui_menu_item_ex(ui: *mut cui_ctx, label: *const c_char, opts: *const cui_menu_item_opts) -> bool;
    pub fn cui_menu_separator(ui: *mut cui_ctx);
    pub fn cui_menu_label(ui: *mut cui_ctx, text: *const c_char);
    pub fn cui_split_begin(ui: *mut cui_ctx, id: *const c_char, ratio: *mut f32, opts: *const cui_split_opts);
    pub fn cui_split_next(ui: *mut cui_ctx);
    pub fn cui_split_end(ui: *mut cui_ctx);
    pub fn cui_user_menu_begin(ui: *mut cui_ctx, name: *const c_char, opts: *const cui_user_menu_opts) -> bool;
    pub fn cui_user_menu_end(ui: *mut cui_ctx);
    pub fn cui_avatar_ex(ui: *mut cui_ctx, name: *const c_char, opts: *const cui_avatar_opts);
    pub fn cui_avatar_group(ui: *mut cui_ctx, names: *const *const c_char, count: c_int, max_visible: c_int, size: f32);
    pub fn cui_list_item_ex(ui: *mut cui_ctx, title: *const c_char, opts: *const cui_list_item_opts) -> bool;
    pub fn cui_pagination(ui: *mut cui_ctx, id: *const c_char, page: *mut c_int, page_count: c_int) -> bool;
    pub fn cui_breadcrumb(ui: *mut cui_ctx, items: *const *const c_char, count: c_int) -> c_int;
    pub fn cui_timeline_item_ex(ui: *mut cui_ctx, title: *const c_char, opts: *const cui_timeline_opts);
    pub fn cui_copy_button(ui: *mut cui_ctx, id: *const c_char, text: *const c_char) -> bool;
    pub fn cui_progress_ring(ui: *mut cui_ctx, label: *const c_char, value: f32, size: f32);
    pub fn cui_typewriter_ex(ui: *mut cui_ctx, id: *const c_char, text: *const c_char, opts: *const cui_text_fx_opts) -> bool;
    pub fn cui_number_roll_ex(ui: *mut cui_ctx, id: *const c_char, text: *const c_char, opts: *const cui_text_fx_opts);
    pub fn cui_chat_message_ex(ui: *mut cui_ctx, text: *const c_char, opts: *const cui_chat_opts);
    pub fn cui_reasoning_begin(ui: *mut cui_ctx, id: *const c_char, opts: *const cui_reasoning_opts) -> bool;
    pub fn cui_reasoning_end(ui: *mut cui_ctx);
    pub fn cui_tool_call_ex(ui: *mut cui_ctx, id: *const c_char, name: *const c_char, opts: *const cui_tool_call_opts);
    pub fn cui_agent_plan_ex(ui: *mut cui_ctx, id: *const c_char, steps: *const cui_plan_step, count: c_int, opts: *const cui_agent_plan_opts) -> c_int;
    pub fn cui_agent_status_ex(ui: *mut cui_ctx, text: *const c_char, opts: *const cui_agent_status_opts);
    pub fn cui_source_list(ui: *mut cui_ctx, id: *const c_char, sources: *const cui_source, count: c_int) -> c_int;
    pub fn cui_citation(ui: *mut cui_ctx, number: c_int, source: *const cui_source) -> bool;
    pub fn cui_response_actions_ex(ui: *mut cui_ctx, id: *const c_char, opts: *const cui_response_actions_opts) -> c_int;
    pub fn cui_prompt_suggestions_ex(ui: *mut cui_ctx, id: *const c_char, items: *const cui_prompt_suggestion, count: c_int, opts: *const cui_prompt_suggestions_opts) -> c_int;
    pub fn cui_model_picker_ex(ui: *mut cui_ctx, id: *const c_char, models: *const cui_model, count: c_int, selected: *mut c_int, opts: *const cui_model_picker_opts) -> bool;
    pub fn cui_chat_composer_ex(ui: *mut cui_ctx, id: *const c_char, buf: *mut c_char, cap: usize, opts: *const cui_composer_opts) -> c_int;
    pub fn cui_context_meter_ex(ui: *mut cui_ctx, id: *const c_char, used: f32, limit: f32, opts: *const cui_context_opts);
    pub fn cui_dot_chart_ex(ui: *mut cui_ctx, id: *const c_char, series: *const cui_dot_series, series_count: c_int, count: c_int, opts: *const cui_dot_chart_opts) -> cui_dot_chart_result;
    pub fn cui_event_track_ex(ui: *mut cui_ctx, id: *const c_char, times: *mut f64, count: *mut c_int, capacity: c_int, min: f64, max: f64, opts: *const cui_event_track_opts) -> cui_event_track_result;
    pub fn cui_readout_ex(ui: *mut cui_ctx, label: *const c_char, value: *const c_char, opts: *const cui_readout_opts);
    pub fn cui_dot_display_ex(ui: *mut cui_ctx, text: *const c_char, opts: *const cui_dot_display_opts);
    pub fn cui_canvas(ui: *mut cui_ctx, id: *const c_char, width: cui_size, height: cui_size, paint: cui_paint_fn, user: *const c_void, user_size: usize) -> cui_canvas_result;
    pub fn cui_state(ui: *mut cui_ctx, id: cui_id, size: usize) -> *mut c_void;
    pub fn cui_draw_rect(ui: *mut cui_ctx, r: cui_rect, radius: cui_corners, fill: cui_color);
    pub fn cui_draw_rect_gradient(ui: *mut cui_ctx, r: cui_rect, radius: cui_corners, from: cui_color, to: cui_color);
    pub fn cui_draw_rect_border(ui: *mut cui_ctx, r: cui_rect, radius: cui_corners, fill: cui_color, border: cui_color, width: f32);
    pub fn cui_draw_shadow(ui: *mut cui_ctx, r: cui_rect, radius: cui_corners, blur: f32, spread: f32, offset: cui_vec2, color: cui_color);
    pub fn cui_draw_line(ui: *mut cui_ctx, a: cui_vec2, b: cui_vec2, thickness: f32, color: cui_color);
    pub fn cui_draw_polygon(ui: *mut cui_ctx, points: *const cui_vec2, count: c_int, color: cui_color);
    pub fn cui_draw_circle(ui: *mut cui_ctx, center: cui_vec2, radius: f32, color: cui_color);
    pub fn cui_draw_arc(ui: *mut cui_ctx, center: cui_vec2, radius: f32, a0: f32, a1: f32, thickness: f32, color: cui_color);
    pub fn cui_draw_text(ui: *mut cui_ctx, font: cui_font, size: f32, pos: cui_vec2, color: cui_color, text: *const c_char, len: c_int);
    pub fn cui_draw_icon(ui: *mut cui_ctx, icon: *const c_char, r: cui_rect, size: f32, color: cui_color);
    pub fn cui_push_clip(ui: *mut cui_ctx, r: cui_rect);
    pub fn cui_pop_clip(ui: *mut cui_ctx);
    pub fn cui_theme_get(ui: *mut cui_ctx) -> *mut cui_theme;
    pub fn cui_app_file_dialog(app: *mut cui_app, opts: *const cui_file_dialog_opts, out: *mut c_char, cap: usize) -> bool;

    /// Reads one log row; `text` receives a pointer into the buffer
    /// (C's `cui_log_get`).
    pub fn cui_log_get(buf: *const cui_log_buffer, i: i32, text: *mut *const c_char) -> *const cui_log_line;
}

pub type cui_file_list_fn = Option<extern "C" fn(user: *mut c_void, path: *const c_char, out: *mut cui_file_entry, cap: i32, names: *mut c_char, names_cap: usize, hidden: bool) -> i32>;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct cui_file_entry {
    pub name: *const c_char,
    pub dir: bool,
    pub size: u64,
    pub modified: i64,
}

pub type cui_gl_renderer = c_void;
pub type cui_sw_renderer = c_void;
pub type cui_gl_loader = Option<extern "C" fn(name: *const c_char) -> *mut c_void>;

mod gen;
pub use gen::*;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::format;
    use std::string::{String, ToString};

    /// The generated C ABI facts for the compiled version (probe/layout_probe.c
    /// via vendor.sh). Missing tables or entries mean vendor.sh needs a re-run.
    fn layout_table() -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, u64>> {
        use std::collections::BTreeMap;
        let path = format!("{}/layout/{}.toml", env!("CARGO_MANIFEST_DIR"), CUI_VERSION_STRING);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("no layout probe table for CUI {CUI_VERSION_STRING} ({e}); run ./vendor.sh")
        });
        let mut table: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
        let mut section = String::new();
        for line in src.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].to_string();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                panic!("bad line in {path}: {line}")
            };
            let key = key.trim().trim_matches('"').to_string();
            if key == "version" {
                continue; // recorded in the filename; not a number
            }
            let value: u64 = value
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("bad number in {path}: {line}"));
            table.entry(section.clone()).or_default().insert(key, value);
        }
        table
    }

    fn check(table: &std::collections::BTreeMap<String, std::collections::BTreeMap<String, u64>>, section: &str, key: &str, actual: u64) {
        let expected = table
            .get(section)
            .and_then(|s| s.get(key))
            .unwrap_or_else(|| {
                panic!("layout table for CUI {CUI_VERSION_STRING} lacks [{section}] {key}; regenerate with ./vendor.sh")
            });
        assert_eq!(actual, *expected, "[{section}] {key}: Rust FFI vs C ABI (CUI {CUI_VERSION_STRING})");
    }

    /// Struct sizes and field offsets are checked exhaustively by the
    /// generated tests/layout.rs; these spot checks pin the enum values the
    /// hand-written layer cares about.
    #[test]
    fn enum_values_match_c() {
        let table = layout_table();
        check(&table, "enum", "CUI_GRID", cui_dir::CUI_GRID as i32 as u64);
        check(&table, "enum", "CUI_ALIGN_STRETCH", cui_align::CUI_ALIGN_STRETCH as i32 as u64);
        check(&table, "enum", "CUI_JUSTIFY_BETWEEN", cui_justify::CUI_JUSTIFY_BETWEEN as i32 as u64);
        check(&table, "enum", "CUI_ELEVATION_FLOATING", cui_elevation::CUI_ELEVATION_FLOATING as i32 as u64);
        check(&table, "enum", "CUI_DIRECTION_AUTO", cui_direction::CUI_DIRECTION_AUTO as i32 as u64);
        check(&table, "enum", "CUI_THEME_DARK", cui_theme_mode::CUI_THEME_DARK as i32 as u64);
        check(&table, "enum", "CUI_ACCENT_ROSE", cui_accent::CUI_ACCENT_ROSE as i32 as u64);
        check(&table, "enum", "CUI_BUTTON_LINK", cui_button_variant::CUI_BUTTON_LINK as i32 as u64);
        check(&table, "enum", "CUI_SIZE_LG", cui_control_size::CUI_SIZE_LG as i32 as u64);
        check(&table, "enum", "CUI_TEXT_END", cui_text_align::CUI_TEXT_END as i32 as u64);
        check(&table, "enum", "CUI_TONE_ACCENT", cui_tone::CUI_TONE_ACCENT as i32 as u64);
        check(&table, "enum", "CUI_SIDE_TOP", cui_side::CUI_SIDE_TOP as i32 as u64);
        check(&table, "enum", "CUI_MOUSE_MIDDLE", cui_mouse_button::CUI_MOUSE_MIDDLE as i32 as u64);
        check(&table, "enum", "CUI_RENDERER_SOFTWARE", cui_renderer_kind::CUI_RENDERER_SOFTWARE as i32 as u64);
        check(&table, "enum", "CUI_KEY_9", cui_key::CUI_KEY_9 as i32 as u64);
        check(&table, "enum", "CUI_KEY_F12", cui_key::CUI_KEY_F12 as i32 as u64);
    }
}
