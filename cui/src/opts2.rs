//! Second batch of options structs — menus, splits, overlays, conversation,
//! instruments and the canvas — mirroring C's structs one-to-one like
//! [`crate::opts`].

use crate::arena::Arena;
use crate::opts::{cstr, cstr_array, col, sz, Annotation};
use crate::types::*;
use cui_sys as ffi;

ffi_enum! {
    /// Where an overlay opens relative to its anchor (C's `cui_placement`).
    pub enum Placement {
        BottomStart = 0,
        Bottom,
        BottomEnd,
        TopStart,
        Top,
        TopEnd,
        RightStart,
        LeftStart,
        Right,
        Left,
    }
    as ffi::cui_placement
}

ffi_enum! {
    /// The x axis of an instrument chart (C's `cui_axis_units`): `Number`
    /// prints numbers with the axis format; `Hours` shows a time of day
    /// (12.25 is 12:15, 30 is 06:00 the next day).
    pub enum Axis { Number = 0, Hours }
    as ffi::cui_axis_units
}

ffi_enum! {
    /// The state of an agent's task (C's `cui_task_status`).
    pub enum TaskStatus {
        Pending = 0,
        Running,
        Done,
        Failed,
        Skipped,
    }
    as ffi::cui_task_status
}

ffi_enum! {
    /// Which kind of native file dialog to open (C's `cui_file_dialog_kind`).
    pub enum FileDialogKind {
        Open = 0,
        Save,
        Folder,
    }
    as ffi::cui_file_dialog_kind
}

/// A picture handle, as returned by [`crate::Ui::image_create`]-style APIs.
/// 0 means "none".
pub type Texture = usize;

/// Corner radii of a drawn rectangle, in points (C's `cui_corners`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Corners {
    /// Top left.
    pub tl: f32,
    /// Top right.
    pub tr: f32,
    /// Bottom right.
    pub br: f32,
    /// Bottom left.
    pub bl: f32,
}

impl Corners {
    /// The same radius on every corner (C's `CUI_CORNERS(r)`).
    pub const fn all(r: f32) -> Corners {
        Corners { tl: r, tr: r, br: r, bl: r }
    }
}

impl Corners {
    pub(crate) fn to_ffi(self) -> ffi::cui_corners {
        ffi::cui_corners { tl: self.tl, tr: self.tr, br: self.br, bl: self.bl }
    }
}

// --------------------------------------------------------------------- menus

/// Options for [`Ui::menu_with`](crate::Ui::menu_with) (C's `cui_menu_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Menu<'a> {
    /// Leading icon, e.g. `ICON_ELLIPSIS`.
    pub icon: Option<&'a str>,
    /// Button style of the trigger; default ghost.
    pub variant: ButtonVariant,
    /// Where the menu opens; default bottom start.
    pub placement: Placement,
    /// Menu width in points; default fits the items.
    pub width: f32,
    /// The trigger is an icon button (the ellipsis unless `icon` is set) with the label as its tooltip.
    pub icon_only: bool,
}

impl Menu<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_menu_opts {
        ffi::cui_menu_opts {
            _pad: 0,
            icon: cstr(a, self.icon),
            variant: self.variant.to_ffi(),
            placement: self.placement.to_ffi(),
            width: self.width,
            icon_only: self.icon_only,
        }
    }
}

/// Options for [`Ui::menu_item_with`](crate::Ui::menu_item_with) (C's `cui_menu_item_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuItem<'a> {
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Shortcut hint shown at the right edge, e.g. "Ctrl N".
    pub shortcut: Option<&'a str>,
    /// Drawn in the danger color.
    pub danger: bool,
    /// Drawn dimmed and ignores clicks.
    pub disabled: bool,
    /// A check mark at the right edge of the item.
    pub checked: bool,
}

impl MenuItem<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_menu_item_opts {
        ffi::cui_menu_item_opts {
            _pad: 0,
            icon: cstr(a, self.icon),
            shortcut: cstr(a, self.shortcut),
            danger: self.danger,
            disabled: self.disabled,
            checked: self.checked,
        }
    }
}

