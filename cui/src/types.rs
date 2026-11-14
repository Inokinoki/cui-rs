//! Basic types mirroring CUI's C enums and small structs, one-to-one.

use cui_sys as ffi;

/// A length along one axis. Mirrors C's `cui_size`
/// (`CUI_FIT` / `CUI_FILL` / `CUI_GROW(w)` / `CUI_PX(v)` / `CUI_PCT(p)`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // mirrors the C struct, field names match
#[repr(C)]
pub struct Size {
    pub kind: SizeKind,
    pub value: f32,
}

#[allow(missing_docs)] // mirrors the C enum
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum SizeKind {
    /// Take the content's size (the default).
    #[default]
    Fit = 0,
    /// Take the space left over. `value` is an optional minimum (1 = plain fill).
    Grow,
    /// Exactly `value` logical points.
    Fixed,
    /// `value` percent (0..100) of the parent.
    Percent,
}

impl Size {
    /// Take the content's size (CUI_FIT, the default).
    pub const FIT: Size = Size { kind: SizeKind::Fit, value: 0.0 };
    /// Take the space left over (CUI_FILL).
    pub const FILL: Size = Size { kind: SizeKind::Grow, value: 1.0 };
    /// `CUI_GROW(w)`: take the space left over, at least `w` points.
    pub const fn grow(w: f32) -> Size {
        Size { kind: SizeKind::Grow, value: w }
    }
    /// `CUI_PX(v)`: exactly `v` logical points.
    pub const fn px(v: f32) -> Size {
        Size { kind: SizeKind::Fixed, value: v }
    }
    /// `CUI_PCT(p)`: `p` percent (0..100) of the parent.
    pub const fn pct(p: f32) -> Size {
        Size { kind: SizeKind::Percent, value: p }
    }
}

impl SizeKind {
    pub(crate) fn to_ffi(self) -> ffi::cui_size_kind {
        match self {
            SizeKind::Fit => ffi::cui_size_kind::CUI_SIZE_FIT,
            SizeKind::Grow => ffi::cui_size_kind::CUI_SIZE_GROW,
            SizeKind::Fixed => ffi::cui_size_kind::CUI_SIZE_FIXED,
            SizeKind::Percent => ffi::cui_size_kind::CUI_SIZE_PERCENT,
        }
    }
}

pub(crate) fn size_to_ffi(s: Size) -> ffi::cui_size {
    ffi::cui_size { kind: s.kind.to_ffi(), value: s.value }
}

/// An RGBA color. Mirrors C's `cui_color`.
///
/// Prefer the semantic constants ([`Color::BACKGROUND`], [`Color::ACCENT`], …)
/// so your UI follows the theme and accent; raw colors are fine for charts and
/// custom drawing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(missing_docs)] // mirrors the C struct, field names match
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Semantic color token numbers, matching C's `cui_token`.
#[allow(missing_docs, dead_code)] // values are read via `as u8`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Token {
    Background = 1,
    Surface,
    SurfaceRaised,
    SurfaceMuted,
    Foreground,
    TextSecondary,
    TextMuted,
    Border,
    BorderSubtle,
    BorderStrong,
    Accent,
    AccentStrong,
    AccentSubtle,
    AccentForeground,
    Success,
    Warning,
    Danger,
    Info,
    ControlOn,
    ControlGlyph,
    ControlTrack,
    ControlTrackHover,
    ControlThumb,
    ControlThumbOn,
    Hover,
    Active,
    Scrim,
    FocusRing,
    Series1,
    Series2,
    Series3,
    Series4,
    Series5,
    Series6,
}

