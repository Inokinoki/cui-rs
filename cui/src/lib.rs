//! # cui — safe Rust bindings for CUI
//!
//! [`CUI`](https://github.com/yappologistic/CUI) is an immediate-mode UI
//! library written in C99: you describe the screen with function calls every
//! frame, and CUI lays it out, handles input and draws it. This crate wraps
//! the C API one-to-one, so the C documentation and examples translate
//! directly:
//!
//! | C | Rust |
//! |---|---|
//! | `cui_button(ui, "Save")` | `ui.button("Save")` |
//! | `cui_button(ui, "Save", .variant = CUI_BUTTON_PRIMARY)` | `ui.button_with("Save", &Button { variant: ButtonVariant::Primary, ..Default::default() })` |
//! | `cui_column(ui, .gap = 16) { ... }` | `ui.column(&Column { gap: 16., ..Default::default() }, \|ui\| { ... })` |
//! | `cui_input(ui, "id", buf, sizeof buf)` | `ui.input("id", &mut my_string)` |
//!
//! The app owns its state in plain variables; widgets read and write it
//! through references and return what happened this frame:
//!
//! ```no_run
//! use cui::{App, AppDesc, Button, ButtonVariant, Column, Row, Size};
//!
//! let mut app = App::new(&AppDesc { title: Some("Hello"), width: 420, height: 300, ..Default::default() })
//!     .expect("could not create window");
//! let mut notifications = true;
//! let mut clicks = 0;
//!
//! while app.frame() {
//!     let mut ui = app.ui();
//!     ui.column(&Column { pad: 32., gap: 16., width: Size::FILL, ..Default::default() }, |ui| {
//!         ui.heading("Hello, CUI", 2);
//!         ui.switch("Notifications", &mut notifications);
//!         ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
//!             if ui.button_with("Click me", &Button { variant: ButtonVariant::Primary, ..Default::default() }) {
//!                 clicks += 1;
//!             }
//!             ui.text(format!("Clicked {clicks} times"));
//!         });
//!     });
//! }
//! ```
//!
//! Strings passed to widgets may be temporaries: they are copied into a
//! per-frame arena that lives exactly as long as CUI needs them. State owned
//! by CUI (scroll positions, focus, hover fades) is keyed by each widget's id
//! — see the "Identity" notes in the C header: `"Label##key"` and
//! `"Label###key"` work exactly as in C.
//!
//! A `cui::App` can be moved between threads but used from only one (as in
//! C: one `cui_ctx` per thread). This crate binds CUI's own window layer
//! (`cui_app_*`); see `cui-sys` for the raw FFI, including the renderer SPI
//! for bringing your own window.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[macro_use]
mod types;

mod arena;
mod draw;
mod ext;
mod gen_safe;
mod icons;
mod opts;
mod opts2;

pub use arena::Arena;
pub use draw::*;
pub use ext::{
    date_from_days, date_to_days, date_today, file_dialog, http_method_name, http_reason, key_name,
    log_get, query_op_name,
};
pub use icons::*;
pub use opts::*;
pub use opts2::*;
pub use types::*;

use cui_sys as ffi;
use core::ffi::{c_char, c_void};
use core::marker::PhantomData;
use core::ptr;

/// Prepares `value`'s buffer for an in-place edit by CUI.
///
/// CUI reads the current text with `strlen` and writes a NUL-terminated
/// string back, bounded by the buffer's capacity. The buffer is filled with
/// NULs first, so the read-back never has to look at uninitialized bytes.
/// `reserve` is the capacity a fresh string gets.
pub(crate) fn begin_c_string(value: &mut String, reserve: usize) {
    if value.capacity() == 0 {
        value.reserve(reserve);
    }
    let cap = value.capacity();
    // NUL-terminate the text and fill the rest of the buffer.
    unsafe { value.as_mut_vec().resize(cap, 0) };
}

/// Reads back the NUL-terminated text CUI wrote into `value`'s buffer.
///
/// The text lives in `value`'s own allocation, so it must not be copied onto
/// itself: `String::push_str` asks `ptr::copy_nonoverlapping` to copy a
/// range onto itself, which is undefined behaviour. The bytes are already
/// where they belong, so the valid-UTF-8 case only adopts the new length.
pub(crate) fn sync_c_string(value: &mut String) {
    let cap = value.capacity();
    let len = {
        let bytes = unsafe { core::slice::from_raw_parts(value.as_ptr().cast::<u8>(), cap) };
        bytes.iter().position(|&b| b == 0).unwrap_or(cap.saturating_sub(1))
    };
    let valid = {
        let bytes = unsafe { core::slice::from_raw_parts(value.as_ptr().cast::<u8>(), len) };
        core::str::from_utf8(bytes).is_ok()
    };
    if valid {
        // The bytes are in place already; just adopt the new length.
        unsafe { value.as_mut_vec().set_len(len) };
    } else {
        // A clipboard paste or an IME can hand CUI broken UTF-8: keep what
        // is valid.
        let text = {
            let bytes = unsafe { core::slice::from_raw_parts(value.as_ptr().cast::<u8>(), len) };
            String::from_utf8_lossy(bytes).into_owned()
        };
        unsafe { value.as_mut_vec().set_len(0) };
        value.push_str(&text);
    }
}

/// Font roles for [`Text::font`](opts::Text::font) and the font APIs.
pub mod font {
    use cui_sys as ffi;
    // ui_font is a plain int alias in C; constants suffice.
    /// Inter Regular (the default).
    pub const BODY: i32 = ffi::CUI_FONT_BODY;
    /// Inter Medium.
    pub const MEDIUM: i32 = ffi::CUI_FONT_MEDIUM;
    /// Geist Medium, for headings 28pt and up.
    pub const DISPLAY: i32 = ffi::CUI_FONT_DISPLAY;
    /// Geist Mono.
    pub const MONO: i32 = ffi::CUI_FONT_MONO;
    /// Lucide icons.
    pub const ICONS: i32 = ffi::CUI_FONT_ICONS;
}

/// The upstream CUI version compiled into your binary, e.g. `"1.4.0"`.
/// Extracted from the digest-verified release assets pinned in
/// `cui-sys/UPSTREAM.toml`, so it always matches the C code you run.
pub const CUI_VERSION: &str = cui_sys::CUI_VERSION_STRING;