/// Options for [`Ui::split`](crate::Ui::split) (C's `cui_split_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Split {
    /// Split top-to-bottom instead of left-to-right.
    pub vertical: bool,
    /// Smallest first pane in points.
    pub min_first: f32,
    /// Smallest second pane in points.
    pub min_second: f32,
    /// Width; default fills.
    pub width: Size,
    /// Height; default fills.
    pub height: Size,
}

impl Split {
    pub(crate) fn flatten(&self) -> ffi::cui_split_opts {
        ffi::cui_split_opts {
            _pad: 0,
            vertical: self.vertical,
            min_first: self.min_first,
            min_second: self.min_second,
            width: sz(self.width),
            height: sz(self.height),
        }
    }
}

/// Options for [`Ui::avatar_with`](crate::Ui::avatar_with) (C's `cui_avatar_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Avatar<'a> {
    /// Diameter in points; default 32.
    pub size: f32,
    /// Status dot on the avatar: "online" green, "busy" red, "away" amber, anything else gray.
    pub status: Option<&'a str>,
    /// Rounded square instead of a circle.
    pub square: bool,
}

impl Avatar<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_avatar_opts {
        ffi::cui_avatar_opts {
            _pad: 0,
            size: self.size,
            image: 0,
            status: cstr(a, self.status),
            square: self.square,
        }
    }
}

/// Options for [`Ui::user_menu_with`](crate::Ui::user_menu_with) (C's `cui_user_menu_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct UserMenu<'a> {
    /// Second line under the name, e.g. an email address; also in the menu's header.
    pub detail: Option<&'a str>,
    /// Status dot on the avatar: "online", "busy", "away" or anything else (gray).
    pub status: Option<&'a str>,
    /// Only the avatar as the button; the menu then opens below it.
    pub compact: bool,
    /// Where the menu opens.
    pub placement: Placement,
    /// Menu width in points.
    pub width: f32,
}

impl UserMenu<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_user_menu_opts {
        ffi::cui_user_menu_opts {
            _pad: 0,
            detail: cstr(a, self.detail),
            status: cstr(a, self.status),
            avatar: 0,
            compact: self.compact,
            placement: self.placement.to_ffi(),
            width: self.width,
        }
    }
}

/// Options for [`Ui::timeline_item_with`](crate::Ui::timeline_item_with)
/// (C's `cui_timeline_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct TimelineItem<'a> {
    /// Muted text at the right of the title, e.g. "2h ago".
    pub time: Option<&'a str>,
    /// Smaller muted text under the title.
    pub description: Option<&'a str>,
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// The dot and connector color.
    pub tone: Tone,
    /// The final item: no connector line below its dot and no space after it.
    pub last: bool,
}

impl TimelineItem<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_timeline_opts {
        ffi::cui_timeline_opts {
            _pad: 0,
            time: cstr(a, self.time),
            description: cstr(a, self.description),
            icon: cstr(a, self.icon),
            tone: self.tone.to_ffi(),
            last: self.last,
        }
    }
}

// ----------------------------------------------------------------- text, chat

/// Motion options shared by [`Ui::typewriter_with`] and [`Ui::number_roll_with`]
/// (C's `cui_text_fx_opts`). All respect reduced motion.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextFx {
    /// Font role; default `font::BODY`.
    pub font: i32,
    /// Size in points; default the body size.
    pub size: f32,
    /// Default the foreground color.
    pub color: Color,
    /// Width; default fits the text.
    pub width: Size,
    /// Single line, ellipsis when it does not fit.
    pub nowrap: bool,
    /// Typewriter: characters per second.
    pub speed: f32,
    /// Seconds before it starts.
    pub delay: f32,
    /// Typewriter: a blinking caret while typing.
    pub caret: bool,
}

