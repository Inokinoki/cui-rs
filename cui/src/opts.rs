//! Safe options structs, one per CUI `*_opts` struct, with the same fields
//! and the same "unset = default" rule: `..Default::default()` plays the role
//! of C's omitted named arguments.
//!
//! String options are `Option<&str>` (`None` = C's `NULL`). They are copied
//! into the frame arena when the widget is called, so temporaries are fine:
//!
//! ```
//! # use cui::Button;
//! let b = Button { variant: cui::ButtonVariant::Primary, icon: Some(cui::ICON_PLUS), ..Default::default() };
//! ```

use crate::arena::Arena;
use crate::types::*;
use cui_sys as ffi;
use core::ffi::c_char;
use core::ptr;

pub(crate) fn cstr(a: &Arena, s: Option<&str>) -> *const c_char {
    s.map_or(ptr::null(), |s| a.cstr(s))
}
pub(crate) fn cstr_array(a: &Arena, s: &[&str]) -> *const *const c_char {
    if s.is_empty() { ptr::null() } else { a.cstr_array(s) }
}
pub(crate) fn sz(s: Size) -> ffi::cui_size {
    size_to_ffi(s)
}
pub(crate) fn col(c: Color) -> ffi::cui_color {
    color_to_ffi(c)
}

// ---------------------------------------------------------------- containers

/// A row of children. Alias of [`BoxOpts`]; pass `&Row { .. }` to `ui.row`.
pub type Row<'a> = BoxOpts<'a>;

/// A column of children. Alias of [`BoxOpts`]; pass `&Column { .. }` to
/// `ui.column`.
pub type Column<'a> = BoxOpts<'a>;

/// Options shared by every container (C's `cui_box`): `ui.row`, `ui.column`
/// and `ui.box_` all take it.

#[derive(Clone, Copy, Debug, Default)]
pub struct BoxOpts<'a> {
    /// Identity for scroll state etc.; defaults to an automatic id.
    pub id: Option<&'a str>,
    /// Layout direction; `ui.row`/`ui.column` set it for you.
    pub dir: Dir,
    /// Width along the cross axis; default fits the content.
    pub width: Size,
    /// Height along the main axis; default fits the content.
    pub height: Size,
    /// Smallest allowed width in points.
    pub min_width: f32,
    /// Largest allowed width in points.
    pub max_width: f32,
    /// Smallest allowed height in points.
    pub min_height: f32,
    /// Largest allowed height in points.
    pub max_height: f32,
    /// Padding on all sides; `pad_x`/`pad_y` and the per-side fields override.
    pub pad: f32,
    /// Left and right padding in points; 0 uses `pad`.
    pub pad_x: f32,
    /// Top and bottom padding in points; 0 uses `pad`.
    pub pad_y: f32,
    /// Top padding in points; 0 uses `pad`.
    pub pad_top: f32,
    /// Right padding in points; 0 uses `pad`.
    pub pad_right: f32,
    /// Bottom padding in points; 0 uses `pad`.
    pub pad_bottom: f32,
    /// Left padding in points; 0 uses `pad`.
    pub pad_left: f32,
    /// Space between children.
    pub gap: f32,
    /// `Dir::Grid`: column count.
    pub columns: i32,
    /// `Dir::Grid`: as many columns as fit at this width, at least one.
    pub min_column_width: f32,
    /// `Dir::Row`: children that do not fit move to a new line.
    pub wrap: bool,
    /// Cross axis alignment of children.
    pub align: Align,
    /// Main axis distribution.
    pub justify: Justify,
    /// Background color.
    pub bg: Color,
    /// Defaults to [`Color::BORDER`] when `border > 0`.
    pub border_color: Color,
    /// Border width in points.
    pub border: f32,
    /// Corner radius in points.
    pub radius: f32,
    /// Top-left corner radius, overriding `radius`.
    pub radius_tl: f32,
    /// Top-right corner radius, overriding `radius`.
    pub radius_tr: f32,
    /// Bottom-right corner radius, overriding `radius`.
    pub radius_br: f32,
    /// Bottom-left corner radius, overriding `radius`.
    pub radius_bl: f32,
    /// Shadow depth.
    pub elevation: Elevation,
    /// Make the box a scroll area (clips content).
    pub scroll_x: bool,
    /// Make the box a vertical scroll area (clips content).
    pub scroll_y: bool,
    /// Clip children to the box.
    pub clip: bool,
    /// 0 means fully opaque.
    pub opacity: f32,
    /// Text direction, inherited by default; mirrors horizontal layout and padding.
    pub direction: Direction,
}