#[allow(missing_docs)] // self-named constants; the important ones below carry docs
impl Color {
    /// Opaque color from red, green and blue components (0..255).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }
    /// Color from red, green, blue and alpha components (0..255).
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }
    /// CUI's token encoding: an unresolved semantic color `{ token, 0xC5, 0x1D, 0 }`,
    /// resolved by the library against the active theme.
    const fn token(t: Token) -> Color {
        Color { r: t as u8, g: 0xC5, b: 0x1D, a: 0 }
    }

    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BACKGROUND: Color = Color::token(Token::Background);
    pub const SURFACE: Color = Color::token(Token::Surface);
    pub const SURFACE_RAISED: Color = Color::token(Token::SurfaceRaised);
    pub const SURFACE_MUTED: Color = Color::token(Token::SurfaceMuted);
    pub const FOREGROUND: Color = Color::token(Token::Foreground);
    pub const TEXT_SECONDARY: Color = Color::token(Token::TextSecondary);
    pub const TEXT_MUTED: Color = Color::token(Token::TextMuted);
    pub const BORDER: Color = Color::token(Token::Border);
    pub const BORDER_SUBTLE: Color = Color::token(Token::BorderSubtle);
    pub const BORDER_STRONG: Color = Color::token(Token::BorderStrong);
    pub const ACCENT: Color = Color::token(Token::Accent);
    pub const ACCENT_STRONG: Color = Color::token(Token::AccentStrong);
    pub const ACCENT_SUBTLE: Color = Color::token(Token::AccentSubtle);
    pub const ACCENT_FOREGROUND: Color = Color::token(Token::AccentForeground);
    pub const SUCCESS: Color = Color::token(Token::Success);
    pub const WARNING: Color = Color::token(Token::Warning);
    pub const DANGER: Color = Color::token(Token::Danger);
    pub const INFO: Color = Color::token(Token::Info);
    pub const HOVER: Color = Color::token(Token::Hover);
    pub const SCRIM: Color = Color::token(Token::Scrim);
    /// The nth chart series color (0-based, wraps mod 6).
    /// Multiplies the alpha channel by `alpha` (C's `cui_alpha`), a free
    /// function in C because it needs no context.
    pub fn with_alpha(self, alpha: f32) -> Color {
        let c = unsafe {
            cui_sys::cui_alpha(
                cui_sys::cui_color { r: self.r, g: self.g, b: self.b, a: self.a },
                alpha,
            )
        };
        Color { r: c.r, g: c.g, b: c.b, a: c.a }
    }

    pub const fn series(n: usize) -> Color {
        Color::token(match n % 6 {
            0 => Token::Series1,
            1 => Token::Series2,
            2 => Token::Series3,
            3 => Token::Series4,
            4 => Token::Series5,
            _ => Token::Series6,
        })
    }
}

pub(crate) fn color_to_ffi(c: Color) -> ffi::cui_color {
    ffi::cui_color { r: c.r, g: c.g, b: c.b, a: c.a }
}
pub(crate) fn color_from_ffi(c: ffi::cui_color) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a: c.a }
}

/// `{x, y}` in logical points. Mirrors C's `cui_vec2`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // mirrors the C struct, field names match
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

/// `{x, y, w, h}` in logical points. Mirrors C's `cui_rect`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[allow(missing_docs)] // mirrors the C struct, field names match
#[repr(C)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Defines a Rust enum mirroring a C enum: variants in the same order, the
/// first one the default, `to_ffi` a size-checked transmute.
macro_rules! ffi_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident { $first:ident = $fvalue:expr, $( $rest:ident $(= $rvalue:expr)? ),* $(,)? }
        as $ffi:ty
    ) => {
        $(#[$meta])*
        #[allow(missing_docs)] // variant names say what they mean
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        #[repr(u32)]
        pub enum $name {
            #[default]
            $first = $fvalue,
            $( $rest $(= $rvalue)?, )*
        }
        impl $name {
            #[allow(dead_code)] // not every enum is passed to C
            pub(crate) fn to_ffi(self) -> $ffi {
                // Same variant values and repr(u32) as the C enum; kept in
                // sync by construction and by the round-trip tests below.
                unsafe { core::mem::transmute(self) }
            }
        }
    };
}