impl TextFx {
    pub(crate) fn flatten(&self, _a: &Arena) -> ffi::cui_text_fx_opts {
        ffi::cui_text_fx_opts {
            _pad: 0,
            font: self.font,
            size: self.size,
            color: col(self.color),
            width: sz(self.width),
            nowrap: self.nowrap,
            speed: self.speed,
            delay: self.delay,
            caret: self.caret,
        }
    }
}

/// Options for [`Ui::chat_message_with`](crate::Ui::chat_message_with)
/// (C's `cui_chat_opts`).
#[derive(Clone, Debug, Default)]
pub struct ChatMessage<'a> {
    /// Right aligned, accent bubble.
    pub mine: bool,
    /// Name above the first bubble of a group.
    pub author: Option<&'a str>,
    /// Shown after the author's name; on your own messages after `status`.
    pub time: Option<&'a str>,
    /// Under your own last message, e.g. "Read".
    pub status: Option<&'a str>,
    /// Continues the previous message.
    pub grouped: bool,
    /// Shows the avatar (others) and status (mine).
    pub last_in_group: bool,
}

impl ChatMessage<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_chat_opts {
        ffi::cui_chat_opts {
            _pad: 0,
            mine: self.mine,
            author: cstr(a, self.author),
            time: cstr(a, self.time),
            status: cstr(a, self.status),
            grouped: self.grouped,
            last_in_group: self.last_in_group,
            avatar: 0,
            markdown: false,
            markdown_result: core::ptr::null_mut(),
        }
    }
}

// --------------------------------------------------------------- conversation

/// Options for [`Ui::reasoning_with`](crate::Ui::reasoning_with) (C's `cui_reasoning_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Reasoning<'a> {
    /// Still thinking: the header shimmers and the body opens; it folds when this turns false.
    pub thinking: bool,
    /// Time spent, for the header: "Thinking for 12s", then "Thought for 12s".
    pub seconds: f32,
    /// Header text to show instead of the "Thinking"/"Thought for" lines.
    pub title: Option<&'a str>,
}

impl Reasoning<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_reasoning_opts {
        ffi::cui_reasoning_opts {
            _pad: 0,
            thinking: self.thinking,
            seconds: self.seconds,
            title: cstr(a, self.title),
        }
    }
}

/// Options for [`Ui::tool_call_with`](crate::Ui::tool_call_with) (C's `cui_tool_call_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ToolCall<'a> {
    /// The call's state.
    pub status: TaskStatus,
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Muted text after the tool's name, e.g. the file or query it works on.
    pub detail: Option<&'a str>,
    /// What was sent to the tool; the row opens to show it (as JSON when it parses).
    pub input: Option<&'a str>,
    /// What the tool returned, shown when open; labelled "Error" when failed.
    pub output: Option<&'a str>,
    /// How long the call took; 0 hides it.
    pub seconds: f32,
}

impl ToolCall<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_tool_call_opts {
        ffi::cui_tool_call_opts {
            _pad: 0,
            status: self.status.to_ffi(),
            icon: cstr(a, self.icon),
            detail: cstr(a, self.detail),
            input: cstr(a, self.input),
            output: cstr(a, self.output),
            seconds: self.seconds,
        }
    }
}

/// One step of an [`Ui::agent_plan`] (C's `cui_plan_step`).
#[derive(Clone, Copy, Debug)]
pub struct PlanStep<'a> {
    /// The step's text; shimmers while the step is running.
    pub title: &'a str,
    /// Smaller muted line under the title (red when the step failed).
    pub detail: Option<&'a str>,
    /// The step's state.
    pub status: TaskStatus,
}

impl PlanStep<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_plan_step {
        ffi::cui_plan_step {
            title: a.cstr(self.title),
            detail: cstr(a, self.detail),
            status: self.status.to_ffi(),
        }
    }
}

/// Options for [`Ui::agent_plan_with`](crate::Ui::agent_plan_with) (C's `cui_agent_plan_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct AgentPlan<'a> {
    /// Heading of the plan; default "Plan".
    pub title: Option<&'a str>,
}