impl BoxOpts<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_box {
        ffi::cui_box {
            _pad: 0,
            id: cstr(a, self.id),
            dir: self.dir.to_ffi(),
            width: sz(self.width),
            height: sz(self.height),
            min_width: self.min_width,
            max_width: self.max_width,
            min_height: self.min_height,
            max_height: self.max_height,
            pad: self.pad,
            pad_x: self.pad_x,
            pad_y: self.pad_y,
            pad_top: self.pad_top,
            pad_right: self.pad_right,
            pad_bottom: self.pad_bottom,
            pad_left: self.pad_left,
            gap: self.gap,
            columns: self.columns,
            min_column_width: self.min_column_width,
            wrap: self.wrap,
            align: self.align.to_ffi(),
            justify: self.justify.to_ffi(),
            bg: col(self.bg),
            border_color: col(self.border_color),
            border: self.border,
            radius: self.radius,
            radius_tl: self.radius_tl,
            radius_tr: self.radius_tr,
            radius_br: self.radius_br,
            radius_bl: self.radius_bl,
            elevation: self.elevation.to_ffi(),
            scroll_x: self.scroll_x,
            scroll_y: self.scroll_y,
            clip: self.clip,
            opacity: self.opacity,
            direction: self.direction.to_ffi(),
        }
    }
}

/// Options for `ui.card` (C's `cui_card_opts`): a bordered surface with
/// generous padding.
#[derive(Clone, Copy, Debug, Default)]
pub struct Card<'a> {
    /// Identity of the card, for its hover state; defaults to an automatic id.
    pub id: Option<&'a str>,
    /// Width; default fits the content.
    pub width: Size,
    /// Height; default fits the content.
    pub height: Size,
    /// E.g. `Size::FILL` up to `max_width`: fills narrow screens, stops on wide ones.
    pub min_width: f32,
    /// Upper bound of the width in points, e.g. a filling card that stops growing.
    pub max_width: f32,
    /// Padding on all sides.
    pub pad: f32,
    /// Left and right padding in points; 0 uses `pad`.
    pub pad_x: f32,
    /// Top and bottom padding in points; 0 uses `pad`.
    pub pad_y: f32,
    /// Space between children.
    pub gap: f32,
    /// Layout direction, default column.
    pub dir: Dir,
    /// Cross axis alignment of children.
    pub align: Align,
    /// Row cards: children that do not fit move to a new line.
    pub wrap: bool,
    /// Lifts slightly on hover and returns clicks.
    pub hoverable: bool,
    /// Muted surface, no border.
    pub muted: bool,
    /// Accent border.
    pub selected: bool,
}

impl Card<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_card_opts {
        ffi::cui_card_opts {
            _pad: 0,
            id: cstr(a, self.id),
            width: sz(self.width),
            height: sz(self.height),
            min_width: self.min_width,
            max_width: self.max_width,
            pad: self.pad,
            pad_x: self.pad_x,
            pad_y: self.pad_y,
            gap: self.gap,
            dir: self.dir.to_ffi(),
            align: self.align.to_ffi(),
            wrap: self.wrap,
            hoverable: self.hoverable,
            muted: self.muted,
            selected: self.selected,
        }
    }
}

/// Options for `ui.scroll` (C's `cui_scroll_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Scroll<'a> {
    /// Identity of the scroll area, for its position; defaults to an automatic id.
    pub id: Option<&'a str>,
    /// Width; default fits the content.
    pub width: Size,
    /// Height; default fills the space it is given.
    pub height: Size,
    /// Also scroll horizontally.
    pub horizontal: bool,
    /// Padding around the content.
    pub pad: f32,
    /// Space between children.
    pub gap: f32,
    /// Content fades out at an edge with more to scroll.
    pub fade_edges: bool,
    /// What it fades into (default: the nearest opaque box around it).
    pub fade_color: Color,
}