ffi_enum! {
    /// Container direction (C's `cui_dir`). Used with `ui.box_`; `ui.row` and
    /// `ui.column` set it for you.
    pub enum Dir { Column = 0, Row, Stack, Grid }
    as ffi::cui_dir
}

ffi_enum! {
    /// Which renderer draws the app (C's `cui_renderer_kind`).
    pub enum RendererKind { Auto = 0, OpenGL, Software }
    as ffi::cui_renderer_kind
}

ffi_enum! {
    /// Cross-axis alignment of children in a container (C's `cui_align`).
    /// `Default` stretches children in columns and centers them in rows.
    pub enum Align { Default = 0, Start, Center, End, Stretch }
    as ffi::cui_align
}

ffi_enum! {
    /// Main-axis distribution in a container (C's `cui_justify`).
    pub enum Justify { Start = 0, Center, End, Between, Around }
    as ffi::cui_justify
}

ffi_enum! {
    /// Shadow depth of a box (C's `cui_elevation`).
    pub enum Elevation { None = 0, Resting, Raised, Floating }
    as ffi::cui_elevation
}

ffi_enum! {
    /// Text direction (C's `cui_direction`). `Inherit` follows the parent.
    pub enum Direction { Inherit = 0, Ltr, Rtl, Auto }
    as ffi::cui_direction
}

ffi_enum! {
    /// Light or dark theme (C's `cui_theme_mode`).
    pub enum ThemeMode { Light = 0, Dark = 1 }
    as ffi::cui_theme_mode
}

ffi_enum! {
    /// The theme's accent color (C's `cui_accent`).
    pub enum Accent { Neutral = 0, Violet, Blue, Green, Amber, Orange, Coral, Rose }
    as ffi::cui_accent
}

impl Accent {
    /// The accents in CUI's display order, for pickers.
    pub const ALL: [Accent; 8] = [
        Accent::Neutral, Accent::Violet, Accent::Blue, Accent::Green,
        Accent::Amber, Accent::Orange, Accent::Coral, Accent::Rose,
    ];

    /// The accent at `index` of [`Accent::ALL`] (the picker's order).
    pub fn from_index(index: i32) -> Accent {
        Self::ALL.get(index as usize).copied().unwrap_or(Accent::Neutral)
    }

    pub(crate) fn from_ffi(v: ffi::cui_accent) -> Accent {
        match v {
            ffi::cui_accent::CUI_ACCENT_VIOLET => Accent::Violet,
            ffi::cui_accent::CUI_ACCENT_BLUE => Accent::Blue,
            ffi::cui_accent::CUI_ACCENT_GREEN => Accent::Green,
            ffi::cui_accent::CUI_ACCENT_AMBER => Accent::Amber,
            ffi::cui_accent::CUI_ACCENT_ORANGE => Accent::Orange,
            ffi::cui_accent::CUI_ACCENT_CORAL => Accent::Coral,
            ffi::cui_accent::CUI_ACCENT_ROSE => Accent::Rose,
            _ => Accent::Neutral,
        }
    }
}

ffi_enum! {
    /// Button visual style (C's `cui_button_variant`). `Default` is secondary
    /// for buttons and ghost for icon buttons; `Primary` is the one main
    /// action.
    pub enum ButtonVariant { Default = 0, Secondary, Primary, Outline, Ghost, Danger, Link }
    as ffi::cui_button_variant
}

ffi_enum! {
    /// Control height preset (C's `cui_control_size`).
    pub enum ControlSize { Md = 0, Sm, Lg }
    as ffi::cui_control_size
}

ffi_enum! {
    /// Text alignment inside its box (C's `cui_text_align`).
    pub enum TextAlign { Start = 0, Center, Right, Left, End }
    as ffi::cui_text_align
}

ffi_enum! {
    /// A toast's or badge's semantic tone (C's `cui_tone`).
    pub enum Tone { Neutral = 0, Info, Success, Warning, Danger, Violet, Teal, Rose, Accent }
    as ffi::cui_tone
}