/// Options for [`Ui::agent_status_with`](crate::Ui::agent_status_with) (C's `cui_agent_status_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct AgentStatus<'a> {
    /// What the agent is doing.
    pub status: TaskStatus,
    /// Elapsed time shown after the text, as 42s or 1m 05s; 0 hides it.
    pub seconds: f32,
    /// Leading icon.
    pub icon: Option<&'a str>,
}

impl AgentStatus<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_agent_status_opts {
        ffi::cui_agent_status_opts {
            _pad: 0,
            status: self.status.to_ffi(),
            seconds: self.seconds,
            icon: cstr(a, self.icon),
        }
    }
}

/// A document an answer used (C's `cui_source`).
#[derive(Clone, Copy, Debug)]
pub struct Source<'a> {
    /// The card's heading; `None` shows the URL's domain.
    pub title: Option<&'a str>,
    /// The address: its domain is shown, and the whole URL as the card's tooltip.
    pub url: &'a str,
    /// An excerpt, shown in the hover card of a citation.
    pub snippet: Option<&'a str>,
}

impl Source<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_source {
        ffi::cui_source {
            title: cstr(a, self.title),
            url: a.cstr(self.url),
            snippet: cstr(a, self.snippet),
        }
    }
}

/// What happened under an answer this frame (C's `CUI_RESPONSE_*` bits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResponseActionsFlags(i32);

impl ResponseActionsFlags {
    pub(crate) const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }
    /// The copy button was clicked.
    pub const fn copied(self) -> bool {
        self.0 & ffi::CUI_RESPONSE_COPY != 0
    }
    /// Regenerate was clicked.
    pub const fn regenerated(self) -> bool {
        self.0 & ffi::CUI_RESPONSE_REGENERATE != 0
    }
    /// Thumbs up.
    pub const fn voted_up(self) -> bool {
        self.0 & ffi::CUI_RESPONSE_UP != 0
    }
    /// Thumbs down.
    pub const fn voted_down(self) -> bool {
        self.0 & ffi::CUI_RESPONSE_DOWN != 0
    }
    /// Share was clicked.
    pub const fn shared(self) -> bool {
        self.0 & ffi::CUI_RESPONSE_SHARE != 0
    }
}

/// Options for [`Ui::response_actions_with`](crate::Ui::response_actions_with)
/// (C's `cui_response_actions_opts`).
#[derive(Debug, Default)]
pub struct ResponseActions<'a> {
    /// The text the copy button copies; `None` leaves the button out.
    pub copy_text: Option<&'a str>,
    /// 1 thumbs up, -1 thumbs down, 0 none: shown on the buttons and updated on a click.
    pub vote: Option<&'a mut i32>,
    /// Hides the regenerate button.
    pub no_regenerate: bool,
    /// Hides the share button.
    pub no_share: bool,
}

impl ResponseActions<'_> {
    pub(crate) fn flatten(&mut self, a: &Arena) -> ffi::cui_response_actions_opts {
        ffi::cui_response_actions_opts {
            _pad: 0,
            copy_text: cstr(a, self.copy_text),
            vote: self.vote.as_deref_mut().map_or(core::ptr::null_mut(), |v| v as *mut i32 as *mut core::ffi::c_int),
            no_regenerate: self.no_regenerate,
            no_share: self.no_share,
        }
    }
}

/// One suggestion card (C's `cui_prompt_suggestion`).
#[derive(Clone, Copy, Debug)]
pub struct PromptSuggestion<'a> {
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// The card's heading, in medium weight.
    pub title: &'a str,
    /// Smaller muted text under the title.
    pub detail: Option<&'a str>,
}

impl PromptSuggestion<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_prompt_suggestion {
        ffi::cui_prompt_suggestion {
            icon: cstr(a, self.icon),
            title: a.cstr(self.title),
            detail: cstr(a, self.detail),
        }
    }
}

/// Options for [`Ui::prompt_suggestions_with`](crate::Ui::prompt_suggestions_with)
/// (C's `cui_prompt_suggestions_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct PromptSuggestions {
    /// Cards per row; default 2.
    pub columns: i32,
}