// ----------------------------------------------------------------- App layer

/// Window and event loop description (C's `cui_app_desc`). Unset fields use
/// CUI's defaults (title "CUI", 1080×720, min 360×280, OpenGL when available).
#[derive(Clone, Debug, Default)]
pub struct AppDesc<'a> {
    /// Window title; `None` uses the CUI default.
    pub title: Option<&'a str>,
    /// Logical size; 0 uses the 1080×720 default.
    pub width: i32,
    /// Logical size; 0 uses the 1080×720 default.
    pub height: i32,
    /// Minimum size; 0 uses the 360×280 default.
    pub min_width: i32,
    /// Minimum size; 0 uses the 360×280 default.
    pub min_height: i32,
    /// Not resizable.
    pub fixed_size: bool,
    /// Render every frame even when idle (games, video).
    pub continuous: bool,
    /// OpenGL: do not wait for the display.
    pub no_vsync: bool,
    /// Create the window hidden (tests).
    pub hidden: bool,
    /// Which renderer to use; `Auto` picks OpenGL, falling back to software.
    pub renderer: RendererKind,
    /// Initial theme.
    pub theme: ThemeMode,
    /// Initial accent color.
    pub accent: Accent,
    /// Use the OS light/dark setting where available.
    pub follow_system_theme: bool,
    /// Linux: the app's ID, the name of its .desktop file without ".desktop".
    pub app_id: Option<&'a str>,
}

fn cstring(s: &str) -> Vec<u8> {
    assert!(!s.contains('\0'), "CUI strings must not contain NUL bytes: {s:?}");
    let mut v = Vec::with_capacity(s.len() + 1);
    v.extend_from_slice(s.as_bytes());
    v.push(0);
    v
}

/// A native window with an event loop and a renderer (C's `cui_app`).
///
/// The classic loop, matching C:
///
/// ```no_run
/// # use cui::{App, AppDesc};
/// # let mut app = App::new(&AppDesc::default()).unwrap();
/// while app.frame() {
///     let mut ui = app.ui();
///     // ... describe the screen ...
/// }
/// ```
pub struct App {
    ptr: *mut ffi::cui_app,
    arena: Arena,
}

// A cui_app is used from one thread; several apps may live on several threads,
// exactly like C's cui_app (see docs/integration.md, "Threads").
unsafe impl Send for App {}

impl App {
    /// Creates the window. Returns `None` when CUI could not (no display, no
    /// GL/software fallback, ...), like a `NULL` from `cui_app_create`.
    pub fn new(desc: &AppDesc) -> Option<App> {
        let title = desc.title.map(cstring);
        let app_id = desc.app_id.map(cstring);
        let d = ffi::cui_app_desc {
            title: title.as_ref().map_or(ptr::null(), |t| t.as_ptr() as *const c_char),
            width: desc.width,
            height: desc.height,
            min_width: desc.min_width,
            min_height: desc.min_height,
            fixed_size: desc.fixed_size,
            continuous: desc.continuous,
            no_vsync: desc.no_vsync,
            hidden: desc.hidden,
            renderer: match desc.renderer {
                RendererKind::Auto => ffi::cui_renderer_kind::CUI_RENDERER_AUTO,
                RendererKind::OpenGL => ffi::cui_renderer_kind::CUI_RENDERER_OPENGL,
                RendererKind::Software => ffi::cui_renderer_kind::CUI_RENDERER_SOFTWARE,
            },
            theme: match desc.theme {
                ThemeMode::Light => ffi::cui_theme_mode::CUI_THEME_LIGHT,
                ThemeMode::Dark => ffi::cui_theme_mode::CUI_THEME_DARK,
            },
            accent: desc.accent.to_ffi(),
            follow_system_theme: desc.follow_system_theme,
            app_id: app_id.as_ref().map_or(ptr::null(), |t| t.as_ptr() as *const c_char),
        };
        let ptr = unsafe { ffi::cui_app_create(&d) };
        (!ptr.is_null()).then(|| App { ptr, arena: Arena::new() })
    }

    /// Creates the app, calls `frame` until the window closes, destroys it,
    /// and returns the exit code. The Rust equivalent of `cui_app_run`, with
    /// a closure instead of a C function pointer.
    pub fn run(desc: &AppDesc, mut frame: impl FnMut(&mut Ui)) -> i32 {
        let mut app = match App::new(desc) {
            Some(app) => app,
            None => return 1,
        };
        while app.frame() {
            frame(&mut app.ui());
        }
        app.destroy();
        0
    }

    /// Presents the previous frame, waits for input or animation, begins a
    /// new frame. Returns false once the window was closed or [`App::quit`]
    /// was called. The frame arena (widget strings) is recycled here.
    pub fn frame(&mut self) -> bool {
        let ok = unsafe { ffi::cui_app_frame(self.ptr) };
        self.arena.reset();
        ok
    }

    /// The UI context for the current frame. Borrowed from the app; all
    /// widgets are methods on it.
    pub fn ui(&mut self) -> Ui<'_> {
        Ui { ctx: unsafe { ffi::cui_app_ui(self.ptr) }, arena: &self.arena }
    }

    /// Ends the loop after the current frame: [`App::frame`] returns false.
    pub fn quit(&mut self) {
        unsafe { ffi::cui_app_quit(self.ptr) }
    }

    /// Destroys the window. [`App::run`] does this for you; call this only in
    /// manual `new`/`frame` loops. Afterwards the app must not be used.
    pub fn destroy(self) {
        unsafe { ffi::cui_app_destroy(self.ptr) }
        // self.arena drops here
    }

    /// Sets the window title.
    pub fn set_title(&mut self, title: &str) {
        let t = cstring(title);
        unsafe { ffi::cui_app_set_title(self.ptr, t.as_ptr() as *const c_char) }
    }

    /// Sets the window size in logical points.
    pub fn set_size(&mut self, width: i32, height: i32) {
        unsafe { ffi::cui_app_set_size(self.ptr, width, height) }
    }

    /// The window's size in logical points.
    pub fn size(&mut self) -> Vec2 {
        let v = unsafe { ffi::cui_app_size(self.ptr) };
        Vec2 { x: v.x, y: v.y }
    }

    /// True when the software renderer is drawing (no usable OpenGL, or it was asked for).
    pub fn is_software(&mut self) -> bool {
        unsafe { ffi::cui_app_is_software(self.ptr) }
    }

    /// Name of the active backend: "win32", "cocoa", "x11", "sdl2", "glfw".
    pub fn backend(&mut self) -> &'static str {
        unsafe { core::ffi::CStr::from_ptr(ffi::cui_app_backend(self.ptr)).to_str().unwrap_or("") }
    }

    /// Opens a URL or file with the system's default handler.
    pub fn open_url(url: &str) -> bool {
        let u = cstring(url);
        unsafe { ffi::cui_app_open_url(u.as_ptr() as *const c_char) }
    }

    /// Renders the current UI into an RGBA8 buffer (any backend). Handy for
    /// screenshots and tests. Returns `None` when the window size is unknown.
    pub fn screenshot_rgba(&mut self) -> Option<Vec<u8>> {
        let size = self.size();
        let scale = unsafe { ffi::cui_dpi_scale(ffi::cui_app_ui(self.ptr)) } as i32;
        let (w, h) = (size.x as i32 * scale, size.y as i32 * scale);
        if w <= 0 || h <= 0 {
            return None;
        }
        let mut buf = vec![0u8; (w * h * 4) as usize];
        let ok = unsafe { ffi::cui_app_screenshot(self.ptr, buf.as_mut_ptr() as *mut c_void, w, h) };
        ok.then_some(buf)
    }
}