ffi_enum! {
    /// Which side a sheet slides in from (C's `cui_side`).
    pub enum Side { Right = 0, Left, Bottom, Top }
    as ffi::cui_side
}

ffi_enum! {
    /// Keyboard key (C's `cui_key`). Letters are `Key::A` … `Key::Z`, digits
    /// `Key::Num0` … `Key::Num9`.
    pub enum Key {
        None = 0,
        Tab, Enter, Escape, Backspace, Delete, Space,
        Left, Right, Up, Down,
        Home, End, PageUp, PageDown,
        A, B, C, D, E, F, G, H, I, J, K, L, M,
        N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
        Minus, Equal, Comma, Period, Slash,
        F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    }
    as ffi::cui_key
}

ffi_enum! {
    /// Mouse button (C's `cui_mouse_button`).
    pub enum MouseButton { Left = 0, Right = 1, Middle = 2 }
    as ffi::cui_mouse_button
}

/// Keyboard modifier flags, combinable with `|` (C's `cui_mod` bits).
#[allow(missing_docs)] // the i32 holds C's cui_mod bits
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods(pub i32);

impl Mods {
    /// Shift.
    pub const SHIFT: Mods = Mods(1 << 0);
    /// Control (Windows/Linux) — see [`Mods::PRIMARY`] for the platform's main modifier.
    pub const CTRL: Mods = Mods(1 << 1);
    /// Alt / Option.
    pub const ALT: Mods = Mods(1 << 2);
    /// Command on macOS, Windows key elsewhere.
    pub const SUPER: Mods = Mods(1 << 3);
    /// The platform's primary shortcut modifier: Command on macOS, Ctrl elsewhere.
    #[cfg(not(target_os = "macos"))]
    pub const PRIMARY: Mods = Mods::CTRL;
    #[cfg(target_os = "macos")]
    /// The platform's primary shortcut modifier: Command on macOS, Ctrl elsewhere.
    pub const PRIMARY: Mods = Mods::SUPER;

    /// True when every flag in `other` is also set here.
    pub const fn contains(self, other: Mods) -> bool {
        self.0 & other.0 == other.0
    }
    /// All flags from both operands.
    pub const fn union(self, other: Mods) -> Mods {
        Mods(self.0 | other.0)
    }
}

impl core::ops::BitOr for Mods {
    type Output = Mods;
    fn bitor(self, rhs: Mods) -> Mods {
        self.union(rhs)
    }
}

/// What happened to a text field this frame (C's `CUI_CHANGED` / `CUI_SUBMITTED` /
/// `CUI_FOCUSED` / `CUI_BLURRED` bits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputFlags(i32);

impl InputFlags {
    pub(crate) const fn from_bits(bits: i32) -> InputFlags {
        InputFlags(bits)
    }
    /// The text changed.
    pub const fn changed(self) -> bool {
        self.0 & ffi::CUI_CHANGED != 0
    }
    /// Enter was pressed (single-line fields).
    pub const fn submitted(self) -> bool {
        self.0 & ffi::CUI_SUBMITTED != 0
    }
    /// The field gained keyboard focus.
    pub const fn focused(self) -> bool {
        self.0 & ffi::CUI_FOCUSED != 0
    }
    /// The field lost keyboard focus.
    pub const fn blurred(self) -> bool {
        self.0 & ffi::CUI_BLURRED != 0
    }
}

impl Tone {
    /// The tones in CUI's display order, for pickers.
    pub const ALL: [Tone; 9] = [
        Tone::Neutral, Tone::Info, Tone::Success, Tone::Warning, Tone::Danger,
        Tone::Violet, Tone::Teal, Tone::Rose, Tone::Accent,
    ];