impl PromptSuggestions {
    pub(crate) fn flatten(&self) -> ffi::cui_prompt_suggestions_opts {
        ffi::cui_prompt_suggestions_opts { _pad: 0, columns: self.columns }
    }
}

/// Options for [`Ui::list_item_with`](crate::Ui::list_item_with) (C's `cui_list_item_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ListItem<'a> {
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Second line under the title, smaller; makes the row taller.
    pub subtitle: Option<&'a str>,
    /// Muted text at the right edge, e.g. a time or a count.
    pub trailing: Option<&'a str>,
    /// Drawn with the active background.
    pub selected: bool,
    /// A name: shows that person's avatar (initials) in place of the icon.
    pub avatar: Option<&'a str>,
}
impl ListItem<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_list_item_opts {
        ffi::cui_list_item_opts {
            _pad: 0,
            icon: cstr(a, self.icon),
            subtitle: cstr(a, self.subtitle),
            trailing: cstr(a, self.trailing),
            selected: self.selected,
            avatar: cstr(a, self.avatar),
        }
    }
}

/// One model in a [`Ui::model_picker`] (C's `cui_model`).
#[derive(Clone, Copy, Debug)]
pub struct Model<'a> {
    /// Shown on the button and as the row's heading in the list.
    pub name: &'a str,
    /// Smaller muted line under the name in the list, e.g. what the model is good at.
    pub detail: Option<&'a str>,
    /// Small accent badge after the name, e.g. "New".
    pub badge: Option<&'a str>,
    /// Leading icon.
    pub icon: Option<&'a str>,
}

impl Model<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_model {
        ffi::cui_model {
            name: a.cstr(self.name),
            detail: cstr(a, self.detail),
            badge: cstr(a, self.badge),
            icon: cstr(a, self.icon),
        }
    }
}

/// Options for [`Ui::model_picker_with`](crate::Ui::model_picker_with) (C's `cui_model_picker_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelPicker {
    /// No border or fill on the button until hovered.
    pub ghost: bool,
    /// Where the list opens.
    pub placement: Placement,
}

impl ModelPicker {
    pub(crate) fn flatten(&self) -> ffi::cui_model_picker_opts {
        ffi::cui_model_picker_opts { _pad: 0, ghost: self.ghost, placement: self.placement.to_ffi() }
    }
}

/// What happened in the composer this frame (C's `CUI_COMPOSER_*` bits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComposerFlags(i32);

impl ComposerFlags {
    pub(crate) const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }
    /// The message was sent (`value` now holds it).
    pub const fn sent(self) -> bool {
        self.0 & ffi::CUI_COMPOSER_SEND != 0
    }
    /// Stop was clicked while busy.
    pub const fn stopped(self) -> bool {
        self.0 & ffi::CUI_COMPOSER_STOP != 0
    }
    /// The attach button was clicked (drop files on the window instead).
    pub const fn attach(self) -> bool {
        self.0 & ffi::CUI_COMPOSER_ATTACH != 0
    }
    /// An attachment's x was clicked (`removed` holds its index).
    pub const fn removed(self) -> bool {
        self.0 & ffi::CUI_COMPOSER_REMOVE != 0
    }
    /// The model was changed.
    pub const fn model_changed(self) -> bool {
        self.0 & ffi::CUI_COMPOSER_MODEL != 0
    }
}

/// Options for [`Ui::chat_composer_with`](crate::Ui::chat_composer_with)
/// (C's `cui_composer_opts`).
#[derive(Debug, Default)]
pub struct Composer<'a> {
    /// Shown when the text is empty.
    pub placeholder: Option<&'a str>,
    /// A reply is being generated: the send button becomes stop and Enter does not send.
    pub busy: bool,
    /// Names of the attached files, shown as chips above the text, each with an x.
    pub attachments: &'a [&'a str],
    /// Set to the index of the attachment whose x was clicked (with [`ComposerFlags::removed`]).
    pub removed: Option<&'a mut i32>,
    /// Models for a picker beside the attach button.
    pub models: &'a [Model<'a>],
    /// Index of the chosen model, read and written.
    pub model: Option<&'a mut i32>,
    /// Maximum lines before the text area scrolls; default 8.
    pub max_lines: i32,
    /// Width; default fills.
    pub width: Size,
}