impl Scroll<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_scroll_opts {
        ffi::cui_scroll_opts {
            _pad: 0,
            id: cstr(a, self.id),
            width: sz(self.width),
            height: sz(self.height),
            horizontal: self.horizontal,
            pad: self.pad,
            gap: self.gap,
            fade_edges: self.fade_edges,
            fade_color: col(self.fade_color),
        }
    }
}

// ---------------------------------------------------------------------- text

/// Options for `ui.text_with` (C's `cui_text_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Text {
    /// Font role, e.g. `font::MONO`; default `font::BODY`.
    pub font: i32,
    /// Size in points; default the theme body size.
    pub size: f32,
    /// Default the foreground color.
    pub color: Color,
    /// Alignment inside the text box.
    pub align: TextAlign,
    /// Single line, ellipsis when it does not fit.
    pub nowrap: bool,
    /// Tabular numbers.
    pub tabular: bool,
    /// Width; default fits the text.
    pub width: Size,
    /// Multiplier, default 1.45.
    pub line_height: f32,
    /// Inherited by default; `Direction::Auto` examines this text.
    pub direction: Direction,
}

impl Text {
    pub(crate) fn flatten(&self, _a: &Arena) -> ffi::cui_text_opts {
        ffi::cui_text_opts {
            _pad: 0,
            font: self.font,
            size: self.size,
            color: col(self.color),
            align: self.align.to_ffi(),
            nowrap: self.nowrap,
            tabular: self.tabular,
            selectable: false,
            width: sz(self.width),
            line_height: self.line_height,
            direction: self.direction.to_ffi(),
        }
    }
}

// ------------------------------------------------------------------- buttons

/// Options for buttons (C's `cui_button_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Button<'a> {
    /// Visual style; default secondary.
    pub variant: ButtonVariant,
    /// Height preset; default medium.
    pub size: ControlSize,
    /// Leading icon, e.g. [`crate::ICON_PLUS`].
    pub icon: Option<&'a str>,
    /// Trailing icon.
    pub icon_right: Option<&'a str>,
    /// Drawn dimmed and ignores clicks.
    pub disabled: bool,
    /// Spinner, ignores clicks.
    pub loading: bool,
    /// Stretch to the parent width.
    pub full_width: bool,
    /// Toggled look.
    pub selected: bool,
    /// Shown after a short hover.
    pub tooltip: Option<&'a str>,
    /// Shown as a hint in the tooltip, e.g. "Ctrl S".
    pub shortcut: Option<&'a str>,
    /// Width; default fits the label.
    pub width: Size,
    /// Text and icon color, e.g. [`Color::ACCENT`].
    pub color: Color,
}

impl Button<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_button_opts {
        ffi::cui_button_opts {
            _pad: 0,
            variant: self.variant.to_ffi(),
            size: self.size.to_ffi(),
            icon: cstr(a, self.icon),
            icon_right: cstr(a, self.icon_right),
            disabled: self.disabled,
            loading: self.loading,
            full_width: self.full_width,
            selected: self.selected,
            tooltip: cstr(a, self.tooltip),
            shortcut: cstr(a, self.shortcut),
            width: sz(self.width),
            color: col(self.color),
        }
    }
}

/// Options for `ui.segmented_with` (C's `cui_segmented_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Segmented<'a> {
    /// Height preset; default medium.
    pub size: ControlSize,
    /// Stretch to the parent width.
    pub full_width: bool,
    /// One icon per item.
    pub icons: &'a [&'a str],
}

impl Segmented<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_segmented_opts {
        ffi::cui_segmented_opts {
            _pad: 0,
            size: self.size.to_ffi(),
            full_width: self.full_width,
            icons: cstr_array(a, self.icons),
        }
    }
}

// ----------------------------------------------------------------- selection

/// Options for checkbox / switch / radio (C's `cui_check_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Check<'a> {
    /// Second line under the label.
    pub description: Option<&'a str>,
    /// Drawn dimmed and ignores clicks.
    pub disabled: bool,
    /// Shown as a mixed state.
    pub indeterminate: bool,
}

impl Check<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_check_opts {
        ffi::cui_check_opts {
            _pad: 0,
            description: cstr(a, self.description),
            disabled: self.disabled,
            indeterminate: self.indeterminate,
        }
    }
}