// ------------------------------------------------------------------ Ui handle

/// The UI context for one frame. Obtained from [`App::ui`]; all components
/// are methods on it. Child handles handed to closures share the same
/// context and arena.
pub struct Ui<'a> {
    pub(crate) ctx: *mut ffi::cui_ctx,
    pub(crate) arena: &'a Arena,
}

// The context is not thread safe (like C's cui_ctx); Ui wraps a raw pointer,
// so it is !Send and !Sync already.
impl<'a> Ui<'a> {
    fn new(ctx: *mut ffi::cui_ctx, arena: &'a Arena) -> Ui<'a> {
        Ui { ctx, arena }
    }

    /// The frame string arena, for building raw `ffi::*_opts` structs whose
    /// pointer fields (labels, formats, item arrays) must stay valid for one
    /// frame. Pointers from `cstr`/`cstr_array`/`struct_array` live until the
    /// frame ends — exactly what a CUI call needs.
    pub fn arena(&self) -> &'a Arena {
        self.arena
    }

    pub(crate) fn cs(&self, s: &impl AsRef<str>) -> *const c_char {
        self.arena.cstr(s.as_ref())
    }

    /// Runs `f` between a container's begin and end, ending the container
    /// even when `f` panics.
    fn scope(&mut self, end: unsafe extern "C" fn(*mut ffi::cui_ctx), f: impl FnOnce(&mut Ui<'a>)) {
        let guard = ScopeGuard { ctx: self.ctx, end };
        f(&mut Ui::new(self.ctx, self.arena));
        drop(guard);
    }

    // ------------------------------------------------------------ containers

    /// A column of children (C's `cui_column` scope).
    pub fn column(&mut self, o: &Column, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_column_begin(self.ctx, &fo) };
        self.scope(ffi::cui_box_end, f);
    }