impl Composer<'_> {
    pub(crate) fn flatten(&mut self, a: &Arena) -> ffi::cui_composer_opts {
        let models: Vec<ffi::cui_model> = self.models.iter().map(|m| m.flatten(a)).collect();
        let models = if models.is_empty() { core::ptr::null() } else { a.struct_array(&models) };
        ffi::cui_composer_opts {
            _pad: 0,
            placeholder: cstr(a, self.placeholder),
            busy: self.busy,
            attachments: cstr_array(a, self.attachments),
            attachment_count: self.attachments.len() as i32,
            removed: self.removed.as_deref_mut().map_or(core::ptr::null_mut(), |r| r as *mut i32 as *mut core::ffi::c_int),
            models,
            model_count: self.models.len() as i32,
            model: self.model.as_deref_mut().map_or(core::ptr::null_mut(), |m| m as *mut i32 as *mut core::ffi::c_int),
            max_lines: self.max_lines,
            width: sz(self.width),
        }
    }
}

/// One slice of a context meter (C's `cui_viz_item`).
#[derive(Clone, Copy, Debug)]
pub struct VizItem<'a> {
    /// The item's name.
    pub label: &'a str,
    /// The item's amount: its area, length or share.
    pub value: f32,
    /// Unset: the next series color.
    pub color: Color,
}

impl VizItem<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_viz_item {
        ffi::cui_viz_item { label: a.cstr(self.label), value: self.value, color: col(self.color) }
    }
}

/// Options for [`Ui::context_meter_with`](crate::Ui::context_meter_with) (C's `cui_context_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct ContextMeter<'a> {
    /// What fills the context (label, amount, color), for the breakdown on hover.
    pub parts: &'a [VizItem<'a>],
    /// After values in the breakdown, e.g. "tokens".
    pub unit: Option<&'a str>,
}

impl ContextMeter<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_context_opts {
        let (parts, count) = if self.parts.is_empty() {
            (core::ptr::null(), 0)
        } else {
            let flat: Vec<ffi::cui_viz_item> = self.parts.iter().map(|p| p.flatten(a)).collect();
            (a.struct_array(&flat), self.parts.len() as i32)
        };
        ffi::cui_context_opts { _pad: 0, parts, part_count: count, unit: cstr(a, self.unit) }
    }
}

// ---------------------------------------------------------------- instruments

/// One series of a [`Ui::dot_chart`] (C's `cui_dot_series`).
#[derive(Clone, Copy, Debug)]
pub struct DotSeries<'a> {
    /// One value per sample; negative values count as 0.
    pub values: &'a [f32],
    /// Names the series in the readout when there are several.
    pub name: Option<&'a str>,
    /// Drawn in the accent color: the item being edited, or the part that matters.
    pub accent: bool,
}

impl DotSeries<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_dot_series {
        ffi::cui_dot_series {
            values: if self.values.is_empty() { core::ptr::null() } else { self.values.as_ptr() },
            name: cstr(a, self.name),
            accent: self.accent,
        }
    }
}