/// Options for `ui.slider_with` (C's `cui_slider_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Slider<'a> {
    /// 0 = continuous.
    pub step: f32,
    /// Show the value at the end of the row.
    pub show_value: bool,
    /// printf format for the value, default "%.0f".
    pub format: Option<&'a str>,
    /// Drawn dimmed and ignores clicks.
    pub disabled: bool,
    /// Width; default fills.
    pub width: Size,
}

impl Slider<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_slider_opts {
        ffi::cui_slider_opts {
            _pad: 0,
            step: self.step,
            show_value: self.show_value,
            format: cstr(a, self.format),
            disabled: self.disabled,
            width: sz(self.width),
        }
    }
}

/// Options for `ui.chip_with` (C's `cui_chip_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Chip<'a> {
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Drawn dimmed and ignores clicks.
    pub disabled: bool,
}

impl Chip<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_chip_opts {
        ffi::cui_chip_opts { _pad: 0, icon: cstr(a, self.icon), disabled: self.disabled }
    }
}

// --------------------------------------------------------------- text input

/// Options for text fields (C's `cui_input_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Input<'a> {
    /// Label above the field.
    pub label: Option<&'a str>,
    /// Shown when the field is empty.
    pub placeholder: Option<&'a str>,
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Helper text below.
    pub hint: Option<&'a str>,
    /// Error text below (red).
    pub error: Option<&'a str>,
    /// Shows dots instead of the text, with an eye button that reveals it.
    pub password: bool,
    /// Drawn dimmed and ignores typing.
    pub disabled: bool,
    /// x button when not empty.
    pub clearable: bool,
    /// Takes keyboard focus as soon as it appears.
    pub autofocus: bool,
    /// Selectable and copyable, not editable.
    pub read_only: bool,
    /// Height preset; default medium.
    pub size: ControlSize,
    /// Width; default fills.
    pub width: Size,
    /// Textarea: height in points; negative fills the space.
    pub height: f32,
    /// Muted text before the value, e.g. "$" or "https://".
    pub prefix: Option<&'a str>,
    /// Muted text after the value, e.g. "USD" or "kg".
    pub suffix: Option<&'a str>,
}

impl Input<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_input_opts {
        ffi::cui_input_opts {
            _pad: 0,
            label: cstr(a, self.label),
            placeholder: cstr(a, self.placeholder),
            icon: cstr(a, self.icon),
            hint: cstr(a, self.hint),
            error: cstr(a, self.error),
            password: self.password,
            disabled: self.disabled,
            clearable: self.clearable,
            autofocus: self.autofocus,
            read_only: self.read_only,
            size: self.size.to_ffi(),
            width: sz(self.width),
            height: self.height,
            prefix: cstr(a, self.prefix),
            suffix: cstr(a, self.suffix),
        }
    }
}

/// Options for `ui.number_with` (C's `cui_number_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Number<'a> {
    /// Label above the field.
    pub label: Option<&'a str>,
    /// Step of the - and + buttons; 0 uses a default.
    pub step: f64,
    /// Decimals shown when the value is not edited.
    pub decimals: i32,
    /// Drawn dimmed and ignores typing.
    pub disabled: bool,
    /// Width; default fits.
    pub width: Size,
    /// Muted text after the value, before the - and + buttons, e.g. "kg".
    pub suffix: Option<&'a str>,
}

impl Number<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_number_opts {
        ffi::cui_number_opts {
            _pad: 0,
            label: cstr(a, self.label),
            step: self.step,
            decimals: self.decimals,
            disabled: self.disabled,
            width: sz(self.width),
            suffix: cstr(a, self.suffix),
        }
    }
}

/// Options for `ui.select_with` (C's `cui_select_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Select<'a> {
    /// Label above the control.
    pub label: Option<&'a str>,
    /// Shown when no item is chosen.
    pub placeholder: Option<&'a str>,
    /// One icon per item.
    pub icons: &'a [&'a str],
    /// Combobox: filter options by typing.
    pub searchable: bool,
    /// Drawn dimmed and ignores clicks.
    pub disabled: bool,
    /// Width; default fills.
    pub width: Size,
}

impl Select<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_select_opts {
        ffi::cui_select_opts {
            _pad: 0,
            label: cstr(a, self.label),
            placeholder: cstr(a, self.placeholder),
            icons: cstr_array(a, self.icons),
            searchable: self.searchable,
            disabled: self.disabled,
            width: sz(self.width),
        }
    }
}

// ---------------------------------------------------------------- navigation