    /// A row of children (C's `cui_row` scope).
    pub fn row(&mut self, o: &Row, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_row_begin(self.ctx, &fo) };
        self.scope(ffi::cui_box_end, f);
    }

    /// A box with an explicit direction (C's `cui_box` scope). Use
    /// [`Ui::column`] / [`Ui::row`] when you do not need `Dir::Stack`/`Grid`.
    pub fn box_(&mut self, o: &BoxOpts, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_box_begin(self.ctx, &fo) };
        self.scope(ffi::cui_box_end, f);
    }

    /// A bordered surface with generous padding (C's `cui_card` scope).
    /// Returns true when a `hoverable` card was clicked.
    pub fn card(&mut self, o: &Card, f: impl FnOnce(&mut Ui<'a>)) -> bool {
        let fo = o.flatten(self.arena);
        let clicked = unsafe { ffi::cui_card_begin(self.ctx, &fo) };
        self.scope(ffi::cui_card_end, f);
        clicked
    }

    /// A scroll area with smooth scrolling and overlay scrollbars
    /// (C's `cui_scroll` scope).
    pub fn scroll(&mut self, o: &Scroll, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_scroll_begin(self.ctx, &fo) };
        self.scope(ffi::cui_scroll_end, f);
    }

    /// Scrolls the scroll area with the given id to `y` points.
    pub fn scroll_to(&mut self, scroll_id: &str, y: f32) {
        unsafe { ffi::cui_scroll_to(self.ctx, self.cs(&scroll_id), y) }
    }

    /// Flexible space (C's `cui_spacer`).
    pub fn spacer(&mut self) {
        unsafe { ffi::cui_spacer(self.ctx) }
    }

    /// Fixed space in points (C's `cui_gap`).
    pub fn gap(&mut self, size: f32) {
        unsafe { ffi::cui_gap(self.ctx, size) }
    }

    /// A hairline that follows the parent direction (C's `cui_separator`).
    pub fn separator(&mut self) {
        unsafe { ffi::cui_separator(self.ctx) }
    }

    // ---------------------------------------------------------------- text

    /// A line of text (C's `cui_text`). Accepts `&str` or `format!(...)`.
    pub fn text(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_text_ex(self.ctx, self.cs(&text), &ffi::cui_text_opts::default()) }
    }

    /// A line of text with options ([`Text`]).
    pub fn text_with(&mut self, text: impl AsRef<str>, o: &Text) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_text_ex(self.ctx, self.cs(&text), &fo) }
    }

    /// A heading, level 1..4 (C's `cui_heading`).
    pub fn heading(&mut self, text: impl AsRef<str>, level: i32) {
        unsafe { ffi::cui_heading(self.ctx, self.cs(&text), level) }
    }

    /// Medium weight text (C's `cui_label`).
    pub fn label(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_label(self.ctx, self.cs(&text)) }
    }

    /// Secondary color text (C's `cui_secondary`).
    pub fn secondary(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_secondary(self.ctx, self.cs(&text)) }
    }

    /// Small, muted text (C's `cui_muted`).
    pub fn muted(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_muted(self.ctx, self.cs(&text)) }
    }

    /// Inline code (C's `cui_code`).
    pub fn code(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_code(self.ctx, self.cs(&text)) }
    }

    /// A keyboard keycap, e.g. "Ctrl K" (C's `cui_kbd`).
    pub fn kbd(&mut self, keys: impl AsRef<str>) {
        unsafe { ffi::cui_kbd(self.ctx, self.cs(&keys)) }
    }

    /// A hyperlink (C's `cui_link`). Returns true when clicked.
    pub fn link(&mut self, text: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_link(self.ctx, self.cs(&text)) }
    }

    /// An icon at a size in points, in a color (C's `cui_icon`).
    pub fn icon(&mut self, icon: &str, size: f32, color: Color) {
        unsafe { ffi::cui_icon(self.ctx, self.arena.cstr(icon), size, color_to_ffi(color)) }
    }

    // -------------------------------------------------------------- buttons

    /// A button (C's `cui_button`). Returns true when clicked.
    pub fn button(&mut self, label: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_button_ex(self.ctx, self.cs(&label), &ffi::cui_button_opts::default()) }
    }

    /// A button with options ([`Button`]). Returns true when clicked.
    pub fn button_with(&mut self, label: impl AsRef<str>, o: &Button) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_button_ex(self.ctx, self.cs(&label), &fo) }
    }

    /// A square/round button with only an icon (C's `cui_icon_button`).
    /// Returns true when clicked.
    pub fn icon_button(&mut self, icon: &str) -> bool {
        unsafe { ffi::cui_icon_button_ex(self.ctx, self.arena.cstr(icon), &ffi::cui_button_opts::default()) }
    }

    /// An icon button with options. Returns true when clicked.
    pub fn icon_button_with(&mut self, icon: &str, o: &Button) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_icon_button_ex(self.ctx, self.arena.cstr(icon), &fo) }
    }

    // ------------------------------------------------------------ selection

    /// A checkbox (C's `cui_checkbox`). Writes through `value`; returns true when changed.
    pub fn checkbox(&mut self, label: impl AsRef<str>, value: &mut bool) -> bool {
        unsafe { ffi::cui_checkbox_ex(self.ctx, self.cs(&label), value, &ffi::cui_check_opts::default()) }
    }

    /// A checkbox with options ([`Check`]). Returns true when changed.
    pub fn checkbox_with(&mut self, label: impl AsRef<str>, value: &mut bool, o: &Check) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_checkbox_ex(self.ctx, self.cs(&label), value, &fo) }
    }

    /// A switch (C's `cui_switch`). Writes through `value`; returns true when changed.
    pub fn switch(&mut self, label: impl AsRef<str>, value: &mut bool) -> bool {
        unsafe { ffi::cui_switch_ex(self.ctx, self.cs(&label), value, &ffi::cui_check_opts::default()) }
    }

    /// A switch with options. Returns true when changed.
    pub fn switch_with(&mut self, label: impl AsRef<str>, value: &mut bool, o: &Check) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_switch_ex(self.ctx, self.cs(&label), value, &fo) }
    }

    /// A radio button (C's `cui_radio`). Writes the chosen `option` into
    /// `value`; returns true when changed.
    pub fn radio(&mut self, label: impl AsRef<str>, value: &mut i32, option: i32) -> bool {
        unsafe { ffi::cui_radio_ex(self.ctx, self.cs(&label), value, option, &ffi::cui_check_opts::default()) }
    }

    /// A radio button with options. Returns true when changed.
    pub fn radio_with(&mut self, label: impl AsRef<str>, value: &mut i32, option: i32, o: &Check) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_radio_ex(self.ctx, self.cs(&label), value, option, &fo) }
    }

    /// A vertical list of radio buttons (C's `cui_radio_group`).
    /// Returns true when changed.
    pub fn radio_group(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32) -> bool {
        unsafe { ffi::cui_radio_group(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index) }
    }

    /// An elastic slider (C's `cui_slider`). Writes through `value`;
    /// returns true when changed.
    pub fn slider(&mut self, label: impl AsRef<str>, value: &mut f32, min: f32, max: f32) -> bool {
        unsafe { ffi::cui_slider_ex(self.ctx, self.cs(&label), value, min, max, &ffi::cui_slider_opts::default()) }
    }

    /// A slider with options ([`Slider`]). Returns true when changed.
    pub fn slider_with(&mut self, label: impl AsRef<str>, value: &mut f32, min: f32, max: f32, o: &Slider) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_slider_ex(self.ctx, self.cs(&label), value, min, max, &fo) }
    }

    /// A selectable chip (C's `cui_chip`). Writes through `selected`;
    /// returns true when changed.
    pub fn chip(&mut self, label: impl AsRef<str>, selected: &mut bool) -> bool {
        unsafe { ffi::cui_chip_ex(self.ctx, self.cs(&label), selected, &ffi::cui_chip_opts::default()) }
    }

    /// A chip with options. Returns true when changed.
    pub fn chip_with(&mut self, label: impl AsRef<str>, selected: &mut bool, o: &Chip) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_chip_ex(self.ctx, self.cs(&label), selected, &fo) }
    }

    /// A segmented control (C's `cui_segmented`). Writes the chosen index
    /// into `index`; returns true when changed.
    pub fn segmented(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32) -> bool {
        unsafe {
            ffi::cui_segmented_ex(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index, &ffi::cui_segmented_opts::default())
        }
    }

    /// A segmented control with options ([`Segmented`]). Returns true when changed.
    pub fn segmented_with(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32, o: &Segmented) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_segmented_ex(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index, &fo) }
    }

    // ----------------------------------------------------------- text input

    /// A single line text field (C's `cui_input`).
    ///
    /// Edits `value` in place. The field can hold up to `value.capacity()`
    /// bytes — size it with `String::with_capacity`. Returns what happened
    /// this frame ([`InputFlags`]).
    pub fn input(&mut self, id: impl AsRef<str>, value: &mut String) -> InputFlags {
        self.text_field(ffi::cui_input_ex, id.as_ref(), value, &ffi::cui_input_opts::default())
    }

    /// A text field with options ([`Input`]).
    pub fn input_with(&mut self, id: impl AsRef<str>, value: &mut String, o: &Input) -> InputFlags {
        let fo = o.flatten(self.arena);
        self.text_field(ffi::cui_input_ex, id.as_ref(), value, &fo)
    }

    /// A multi-line text area with word wrap (C's `cui_textarea`).
    pub fn textarea(&mut self, id: impl AsRef<str>, value: &mut String) -> InputFlags {
        self.text_field(ffi::cui_textarea_ex, id.as_ref(), value, &ffi::cui_input_opts::default())
    }

    /// A text area with options.
    pub fn textarea_with(&mut self, id: impl AsRef<str>, value: &mut String, o: &Input) -> InputFlags {
        let fo = o.flatten(self.arena);
        self.text_field(ffi::cui_textarea_ex, id.as_ref(), value, &fo)
    }

    /// A search field with a search icon and a clear button (C's `cui_search`).
    pub fn search(&mut self, id: impl AsRef<str>, value: &mut String) -> InputFlags {
        self.text_field(ffi::cui_search_ex, id.as_ref(), value, &ffi::cui_input_opts::default())
    }

    /// A search field with options.
    pub fn search_with(&mut self, id: impl AsRef<str>, value: &mut String, o: &Input) -> InputFlags {
        let fo = o.flatten(self.arena);
        self.text_field(ffi::cui_search_ex, id.as_ref(), value, &fo)
    }

    fn text_field(
        &mut self,
        f: unsafe extern "C" fn(*mut ffi::cui_ctx, *const c_char, *mut c_char, usize, *const ffi::cui_input_opts) -> i32,
        id: &str,
        value: &mut String,
        opts: &ffi::cui_input_opts,
    ) -> InputFlags {
        let idp = self.arena.cstr(id);
        begin_c_string(value, 63);
        let cap = value.capacity();
        let flags = unsafe { f(self.ctx, idp, value.as_mut_ptr() as *mut c_char, cap, opts) };
        sync_c_string(value);
        InputFlags::from_bits(flags)
    }

    /// A bounded number field (C's `cui_number`). Writes through `value`;
    /// returns true when it changed.
    pub fn number(&mut self, id: impl AsRef<str>, value: &mut f64, min: f64, max: f64) -> bool {
        unsafe { ffi::cui_number_ex(self.ctx, self.cs(&id), value, min, max, &ffi::cui_number_opts::default()) }
    }

    /// A number field with options ([`Number`]). Returns true when changed.
    pub fn number_with(&mut self, id: impl AsRef<str>, value: &mut f64, min: f64, max: f64, o: &Number) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_number_ex(self.ctx, self.cs(&id), value, min, max, &fo) }
    }

    /// A dropdown select (C's `cui_select`). Writes the chosen index into
    /// `index`; returns true when changed.
    pub fn select(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32) -> bool {
        unsafe { ffi::cui_select_ex(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index, &ffi::cui_select_opts::default()) }
    }

    /// A select with options ([`Select`]). Returns true when changed.
    pub fn select_with(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32, o: &Select) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_select_ex(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index, &fo) }
    }

    // ------------------------------------------------------------ navigation

    /// A tab bar (C's `cui_tabs`). Writes the chosen index into `index`;
    /// returns true when changed.
    pub fn tabs(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32) -> bool {
        unsafe { ffi::cui_tabs_ex(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index, &ffi::cui_tabs_opts::default()) }
    }

    /// A tab bar with options ([`Tabs`]). Returns true when changed.
    pub fn tabs_with(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32, o: &Tabs) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_tabs_ex(self.ctx, self.cs(&id), self.arena.cstr_array(items), items.len() as i32, index, &fo) }
    }

    /// A sidebar navigation item (C's `cui_nav_item`). Returns true when clicked.
    pub fn nav_item(&mut self, label: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_nav_item_ex(self.ctx, self.cs(&label), &ffi::cui_nav_opts::default()) }
    }

    /// A navigation item with options ([`Nav`]). Returns true when clicked.
    pub fn nav_item_with(&mut self, label: impl AsRef<str>, o: &Nav) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_nav_item_ex(self.ctx, self.cs(&label), &fo) }
    }

    /// A section title between navigation items (C's `cui_nav_section`).
    pub fn nav_section(&mut self, title: impl AsRef<str>) {
        unsafe { ffi::cui_nav_section(self.ctx, self.cs(&title)) }
    }

    // --------------------------------------------------------------- overlay

    /// A modal dialog (C's `cui_dialog`). Drawn while `open` is true; closing
    /// it (the x, Esc, a click outside) sets it to false. The closure runs
    /// only while the dialog is open.
    pub fn dialog(&mut self, title: impl AsRef<str>, open: &mut bool, f: impl FnOnce(&mut Ui<'a>)) {
        self.dialog_impl(title.as_ref(), open, &ffi::cui_dialog_opts::default(), f);
    }

    /// A modal dialog with options ([`Dialog`]).
    pub fn dialog_with(&mut self, title: impl AsRef<str>, open: &mut bool, o: &Dialog, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        self.dialog_impl(title.as_ref(), open, &fo, f);
    }

    fn dialog_impl(&mut self, title: &str, open: &mut bool, fo: &ffi::cui_dialog_opts, f: impl FnOnce(&mut Ui<'a>)) {
        let is_open = unsafe { ffi::cui_dialog_begin(self.ctx, self.arena.cstr(title), open, fo) };
        if is_open {
            self.scope(ffi::cui_dialog_end, f);
        }
    }

    /// A right-aligned row for dialog actions (C's `cui_dialog_footer` scope).
    /// Call inside a dialog closure.
    pub fn dialog_footer(&mut self, f: impl FnOnce(&mut Ui<'a>)) {
        unsafe { ffi::cui_dialog_footer_begin(self.ctx) };
        self.scope(ffi::cui_dialog_footer_end, f);
    }

    /// Queue a toast (C's `cui_toast`). Use the returned [`ToastHandle`] with
    /// [`Ui::toast_action_clicked`] / [`Ui::toast_dismiss`].
    pub fn toast(&mut self, title: impl AsRef<str>) -> ToastHandle {
        ToastHandle(unsafe { ffi::cui_toast_ex(self.ctx, self.cs(&title), &ffi::cui_toast_opts::default()) })
    }

    /// A toast with options ([`Toast`]).
    pub fn toast_with(&mut self, title: impl AsRef<str>, o: &Toast) -> ToastHandle {
        let fo = o.flatten(self.arena);
        ToastHandle(unsafe { ffi::cui_toast_ex(self.ctx, self.cs(&title), &fo) })
    }

    /// True when the toast's action button was pressed before it started closing.
    pub fn toast_action_clicked(&mut self, handle: ToastHandle) -> bool {
        unsafe { ffi::cui_toast_action_clicked(self.ctx, handle.0) }
    }

    /// Dismiss a toast early.
    pub fn toast_dismiss(&mut self, handle: ToastHandle) {
        unsafe { ffi::cui_toast_dismiss(self.ctx, handle.0) }
    }

    /// A tooltip anchored to the previous component (C's `cui_tooltip`).
    pub fn tooltip(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_tooltip(self.ctx, self.cs(&text)) }
    }

    // ----------------------------------------------------- feedback, display

    /// A small label with a tone (C's `cui_badge`).
    pub fn badge(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_badge_ex(self.ctx, self.cs(&text), &ffi::cui_badge_opts::default()) }
    }

    /// A badge with options ([`Badge`]).
    pub fn badge_with(&mut self, text: impl AsRef<str>, o: &Badge) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_badge_ex(self.ctx, self.cs(&text), &fo) }
    }

    /// An empty state (C's `cui_empty_state`). Returns true when its action
    /// button is clicked.
    pub fn empty_state(&mut self, title: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_empty_state_ex(self.ctx, self.cs(&title), &ffi::cui_empty_opts::default()) }
    }

    /// An empty state with options ([`EmptyState`]). Returns true when its
    /// action button is clicked.
    pub fn empty_state_with(&mut self, title: impl AsRef<str>, o: &EmptyState) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_empty_state_ex(self.ctx, self.cs(&title), &fo) }
    }

    /// A labeled big number (C's `cui_metric`). `value` is the formatted text
    /// to show, e.g. `format!("${}", total)`.
    pub fn metric(&mut self, label: impl AsRef<str>, value: impl AsRef<str>) {
        unsafe { ffi::cui_metric_ex(self.ctx, self.cs(&label), self.cs(&value), &ffi::cui_metric_opts::default()) }
    }

    /// A metric with options ([`Metric`]).
    pub fn metric_with(&mut self, label: impl AsRef<str>, value: impl AsRef<str>, o: &Metric) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_metric_ex(self.ctx, self.cs(&label), self.cs(&value), &fo) }
    }

    /// A progress bar (C's `cui_progress`). `value` is 0..1.
    pub fn progress(&mut self, value: f32) {
        unsafe { ffi::cui_progress_ex(self.ctx, value, &ffi::cui_progress_opts::default()) }
    }

    /// A progress bar with options ([`Progress`]).
    pub fn progress_with(&mut self, value: f32, o: &Progress) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_progress_ex(self.ctx, value, &fo) }
    }

    // --------------------------------------------------------------- charts

    /// A line chart (C's `cui_line_chart`).
    pub fn line_chart(&mut self, id: impl AsRef<str>, series: &[Series]) {
        self.chart_impl(ffi::cui_line_chart_ex, id.as_ref(), series, &ffi::cui_chart_opts::default());
    }

    /// A line chart with options ([`Chart`]).
    pub fn line_chart_with(&mut self, id: impl AsRef<str>, series: &[Series], o: &Chart) {
        let fo = o.flatten(self.arena);
        self.chart_impl(ffi::cui_line_chart_ex, id.as_ref(), series, &fo);
    }

    /// A bar chart (C's `cui_bar_chart`).
    pub fn bar_chart(&mut self, id: impl AsRef<str>, series: &[Series]) {
        self.chart_impl(ffi::cui_bar_chart_ex, id.as_ref(), series, &ffi::cui_chart_opts::default());
    }

    /// A bar chart with options.
    pub fn bar_chart_with(&mut self, id: impl AsRef<str>, series: &[Series], o: &Chart) {
        let fo = o.flatten(self.arena);
        self.chart_impl(ffi::cui_bar_chart_ex, id.as_ref(), series, &fo);
    }

    fn chart_impl(
        &mut self,
        f: unsafe extern "C" fn(*mut ffi::cui_ctx, *const c_char, *const ffi::cui_series, i32, *const ffi::cui_chart_opts),
        id: &str,
        series: &[Series],
        opts: &ffi::cui_chart_opts,
    ) {
        assert!(!series.is_empty());
        let flat: Vec<ffi::cui_series> = series.iter().map(|s| s.flatten(self.arena)).collect();
        unsafe { f(self.ctx, self.arena.cstr(id), flat.as_ptr(), flat.len() as i32, opts) }
    }

    /// A donut chart (C's `cui_donut_chart`). One `values` entry per `labels` entry.
    pub fn donut_chart(&mut self, id: impl AsRef<str>, values: &[f32], labels: &[&str]) {
        self.donut_impl(id.as_ref(), values, labels, &ffi::cui_chart_opts::default());
    }

    /// A donut chart with options ([`Chart`]).
    pub fn donut_chart_with(&mut self, id: impl AsRef<str>, values: &[f32], labels: &[&str], o: &Chart) {
        let fo = o.flatten(self.arena);
        self.donut_impl(id.as_ref(), values, labels, &fo);
    }

    fn donut_impl(&mut self, id: &str, values: &[f32], labels: &[&str], opts: &ffi::cui_chart_opts) {
        assert_eq!(values.len(), labels.len(), "donut chart: one label per value");
        assert!(!values.is_empty());
        unsafe {
            ffi::cui_donut_chart_ex(self.ctx, self.arena.cstr(id), values.as_ptr(), self.arena.cstr_array(labels), values.len() as i32, opts)
        }
    }

    /// A small inline chart (C's `cui_sparkline`).
    pub fn sparkline(&mut self, values: &[f32], width: Size, height: f32, color: Color) {
        assert!(!values.is_empty());
        unsafe { ffi::cui_sparkline(self.ctx, values.as_ptr(), values.len() as i32, size_to_ffi(width), height, color_to_ffi(color)) }
    }

    // --------------------------------------------------------------- table

    /// Begins a table (C's `cui_table`). Returns true when rows should be
    /// built; pair with [`Ui::table_end`]:
    ///
    /// ```no_run
    /// # use cui::{Table, TableColumn, Ui};
    /// # fn f(ui: &mut Ui) {
    /// if ui.table_begin("orders", &[TableColumn { title: "Name", ..Default::default() }], &mut Table::default()) {
    ///     ui.table_row(false);
    ///     ui.table_cell_text("Alice");
    ///     ui.table_end();
    /// }
    /// # }
    /// ```
    pub fn table_begin(&mut self, id: impl AsRef<str>, columns: &[TableColumn], o: &mut Table) -> bool {
        assert!(!columns.is_empty());
        let flat: Vec<ffi::cui_table_column> = columns.iter().map(|c| c.flatten(self.arena)).collect();
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_table_begin_ex(self.ctx, self.arena.cstr(id.as_ref()), flat.as_ptr(), flat.len() as i32, &fo) }
    }

    /// Begins a row. Returns true when the row is clicked.
    pub fn table_row(&mut self, selected: bool) -> bool {
        unsafe { ffi::cui_table_row(self.ctx, selected) }
    }

    /// Begins the next cell; put components inside, ended implicitly by the
    /// next cell or row.
    pub fn table_cell(&mut self) {
        unsafe { ffi::cui_table_cell(self.ctx) }
    }

    /// A cell with plain text (C's `cui_table_cell_text`).
    pub fn table_cell_text(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_table_cell_text(self.ctx, self.cs(&text)) }
    }

    /// Ends the table opened by [`Ui::table_begin`].
    pub fn table_end(&mut self) {
        unsafe { ffi::cui_table_end(self.ctx) }
    }

    // ------------------------------------------------------------- identity

    /// Makes an id from a string (C's `cui_make_id`).
    pub fn make_id(&mut self, s: impl AsRef<str>) -> u32 {
        unsafe { ffi::cui_make_id(self.ctx, self.cs(&s)) }
    }

    /// Pushes a string id scope, so widgets made in a loop get distinct ids
    /// (C's `cui_push_id`). The returned guard pops the scope when dropped.
    pub fn push_id(&mut self, s: impl AsRef<str>) -> PopId<'a> {
        unsafe { ffi::cui_push_id(self.ctx, self.cs(&s)) };
        PopId { ctx: self.ctx, _frame: PhantomData }
    }

    /// Pushes an integer id scope (C's `cui_push_id_int`), e.g. a loop index.
    pub fn push_id_int(&mut self, value: i32) -> PopId<'a> {
        unsafe { ffi::cui_push_id_int(self.ctx, value) };
        PopId { ctx: self.ctx, _frame: PhantomData }
    }

    /// Pops an id scope explicitly (normally the [`PopId`] guard does it).
    pub fn pop_id(&mut self) {
        unsafe { ffi::cui_pop_id(self.ctx) }
    }

    /// The id of the most recent component (for tooltips, popovers).
    pub fn last_id(&mut self) -> u32 {
        unsafe { ffi::cui_last_id(self.ctx) }
    }

    /// The rectangle of the most recent component.
    pub fn last_rect(&mut self) -> Rect {
        let r = unsafe { ffi::cui_last_rect(self.ctx) };
        Rect { x: r.x, y: r.y, w: r.w, h: r.h }
    }

    /// Rectangle of any component or box id from the latest layout.
    pub fn get_rect(&mut self, id: u32) -> Rect {
        let r = unsafe { ffi::cui_get_rect(self.ctx, id) };
        Rect { x: r.x, y: r.y, w: r.w, h: r.h }
    }

    /// True when the most recent component is hovered.
    pub fn last_hovered(&mut self) -> bool {
        unsafe { ffi::cui_last_hovered(self.ctx) }
    }

    /// Disables every widget built while the returned guard is alive
    /// (C's `cui_push_disabled`).
    pub fn disabled(&mut self, disabled: bool) -> PopDisabled<'a> {
        unsafe { ffi::cui_push_disabled(self.ctx, disabled) };
        PopDisabled { ctx: self.ctx, _frame: PhantomData }
    }

    // ---------------------------------------------------------------- input

    /// True when the key was pressed this frame, with repeat (C's `cui_key_pressed`).
    pub fn key_pressed(&mut self, key: Key) -> bool {
        unsafe { ffi::cui_key_pressed(self.ctx, key.to_ffi()) }
    }

    /// True while the key is held down (C's `cui_key_down`).
    pub fn key_down(&mut self, key: Key) -> bool {
        unsafe { ffi::cui_key_down(self.ctx, key.to_ffi()) }
    }

    /// True when the modifier+key shortcut was pressed this frame, e.g.
    /// `ui.shortcut(Mods::PRIMARY, Key::K)` (C's `cui_shortcut`).
    pub fn shortcut(&mut self, mods: Mods, key: Key) -> bool {
        unsafe { ffi::cui_shortcut(self.ctx, mods.0, key.to_ffi()) }
    }

    /// The modifier keys held this frame (C's `cui_key_mods`).
    pub fn key_mods(&mut self) -> Mods {
        Mods(unsafe { ffi::cui_key_mods(self.ctx) })
    }

    /// The mouse position in logical points (C's `cui_mouse_pos`).
    pub fn mouse_pos(&mut self) -> Vec2 {
        let v = unsafe { ffi::cui_mouse_pos(self.ctx) };
        Vec2 { x: v.x, y: v.y }
    }

    /// True while the button is held down (C's `cui_mouse_down`).
    pub fn mouse_down(&mut self, b: MouseButton) -> bool {
        unsafe { ffi::cui_mouse_down(self.ctx, b.to_ffi()) }
    }

    /// True when the button was pressed this frame (C's `cui_mouse_clicked`).
    pub fn mouse_clicked(&mut self, b: MouseButton) -> bool {
        unsafe { ffi::cui_mouse_clicked(self.ctx, b.to_ffi()) }
    }

    /// True when a text field has keyboard focus, so the app should not treat
    /// typing as shortcuts (C's `cui_wants_keyboard`).
    pub fn wants_keyboard(&mut self) -> bool {
        unsafe { ffi::cui_wants_keyboard(self.ctx) }
    }

    /// True when the mouse is over a component that uses it (C's `cui_wants_mouse`).
    pub fn wants_mouse(&mut self) -> bool {
        unsafe { ffi::cui_wants_mouse(self.ctx) }
    }

    // ---------------------------------------------------------------- theme

    /// Switches the theme with a smooth animation (C's `cui_set_theme`).
    pub fn set_theme(&mut self, mode: ThemeMode) {
        unsafe { ffi::cui_set_theme(self.ctx, mode.to_ffi()) }
    }

    /// The current theme mode (C's `cui_get_theme_mode`).
    pub fn theme_mode(&mut self) -> ThemeMode {
        match unsafe { ffi::cui_get_theme_mode(self.ctx) } {
            ffi::cui_theme_mode::CUI_THEME_DARK => ThemeMode::Dark,
            _ => ThemeMode::Light,
        }
    }

    /// Sets the accent color (C's `cui_set_accent`).
    pub fn set_accent(&mut self, accent: Accent) {
        unsafe { ffi::cui_set_accent(self.ctx, accent.to_ffi()) }
    }

    /// The current accent (C's `cui_get_accent`).
    pub fn accent(&mut self) -> Accent {
        Accent::from_ffi(unsafe { ffi::cui_get_accent(self.ctx) })
    }

    /// Resolves a semantic color (a token) against the active theme
    /// (C's `cui_color_of`).
    pub fn color_of(&mut self, c: Color) -> Color {
        color_from_ffi(unsafe { ffi::cui_color_of(self.ctx, color_to_ffi(c)) })
    }

    /// The viewport size in logical points (C's `cui_viewport_size`).
    pub fn viewport_size(&mut self) -> Vec2 {
        let v = unsafe { ffi::cui_viewport_size(self.ctx) };
        Vec2 { x: v.x, y: v.y }
    }

    /// The DPI scale: 2.0 on a typical retina display (C's `cui_dpi_scale`).
    pub fn dpi_scale(&mut self) -> f32 {
        unsafe { ffi::cui_dpi_scale(self.ctx) }
    }

    /// Asks for a frame even with no input (C's `cui_request_redraw`), for
    /// things that keep moving.
    pub fn request_redraw(&mut self) {
        unsafe { ffi::cui_request_redraw(self.ctx) }
    }

    /// Seconds since the previous frame (C's `cui_delta_time`).
    pub fn delta_time(&mut self) -> f32 {
        unsafe { ffi::cui_delta_time(self.ctx) }
    }

    /// Seconds since the context was created (C's `cui_time`).
    pub fn time(&mut self) -> f64 {
        unsafe { ffi::cui_time(self.ctx) }
    }
}