/// Options for [`Ui::dot_chart_with`](crate::Ui::dot_chart_with) (C's `cui_dot_chart_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct DotChart<'a> {
    /// Width; default fills.
    pub width: Size,
    /// The plot and its labels; default 200.
    pub height: f32,
    /// What the first and last sample stand for; equal: 0 to count - 1.
    pub x_min: f64,
    /// What the last sample stands for.
    pub x_max: f64,
    /// How x labels are shown.
    pub x_units: Axis,
    /// X labels with [`Axis::Number`]; default "%g".
    pub x_format: Option<&'a str>,
    /// Spacing of the x labels in x units; 0 picks one that fits.
    pub x_step: f64,
    /// Value range when min != max; otherwise 0 to the largest stacked value.
    pub min: f32,
    /// Value range when min != max.
    pub max: f32,
    /// Values in the readout; default "%.0f".
    pub format: Option<&'a str>,
    /// After values in the readout, e.g. "MG".
    pub unit: Option<&'a str>,
    /// Distance between dot centers in points; default 6.
    pub pitch: f32,
    /// Hides the dim dots of the empty grid.
    pub no_grid: bool,
    /// Hides the x labels, e.g. when an event track under the chart has them.
    pub no_axis: bool,
    /// No value label under the pointer.
    pub no_readout: bool,
    /// Lines and points drawn over the dots.
    pub annotations: &'a [Annotation<'a>],
}

impl DotChart<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_dot_chart_opts {
        let (annotations, annotation_count) = if self.annotations.is_empty() {
            (core::ptr::null(), 0)
        } else {
            let flat: Vec<ffi::cui_chart_annotation> =
                self.annotations.iter().map(|x| x.flatten(a)).collect();
            (a.struct_array(&flat), self.annotations.len() as i32)
        };
        ffi::cui_dot_chart_opts {
            _pad: 0,
            width: sz(self.width),
            height: self.height,
            x_min: self.x_min,
            x_max: self.x_max,
            x_units: self.x_units.to_ffi(),
            x_format: cstr(a, self.x_format),
            x_step: self.x_step,
            min: self.min,
            max: self.max,
            format: cstr(a, self.format),
            unit: cstr(a, self.unit),
            pitch: self.pitch,
            no_grid: self.no_grid,
            no_axis: self.no_axis,
            no_readout: self.no_readout,
            annotations,
            annotation_count,
        }
    }
}

/// What a [`Ui::dot_chart`] reports this frame (C's `cui_dot_chart_result`).
#[derive(Clone, Copy, Debug, Default)]
pub struct DotChartResult {
    /// The sample under the pointer or the keyboard cursor, or -1.
    pub hovered: i32,
    /// That sample's x, in x units.
    pub x: f64,
    /// The stacked value there.
    pub value: f32,
    /// The plot was clicked, or Enter was pressed on it, this frame.
    pub clicked: bool,
}

/// Options for [`Ui::event_track_with`](crate::Ui::event_track_with) (C's `cui_event_track_opts`).
#[derive(Debug, Default)]
pub struct EventTrack<'a> {
    /// Width; default fills.
    pub width: Size,
    /// Events snap to multiples of this from `min`; 0 does not snap.
    pub step: f64,
    /// How labels are shown.
    pub units: Axis,
    /// Labels with [`Axis::Number`]; default "%g".
    pub format: Option<&'a str>,
    /// Spacing of the axis labels; 0 picks one that fits.
    pub label_step: f64,
    /// The selected event or -1, kept by you; `None` keeps it in the track.
    pub selected: Option<&'a mut i32>,
    /// Clicks and Enter do not add events.
    pub no_add: bool,
    /// Double-clicks and Delete do not remove events.
    pub no_remove: bool,
    /// Hides the axis labels under the dots.
    pub no_axis: bool,
}

/// What an [`Ui::event_track`] did this frame (C's `cui_event_track_result`).
#[derive(Clone, Copy, Debug, Default)]
pub struct EventTrackResult {
    /// An event was moved, added or removed this frame.
    pub changed: bool,
    /// Index of the event added this frame, or -1.
    pub added: i32,
    /// Index the removed event had, or -1.
    pub removed: i32,
    /// The selected event after this frame, or -1.
    pub selected: i32,
    /// An event is being dragged.
    pub dragging: bool,
}