/// Options for `ui.tabs_with` (C's `cui_tabs_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Tabs<'a> {
    /// Visual style; default underline.
    pub variant: TabsVariant,
    /// One icon per item.
    pub icons: &'a [&'a str],
    /// Stretch to the parent width.
    pub full_width: bool,
}

impl Tabs<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_tabs_opts {
        ffi::cui_tabs_opts {
            _pad: 0,
            variant: self.variant.to_ffi(),
            icons: cstr_array(a, self.icons),
            full_width: self.full_width,
        }
    }
}

/// Tab bar style (C's `cui_tabs_variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum TabsVariant {
    /// A line under the active tab.
    #[default]
    Underline = 0,
    /// A pill around the active tab.
    Pill,
}

impl TabsVariant {
    pub(crate) fn to_ffi(self) -> ffi::cui_tabs_variant {
        match self {
            TabsVariant::Underline => ffi::cui_tabs_variant::CUI_TABS_UNDERLINE,
            TabsVariant::Pill => ffi::cui_tabs_variant::CUI_TABS_PILL,
        }
    }
}

/// Options for `ui.nav_item_with` (C's `cui_nav_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Nav<'a> {
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Small text at the right edge of the item, e.g. a count.
    pub badge: Option<&'a str>,
    /// The current page: medium weight text, and the group's highlight glides to it.
    pub selected: bool,
    /// Items with the same group name share one highlight; `None` puts them in one common group.
    pub group: Option<&'a str>,
}

impl Nav<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_nav_opts {
        ffi::cui_nav_opts {
            _pad: 0,
            icon: cstr(a, self.icon),
            badge: cstr(a, self.badge),
            selected: self.selected,
            group: cstr(a, self.group),
        }
    }
}

// ------------------------------------------------------------------- overlay

/// Options for `ui.dialog_with` (C's `cui_dialog_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Dialog<'a> {
    /// Second line under the title.
    pub description: Option<&'a str>,
    /// Default 440.
    pub width: f32,
    /// Hides the x in the corner (Esc and a click outside still close it unless `persistent`).
    pub no_close_button: bool,
    /// Clicking outside or Esc does not close.
    pub persistent: bool,
}

impl Dialog<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_dialog_opts {
        ffi::cui_dialog_opts {
            _pad: 0,
            description: cstr(a, self.description),
            width: self.width,
            no_close_button: self.no_close_button,
            persistent: self.persistent,
        }
    }
}

/// Options for `ui.toast_with` (C's `cui_toast_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Toast<'a> {
    /// Second line under the title.
    pub description: Option<&'a str>,
    /// Semantic color of the toast.
    pub tone: Tone,
    /// Seconds, default 4; negative = until dismissed.
    pub duration: f32,
    /// Optional action button label.
    pub action: Option<&'a str>,
    /// A draining line shows how long the action stays (undo).
    pub show_timer: bool,
}

impl Toast<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_toast_opts {
        ffi::cui_toast_opts {
            _pad: 0,
            description: cstr(a, self.description),
            tone: self.tone.to_ffi(),
            duration: self.duration,
            action: cstr(a, self.action),
            show_timer: self.show_timer,
        }
    }
}

/// Options for `ui.badge_with` (C's `cui_badge_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Badge<'a> {
    /// Semantic color of the badge.
    pub tone: Tone,
    /// A small dot before the text, in the tone's color.
    pub dot: bool,
    /// Filled with the tone's color instead of a light tint of it.
    pub solid: bool,
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Height preset; default medium.
    pub size: ControlSize,
}

impl Badge<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_badge_opts {
        ffi::cui_badge_opts {
            _pad: 0,
            tone: self.tone.to_ffi(),
            dot: self.dot,
            solid: self.solid,
            icon: cstr(a, self.icon),
            size: self.size.to_ffi(),
        }
    }
}

/// Options for `ui.empty_state_with` (C's `cui_empty_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyState<'a> {
    /// Shown above the title, e.g. `ICON_LIST_CHECKS`.
    pub icon: Option<&'a str>,
    /// Second line under the title.
    pub description: Option<&'a str>,
    /// Label of a button under the text; the call returns true when it is clicked.
    pub action: Option<&'a str>,
}