/// Pops an id scope when dropped (see [`Ui::push_id`] / [`Ui::push_id_int`]).
#[must_use = "the id scope is popped when the guard is dropped"]
pub struct PopId<'a> {
    ctx: *mut ffi::cui_ctx,
    _frame: PhantomData<&'a ()>,
}

impl Drop for PopId<'_> {
    fn drop(&mut self) {
        unsafe { ffi::cui_pop_id(self.ctx) }
    }
}

/// Restores the disabled stack when dropped (see [`Ui::disabled`]).
#[must_use = "the disabled state is restored when the guard is dropped"]
pub struct PopDisabled<'a> {
    ctx: *mut ffi::cui_ctx,
    _frame: PhantomData<&'a ()>,
}

impl Drop for PopDisabled<'_> {
    fn drop(&mut self) {
        unsafe { ffi::cui_pop_disabled(self.ctx) }
    }
}

/// A queued toast (see [`Ui::toast`]). Keep the handle to detect action
/// clicks and to dismiss early.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToastHandle(i32);

struct ScopeGuard {
    ctx: *mut ffi::cui_ctx,
    end: unsafe extern "C" fn(*mut ffi::cui_ctx),
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        // Runs on unwind too, keeping CUI's container stack balanced.
        unsafe { (self.end)(self.ctx) }
    }
}