/// Options for [`Ui::readout_with`](crate::Ui::readout_with) (C's `cui_readout_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Readout<'a> {
    /// After the value, smaller and muted, e.g. "MG".
    pub unit: Option<&'a str>,
    /// A muted line under the value.
    pub detail: Option<&'a str>,
    /// The value's size: Sm 17, Md 28 (default), Lg 44 points.
    pub size: ControlSize,
    /// The value color; default foreground.
    pub tone: Tone,
    /// The value in the accent color.
    pub accent: bool,
    /// Alignment inside the readout; default start.
    pub align: TextAlign,
    /// Default fits the value.
    pub width: Size,
}

impl Readout<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_readout_opts {
        ffi::cui_readout_opts {
            _pad: 0,
            unit: cstr(a, self.unit),
            detail: cstr(a, self.detail),
            size: self.size.to_ffi(),
            tone: self.tone.to_ffi(),
            accent: self.accent,
            align: self.align.to_ffi(),
            width: sz(self.width),
        }
    }
}

/// Options for [`Ui::dot_display_with`](crate::Ui::dot_display_with) (C's `cui_dot_display_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct DotDisplay {
    /// Distance between dot centers in points; default 4.
    pub pitch: f32,
    /// Lit dots; unset: the foreground.
    pub color: Color,
    /// Lit dots in the accent color.
    pub accent: bool,
    /// Hides the unlit dots.
    pub no_dim: bool,
    /// Scrolls text that is wider than the box instead of shrinking it.
    pub marquee: bool,
    /// Where the text sits in a wider box, default start.
    pub align: TextAlign,
    /// Default fit to the text.
    pub width: Size,
}

impl DotDisplay {
    pub(crate) fn flatten(&self, _a: &Arena) -> ffi::cui_dot_display_opts {
        ffi::cui_dot_display_opts {
            _pad: 0,
            pitch: self.pitch,
            color: col(self.color),
            accent: self.accent,
            no_dim: self.no_dim,
            marquee: self.marquee,
            align: self.align.to_ffi(),
            width: sz(self.width),
        }
    }
}

/// One command of a [`Ui::command_palette`] (C's `cui_command`).
#[derive(Clone, Copy, Debug)]
pub struct Command<'a> {
    /// The command's text in the list; what typing in the palette searches.
    pub title: &'a str,
    /// Leading icon.
    pub icon: Option<&'a str>,
    /// Heading above a run of commands with the same group, shown while the search is empty.
    pub group: Option<&'a str>,
    /// Shortcut hint shown at the right edge.
    pub shortcut: Option<&'a str>,
}

impl Command<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_command {
        ffi::cui_command {
            title: a.cstr(self.title),
            icon: cstr(a, self.icon),
            group: cstr(a, self.group),
            shortcut: cstr(a, self.shortcut),
        }
    }
}

/// Options for [`Ui::file_dialog`](crate::App::file_dialog) (C's `cui_file_dialog_opts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct FileDialog<'a> {
    /// Which dialog to open.
    pub kind: FileDialogKind,
    /// Dialog title.
    pub title: Option<&'a str>,
    /// e.g. "Images".
    pub filter_name: Option<&'a str>,
    /// Extensions separated by ';', e.g. "png;jpg".
    pub filter: Option<&'a str>,
    /// Save dialogs.
    pub default_name: Option<&'a str>,
}

impl FileDialog<'_> {
    pub(crate) fn flatten(&self, a: &Arena) -> ffi::cui_file_dialog_opts {
        ffi::cui_file_dialog_opts {
            kind: match self.kind {
                FileDialogKind::Open => ffi::cui_file_dialog_kind::CUI_FILE_OPEN,
                FileDialogKind::Save => ffi::cui_file_dialog_kind::CUI_FILE_SAVE,
                FileDialogKind::Folder => ffi::cui_file_dialog_kind::CUI_FILE_FOLDER,
            },
            title: cstr(a, self.title),
            filter_name: cstr(a, self.filter_name),
            filter: cstr(a, self.filter),
            default_name: cstr(a, self.default_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_layout() {
        assert_eq!(core::mem::size_of::<Corners>(), 16);
        assert_eq!(Corners::all(3.0).br, 3.0);
    }
}