impl EmptyState<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_empty_opts {
        ffi::cui_empty_opts {
            _pad: 0,
            icon: cstr(a, self.icon),
            description: cstr(a, self.description),
            action: cstr(a, self.action),
        }
    }
}

/// Options for `ui.metric_with` (C's `cui_metric_opts`): a big number with a
/// label and an optional trend.
#[derive(Clone, Copy, Debug, Default)]
pub struct Metric<'a> {
    /// Change shown beside the value with a trend arrow, as a percentage
    /// (4.2 shows +4.2%); 0 hides it.
    pub delta: f32,
    /// Text shown instead of the percentage, e.g. "+12 this week"; `delta` still sets arrow and color.
    pub delta_label: Option<&'a str>,
    /// Values for a sparkline under the value; needs at least 2.
    pub spark: &'a [f32],
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Width; default fits.
    pub width: Size,
    /// Build times, errors, latency: a fall is good news, shown in green.
    pub lower_is_better: bool,
}

impl Metric<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_metric_opts {
        let (spark, spark_count) = if self.spark.len() >= 2 {
            (self.spark.as_ptr(), self.spark.len() as i32)
        } else {
            (ptr::null(), 0)
        };
        ffi::cui_metric_opts {
            _pad: 0,
            delta: self.delta,
            delta_label: cstr(a, self.delta_label),
            spark,
            spark_count,
            icon: cstr(a, self.icon),
            width: sz(self.width),
            lower_is_better: self.lower_is_better,
        }
    }
}

/// Options for `ui.progress_with` (C's `cui_progress_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Progress<'a> {
    /// No value: a segment travels the track.
    pub indeterminate: bool,
    /// Fill color; default accent.
    pub tone: Tone,
    /// Points, default 6.
    pub height: f32,
    /// A row above the bar with this name.
    pub label: Option<&'a str>,
    /// A whole percentage at the end of that row.
    pub show_value: bool,
    /// Width; default fills.
    pub width: Size,
}

impl Progress<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_progress_opts {
        ffi::cui_progress_opts {
            _pad: 0,
            indeterminate: self.indeterminate,
            tone: self.tone.to_ffi(),
            height: self.height,
            label: cstr(a, self.label),
            show_value: self.show_value,
            width: sz(self.width),
            dither: false,
        }
    }
}

// -------------------------------------------------------------- table, chart

/// One column of a [`Ui::table_begin`](crate::Ui::table_begin).
#[derive(Clone, Copy, Debug, Default)]
pub struct TableColumn<'a> {
    /// Text of the column's header.
    pub title: &'a str,
    /// Default fill.
    pub width: Size,
    /// Text alignment in the column.
    pub align: TextAlign,
    /// Header is clickable; needs `Table::sort`.
    pub sortable: bool,
}

impl TableColumn<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_table_column {
        ffi::cui_table_column {
            title: a.cstr(self.title),
            width: sz(self.width),
            align: self.align.to_ffi(),
            sortable: self.sortable,
        }
    }
}

/// Sort state of a table, owned by the app and passed by reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableSort {
    /// The column's index in the `columns` slice passed to `ui.table_begin`.
    pub column: i32,
    /// Sort largest first.
    pub descending: bool,
}

/// Options for `ui.table_begin` (C's `cui_table_opts`).
#[derive(Debug, Default)]
pub struct Table<'a> {
    /// Updated when a sortable header is clicked.
    pub sort: Option<&'a mut TableSort>,
    /// Scrolls when set.
    pub height: Size,
    /// Alternating row backgrounds.
    pub striped: bool,
    /// Rows at least 36 points tall instead of 44.
    pub compact: bool,
    /// Rows take keyboard focus and show the hand cursor.
    pub selectable: bool,
}

impl Table<'_> {
    pub(crate) fn flatten(&mut self, _a: &Arena) -> ffi::cui_table_opts {
        ffi::cui_table_opts {
            _pad: 0,
            sort: self.sort.as_deref_mut().map_or(ptr::null_mut(), |s| s as *mut TableSort as *mut ffi::cui_table_sort),
            height: sz(self.height),
            striped: self.striped,
            compact: self.compact,
            selectable: self.selectable,
        }
    }
}

/// One data series of a chart (C's `cui_series`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Series<'a> {
    /// Shown in the legend and in the hover tooltip.
    pub name: Option<&'a str>,
    /// The series' data, one value per point.
    pub values: &'a [f32],
    /// Default: the series palette.
    pub color: Color,
}