#[cfg(test)]
mod capability_cfg_tests {
    /// The DEP_CUI_* → cfg pipe works end to end: with the 1.4
    /// pin, the 1.4 gate is set and gates further up the ladder are not.
    #[cfg(cui_ge_1_4)]
    #[test]
    fn capability_cfg_flows() {
        // cfg! folds to a constant on purpose: the point is that the gate
        // was set by the build script at all.
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(cfg!(cui_ge_1_4));
            assert!(!cfg!(cui_ge_1_9));
        }
    }
}

#[cfg(test)]
mod c_string_tests {
    use super::{begin_c_string, sync_c_string};

    /// Writes `text` into the buffer the way CUI does: in place, NUL-terminated.
    fn c_write(buf: *mut u8, text: &[u8]) {
        unsafe { buf.copy_from_nonoverlapping(text.as_ptr(), text.len()) };
    }

    #[test]
    fn reads_back_what_cui_wrote() {
        let mut value = String::with_capacity(64);
        value.push_str("old");
        begin_c_string(&mut value, 64);
        c_write(value.as_mut_ptr(), b"new\0");
        sync_c_string(&mut value);
        assert_eq!(value, "new");
    }

    #[test]
    fn an_untouched_field_keeps_its_text() {
        let mut value = String::with_capacity(64);
        value.push_str("old");
        begin_c_string(&mut value, 64);
        sync_c_string(&mut value);
        assert_eq!(value, "old");
    }

    #[test]
    fn an_empty_field_stays_empty() {
        let mut value = String::with_capacity(64);
        begin_c_string(&mut value, 64);
        sync_c_string(&mut value);
        assert_eq!(value, "");
    }

    #[test]
    fn broken_utf8_keeps_what_is_valid() {
        let mut value = String::with_capacity(64);
        begin_c_string(&mut value, 64);
        c_write(value.as_mut_ptr(), b"a\xffb\0");
        sync_c_string(&mut value);
        assert_eq!(value, "a\u{fffd}b");
    }

    #[test]
    fn the_buffer_is_terminated_for_cui() {
        let mut value = String::with_capacity(64);
        value.push_str("abc");
        begin_c_string(&mut value, 64);
        // CUI reads the field with strlen: the text must be NUL-terminated,
        // and the whole buffer must be initialized.
        let bytes = unsafe { core::slice::from_raw_parts(value.as_ptr(), value.len()) };
        assert_eq!(bytes.len(), 64);
        assert_eq!(bytes[3], 0);
        assert!(bytes[4..].iter().all(|&b| b == 0));
    }
}