    /// The tone at `index` of [`Tone::ALL`].
    pub fn from_index(index: i32) -> Tone {
        Self::ALL.get(index as usize).copied().unwrap_or(Tone::Neutral)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_match_c() {
        assert_eq!(core::mem::size_of::<Size>(), 8);
        assert_eq!(core::mem::size_of::<Color>(), 4);
        assert_eq!(core::mem::size_of::<Vec2>(), 8);
        assert_eq!(core::mem::size_of::<Rect>(), 16);
        // Safe structs must equal their ffi counterparts.
        assert_eq!(core::mem::size_of::<Size>(), core::mem::size_of::<ffi::cui_size>());
        assert_eq!(core::mem::size_of::<Color>(), core::mem::size_of::<ffi::cui_color>());
    }

    #[test]
    fn enums_transmute_round_trip() {
        // The transmute in ffi_enum! relies on identical discriminants; spot
        // check every enum's first, one middle and last variant by name.
        assert_eq!(Dir::Grid.to_ffi(), ffi::cui_dir::CUI_GRID);
        assert_eq!(Align::Stretch.to_ffi(), ffi::cui_align::CUI_ALIGN_STRETCH);
        assert_eq!(Justify::Between.to_ffi(), ffi::cui_justify::CUI_JUSTIFY_BETWEEN);
        assert_eq!(Elevation::Floating.to_ffi(), ffi::cui_elevation::CUI_ELEVATION_FLOATING);
        assert_eq!(Direction::Auto.to_ffi(), ffi::cui_direction::CUI_DIRECTION_AUTO);
        assert_eq!(ThemeMode::Dark.to_ffi(), ffi::cui_theme_mode::CUI_THEME_DARK);
        assert_eq!(Accent::Rose.to_ffi(), ffi::cui_accent::CUI_ACCENT_ROSE);
        assert_eq!(ButtonVariant::Link.to_ffi(), ffi::cui_button_variant::CUI_BUTTON_LINK);
        assert_eq!(ControlSize::Lg.to_ffi(), ffi::cui_control_size::CUI_SIZE_LG);
        assert_eq!(TextAlign::End.to_ffi(), ffi::cui_text_align::CUI_TEXT_END);
        assert_eq!(Tone::Accent.to_ffi(), ffi::cui_tone::CUI_TONE_ACCENT);
        assert_eq!(Side::Top.to_ffi(), ffi::cui_side::CUI_SIDE_TOP);
        assert_eq!(MouseButton::Middle.to_ffi(), ffi::cui_mouse_button::CUI_MOUSE_MIDDLE);
        assert_eq!(RendererKind::Software.to_ffi(), ffi::cui_renderer_kind::CUI_RENDERER_SOFTWARE);
        assert_eq!(Key::F12.to_ffi(), ffi::cui_key::CUI_KEY_F12);
        assert_eq!(Key::Num9.to_ffi(), ffi::cui_key::CUI_KEY_9);
    }

    #[test]
    fn color_tokens_round_trip() {
        // The magic token encoding from CUI_COLOR_*.
        let t = Color::ACCENT;
        assert_eq!((t.g, t.b, t.a), (0xC5, 0x1D, 0));
        assert_eq!(Color::series(7), Color::series(1));
    }

    #[test]
    fn mods_combine() {
        assert!(Mods::CTRL.contains(Mods::CTRL));
        let m = Mods::CTRL | Mods::SHIFT;
        assert!(m.contains(Mods::SHIFT));
        assert!(!m.contains(Mods::ALT));
    }
}



/// A calendar date (C's `cui_date`). `month` is 1..12.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Date {
    /// Calendar year.
    pub year: i32,
    /// Month 1..12.
    pub month: i32,
    /// Day of month 1..31.
    pub day: i32,
}

impl Date {
    pub(crate) fn to_ffi(self) -> ffi::cui_date {
        ffi::cui_date { year: self.year, month: self.month, day: self.day }
    }
    pub(crate) fn from_ffi(d: ffi::cui_date) -> Date {
        Date { year: d.year, month: d.month, day: d.day }
    }
}