impl Series<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_series {
        ffi::cui_series {
            name: cstr(a, self.name),
            values: if self.values.is_empty() { ptr::null() } else { self.values.as_ptr() },
            count: self.values.len() as i32,
            color: col(self.color),
        }
    }
}

/// A line, a threshold or a point drawn over a chart (C's `cui_chart_annotation`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Annotation<'a> {
    /// A vertical line, a horizontal line or a point.
    pub kind: AnnotationKind,
    /// In the chart's x units: the point's index for line and bar charts.
    pub x: f64,
    /// A value, in the chart's value units.
    pub y: f64,
    /// First line, e.g. "PEAK 203 MG"; `None` for a line with no label.
    pub label: Option<&'a str>,
    /// Second line, muted, e.g. "13:06".
    pub detail: Option<&'a str>,
    /// The line, dot and label in the accent color.
    pub accent: bool,
}

/// Kind of a chart annotation (C's `cui_annotation_kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum AnnotationKind {
    /// A vertical line at `x`, labelled at its top.
    #[default]
    X = 0,
    /// A dashed horizontal line at the value `y`, labelled at its end.
    Y,
    /// A dot at `x`, `y`, labelled beside it.
    Point,
}

impl AnnotationKind {
    fn to_ffi(self) -> ffi::cui_annotation_kind {
        match self {
            AnnotationKind::X => ffi::cui_annotation_kind::CUI_ANNOTATION_X,
            AnnotationKind::Y => ffi::cui_annotation_kind::CUI_ANNOTATION_Y,
            AnnotationKind::Point => ffi::cui_annotation_kind::CUI_ANNOTATION_POINT,
        }
    }
}

impl Annotation<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_chart_annotation {
        ffi::cui_chart_annotation {
            kind: self.kind.to_ffi(),
            x: self.x,
            y: self.y,
            label: cstr(a, self.label),
            detail: cstr(a, self.detail),
            accent: self.accent,
        }
    }
}

/// Options for the chart widgets (C's `cui_chart_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Chart<'a> {
    /// Width; default fills.
    pub width: Size,
    /// Default 220.
    pub height: f32,
    /// X axis labels; 0 labels means one per data point.
    pub labels: &'a [&'a str],
    /// Fill under lines.
    pub area: bool,
    /// Leave out the horizontal grid lines.
    pub no_grid: bool,
    /// Leave out the legend under the chart.
    pub no_legend: bool,
    /// Bars stack in one column; lines stack on the series before them.
    pub stacked: bool,
    /// Curved lines.
    pub smooth: bool,
    /// Value format for axis and tooltip, default "%.0f".
    pub format: Option<&'a str>,
    /// Fixed y range when min != max.
    pub min: f32,
    /// Fixed y range when min != max.
    pub max: f32,
    /// Lines and points drawn over the chart (line and bar charts).
    pub annotations: &'a [Annotation<'a>],
}

impl Chart<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_chart_opts {
        let (annotations, annotation_count) = if self.annotations.is_empty() {
            (ptr::null(), 0)
        } else {
            (a.struct_array(&self.annotations.iter().map(|x| x.flatten(a)).collect::<Vec<_>>()), self.annotations.len() as i32)
        };
        ffi::cui_chart_opts {
            _pad: 0,
            width: sz(self.width),
            height: self.height,
            labels: cstr_array(a, self.labels),
            label_count: self.labels.len() as i32,
            area: self.area,
            no_grid: self.no_grid,
            no_legend: self.no_legend,
            stacked: self.stacked,
            smooth: self.smooth,
            format: cstr(a, self.format),
            min: self.min,
            max: self.max,
            annotations,
            annotation_count,
            dither: false,
        }
    }
}

/// Options for [`crate::Ui::sheet`] (C's `cui_sheet_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Sheet<'a> {
    /// Which side the sheet slides in from; default right.
    pub side: Side,
    /// The sheet's size in points along its slide axis.
    pub size: f32,
    /// A muted line under the title.
    pub description: Option<&'a str>,
}

impl Sheet<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_sheet_opts {
        ffi::cui_sheet_opts {
            _pad: 0,
            side: self.side.to_ffi(),
            size: self.size,
            description: cstr(a, self.description),
        }
    }
}
