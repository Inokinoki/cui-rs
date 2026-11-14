//! Second batch of [`Ui`] methods: menus, splits, overlays, conversation,
//! instruments, canvas helpers — mirroring the second half of the bound C
//! surface. Doc comments carry the C name, so the upstream docs translate.

use crate::arena::Arena;
use crate::opts::{cstr, Series, Sheet};
use crate::opts2::*;
use crate::types::*;
use crate::{begin_c_string, sync_c_string, Ui};
use cui_sys as ffi;
use core::ffi::{c_char, c_void};

impl<'a> crate::Ui<'a> {
    // ---------------------------------------------------------------- menus

    /// A dropdown menu (C's `cui_menu` scope). The closure runs only while
    /// the menu is open; it builds the items.
    pub fn menu(&mut self, label: impl AsRef<str>, f: impl FnOnce(&mut Ui<'a>)) {
        self.menu_impl(label.as_ref(), &ffi::cui_menu_opts::default(), f);
    }

    /// A menu with options ([`Menu`]).
    pub fn menu_with(&mut self, label: impl AsRef<str>, o: &Menu, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        self.menu_impl(label.as_ref(), &fo, f);
    }

    fn menu_impl(&mut self, label: &str, fo: &ffi::cui_menu_opts, f: impl FnOnce(&mut Ui<'a>)) {
        let is_open = unsafe { ffi::cui_menu_begin(self.ctx, self.arena.cstr(label), fo) };
        if is_open {
            self.scope(ffi::cui_menu_end, f);
        }
    }

    /// A context menu over the previous component (C's `cui_context_menu`
    /// scope); opens on right click. The closure runs only while it is open.
    pub fn context_menu(&mut self, id: impl AsRef<str>, f: impl FnOnce(&mut Ui<'a>)) {
        let is_open = unsafe { ffi::cui_context_menu_begin(self.ctx, self.cs(&id)) };
        if is_open {
            self.scope(ffi::cui_context_menu_end, f);
        }
    }

    /// A menu item (C's `cui_menu_item`). Returns true when clicked.
    pub fn menu_item(&mut self, label: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_menu_item_ex(self.ctx, self.cs(&label), &ffi::cui_menu_item_opts::default()) }
    }

    /// A menu item with options ([`MenuItem`]). Returns true when clicked.
    pub fn menu_item_with(&mut self, label: impl AsRef<str>, o: &MenuItem) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_menu_item_ex(self.ctx, self.cs(&label), &fo) }
    }

    /// A hairline between menu groups (C's `cui_menu_separator`).
    pub fn menu_separator(&mut self) {
        unsafe { ffi::cui_menu_separator(self.ctx) }
    }

    /// A quiet heading inside a menu (C's `cui_menu_label`).
    pub fn menu_label(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_menu_label(self.ctx, self.cs(&text)) }
    }

    // ---------------------------------------------------------------- split

    /// Two panes with a draggable divider (C's `cui_split` scope). `ratio`
    /// is the first pane's share in 0..1, owned by the app; call
    /// [`Ui::split_next`] between the panes inside the closure.
    pub fn split(&mut self, id: impl AsRef<str>, ratio: &mut f32, o: &Split, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten();
        unsafe { ffi::cui_split_begin(self.ctx, self.cs(&id), ratio, &fo) };
        self.scope(ffi::cui_split_end, f);
    }

    /// Moves to the second pane of a [`Ui::split`] (C's `cui_split_next`).
    pub fn split_next(&mut self) {
        unsafe { ffi::cui_split_next(self.ctx) }
    }

    // -------------------------------------------------------------- overlays

    /// A side drawer (C's `cui_sheet` scope). Drawn while `open` is true;
    /// closing sets it to false. The closure runs only while open.
    pub fn sheet(&mut self, title: impl AsRef<str>, open: &mut bool, o: &Sheet, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        let is_open = unsafe { ffi::cui_sheet_begin(self.ctx, self.arena.cstr(title.as_ref()), open, &fo) };
        if is_open {
            self.scope(ffi::cui_sheet_end, f);
        }
    }

    /// A Spotlight style command palette (C's `cui_command_palette`).
    /// Returns the chosen command's index, or -1.
    pub fn command_palette(&mut self, open: &mut bool, commands: &[Command]) -> i32 {
        assert!(!commands.is_empty());
        let flat: Vec<ffi::cui_command> = commands.iter().map(|c| c.flatten(self.arena)).collect();
        unsafe { ffi::cui_command_palette(self.ctx, open, flat.as_ptr(), flat.len() as i32) }
    }

    /// Page buttons (C's `cui_pagination`). `page` is 0 based; returns true
    /// when it changed.
    pub fn pagination(&mut self, id: impl AsRef<str>, page: &mut i32, page_count: i32) -> bool {
        unsafe { ffi::cui_pagination(self.ctx, self.cs(&id), page, page_count) }
    }

    /// A user menu: an avatar button that opens a menu (C's `cui_user_menu`
    /// scope). The closure runs only while the menu is open.
    pub fn user_menu(&mut self, name: impl AsRef<str>, f: impl FnOnce(&mut Ui<'a>)) {
        self.user_menu_impl(name.as_ref(), &ffi::cui_user_menu_opts::default(), f);
    }

    /// A user menu with options ([`UserMenu`]).
    pub fn user_menu_with(&mut self, name: impl AsRef<str>, o: &UserMenu, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        self.user_menu_impl(name.as_ref(), &fo, f);
    }

    fn user_menu_impl(&mut self, name: &str, fo: &ffi::cui_user_menu_opts, f: impl FnOnce(&mut Ui<'a>)) {
        let is_open = unsafe { ffi::cui_user_menu_begin(self.ctx, self.arena.cstr(name), fo) };
        if is_open {
            self.scope(ffi::cui_user_menu_end, f);
        }
    }

    /// A breadcrumb trail (C's `cui_breadcrumb`). Returns the clicked index
    /// or -1.
    pub fn breadcrumb(&mut self, items: &[&str]) -> i32 {
        assert!(!items.is_empty());
        unsafe { ffi::cui_breadcrumb(self.ctx, self.arena.cstr_array(items), items.len() as i32) }
    }

    // -------------------------------------------------------------- display

    /// An avatar with initials (C's `cui_avatar`).
    pub fn avatar(&mut self, name: impl AsRef<str>) {
        unsafe { ffi::cui_avatar_ex(self.ctx, self.cs(&name), &ffi::cui_avatar_opts::default()) }
    }

    /// An avatar with options ([`Avatar`]).
    pub fn avatar_with(&mut self, name: impl AsRef<str>, o: &Avatar) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_avatar_ex(self.ctx, self.cs(&name), &fo) }
    }

    /// A row of overlapping avatars (C's `cui_avatar_group`).
    pub fn avatar_group(&mut self, names: &[&str], max_visible: i32, size: f32) {
        assert!(!names.is_empty());
        unsafe { ffi::cui_avatar_group(self.ctx, self.arena.cstr_array(names), names.len() as i32, max_visible, size) }
    }

    /// A selectable row with optional icon, subtitle and trailing text
    /// (C's `cui_list_item`). Returns true when clicked.
    pub fn list_item(&mut self, title: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_list_item_ex(self.ctx, self.cs(&title), &ffi::cui_list_item_opts::default()) }
    }

    /// A list item with options ([`ListItem`]). Returns true when clicked.
    pub fn list_item_with(&mut self, title: impl AsRef<str>, o: &ListItem) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_list_item_ex(self.ctx, self.cs(&title), &fo) }
    }

    /// A small button that copies text and confirms with a check
    /// (C's `cui_copy_button`).
    pub fn copy_button(&mut self, id: impl AsRef<str>, text: impl AsRef<str>) -> bool {
        unsafe { ffi::cui_copy_button(self.ctx, self.cs(&id), self.cs(&text)) }
    }

    /// Circular progress with the value in the middle (C's `cui_progress_ring`).
    pub fn progress_ring(&mut self, label: impl AsRef<str>, value: f32, size: f32) {
        unsafe { ffi::cui_progress_ring(self.ctx, self.cs(&label), value, size) }
    }

    /// A chat bubble (C's `cui_chat_message`).
    pub fn chat_message(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_chat_message_ex(self.ctx, self.cs(&text), &ffi::cui_chat_opts::default()) }
    }

    /// A chat bubble with options ([`ChatMessage`]).
    pub fn chat_message_with(&mut self, text: impl AsRef<str>, o: &ChatMessage) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_chat_message_ex(self.ctx, self.cs(&text), &fo) }
    }

    /// An activity timeline entry (C's `cui_timeline_item`).
    pub fn timeline_item(&mut self, title: impl AsRef<str>) {
        unsafe { ffi::cui_timeline_item_ex(self.ctx, self.cs(&title), &ffi::cui_timeline_opts::default()) }
    }

    /// A timeline entry with options ([`TimelineItem`]).
    pub fn timeline_item_with(&mut self, title: impl AsRef<str>, o: &TimelineItem) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_timeline_item_ex(self.ctx, self.cs(&title), &fo) }
    }

    /// Types the text out (C's `cui_typewriter`). Returns true once complete.
    pub fn typewriter(&mut self, id: impl AsRef<str>, text: impl AsRef<str>, o: &TextFx) -> bool {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_typewriter_ex(self.ctx, self.cs(&id), self.cs(&text), &fo) }
    }

    /// A figure that rolls when it changes (C's `cui_number_roll`).
    pub fn number_roll(&mut self, id: impl AsRef<str>, text: impl AsRef<str>, o: &TextFx) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_number_roll_ex(self.ctx, self.cs(&id), self.cs(&text), &fo) }
    }

    // ---------------------------------------------------------- conversation

    /// A collapsible block (C's `cui_reasoning` scope). The
    /// closure runs only while the block is shown.
    pub fn reasoning(&mut self, id: impl AsRef<str>, o: &Reasoning, f: impl FnOnce(&mut Ui<'a>)) {
        let fo = o.flatten(self.arena);
        let shown = unsafe { ffi::cui_reasoning_begin(self.ctx, self.cs(&id), &fo) };
        if shown {
            self.scope(ffi::cui_reasoning_end, f);
        }
    }

    /// A tool call row that opens to its input and output (C's `cui_tool_call`).
    pub fn tool_call(&mut self, id: impl AsRef<str>, name: impl AsRef<str>, o: &ToolCall) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_tool_call_ex(self.ctx, self.cs(&id), self.cs(&name), &fo) }
    }

    /// An agent's plan with steps on a rail (C's `cui_agent_plan`).
    /// Returns the running step, or -1.
    pub fn agent_plan(&mut self, id: impl AsRef<str>, steps: &[PlanStep], o: &AgentPlan) -> i32 {
        assert!(!steps.is_empty());
        let flat: Vec<ffi::cui_plan_step> = steps.iter().map(|s| s.flatten(self.arena)).collect();
        let fo = ffi::cui_agent_plan_opts { _pad: 0, title: cstr(self.arena, o.title) };
        unsafe { ffi::cui_agent_plan_ex(self.ctx, self.cs(&id), flat.as_ptr(), flat.len() as i32, &fo) }
    }

    /// A pill with what the agent is doing right now (C's `cui_agent_status`).
    pub fn agent_status(&mut self, text: impl AsRef<str>, o: &AgentStatus) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_agent_status_ex(self.ctx, self.cs(&text), &fo) }
    }

    /// Cards for the documents an answer used (C's `cui_source_list`).
    /// Returns the clicked index or -1.
    pub fn source_list(&mut self, id: impl AsRef<str>, sources: &[Source]) -> i32 {
        assert!(!sources.is_empty());
        let flat: Vec<ffi::cui_source> = sources.iter().map(|s| s.flatten(self.arena)).collect();
        unsafe { ffi::cui_source_list(self.ctx, self.cs(&id), flat.as_ptr(), flat.len() as i32) }
    }

    /// An inline citation that shows a hover card (C's `cui_citation`).
    /// Returns true when clicked.
    pub fn citation(&mut self, number: i32, source: &Source) -> bool {
        let fs = source.flatten(self.arena);
        unsafe { ffi::cui_citation(self.ctx, number, &fs) }
    }

    /// Copy, regenerate, vote and share under an answer (C's `cui_response_actions`).
    /// Returns what was clicked.
    pub fn response_actions(&mut self, id: impl AsRef<str>, o: &mut ResponseActions) -> ResponseActionsFlags {
        let fo = o.flatten(self.arena);
        ResponseActionsFlags::from_bits(unsafe { ffi::cui_response_actions_ex(self.ctx, self.cs(&id), &fo) })
    }

    /// Cards to start a conversation with (C's `cui_prompt_suggestions`).
    /// Returns the clicked card's index or -1.
    pub fn prompt_suggestions(&mut self, id: impl AsRef<str>, items: &[PromptSuggestion], o: &PromptSuggestions) -> i32 {
        assert!(!items.is_empty());
        let flat: Vec<ffi::cui_prompt_suggestion> = items.iter().map(|s| s.flatten(self.arena)).collect();
        let fo = o.flatten();
        unsafe { ffi::cui_prompt_suggestions_ex(self.ctx, self.cs(&id), flat.as_ptr(), flat.len() as i32, &fo) }
    }

    /// The current model on a button; a list with details opens
    /// (C's `cui_model_picker`). Returns true when the selection changed.
    pub fn model_picker(&mut self, id: impl AsRef<str>, models: &[Model], selected: &mut i32, o: &ModelPicker) -> bool {
        assert!(!models.is_empty());
        let flat: Vec<ffi::cui_model> = models.iter().map(|m| m.flatten(self.arena)).collect();
        let fo = o.flatten();
        unsafe {
            ffi::cui_model_picker_ex(self.ctx, self.cs(&id), flat.as_ptr(), flat.len() as i32, selected, &fo)
        }
    }

    /// A small ring of how full the context is (C's `cui_context_meter`).
    pub fn context_meter(&mut self, id: impl AsRef<str>, used: f32, limit: f32, o: &ContextMeter) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_context_meter_ex(self.ctx, self.cs(&id), used, limit, &fo) }
    }

    /// A growing composer with attachments, a model picker and a send/stop
    /// button (C's `cui_chat_composer`). Edits `value` in place like
    /// [`Ui::input`]; returns what happened.
    pub fn chat_composer(&mut self, id: impl AsRef<str>, value: &mut String, o: &mut Composer) -> ComposerFlags {
        self.composer_impl(id.as_ref(), value, &o.flatten(self.arena))
    }

    fn composer_impl(&mut self, id: &str, value: &mut String, opts: &ffi::cui_composer_opts) -> ComposerFlags {
        let idp = self.arena.cstr(id);
        begin_c_string(value, 1023);
        let cap = value.capacity();
        let flags = unsafe { ffi::cui_chat_composer_ex(self.ctx, idp, value.as_mut_ptr() as *mut c_char, cap, opts) };
        sync_c_string(value);
        ComposerFlags::from_bits(flags)
    }

    // ----------------------------------------------------------- instruments

    /// A dot-matrix chart (C's `cui_dot_chart`). `count` values are spread
    /// evenly from `x_min` to `x_max`; returns clicks and the readout.
    pub fn dot_chart(&mut self, id: impl AsRef<str>, series: &[DotSeries], count: i32, o: &DotChart) -> DotChartResult {
        assert!(!series.is_empty());
        let flat: Vec<ffi::cui_dot_series> = series.iter().map(|s| s.flatten(self.arena)).collect();
        let fo = o.flatten(self.arena);
        let r = unsafe {
            ffi::cui_dot_chart_ex(self.ctx, self.cs(&id), flat.as_ptr(), flat.len() as i32, count, &fo)
        };
        DotChartResult { hovered: r.hovered, x: r.x, value: r.value, clicked: r.clicked }
    }

    /// Moments on a time axis, as dots you drag along it (C's `cui_event_track`).
    /// You own the array: the track keeps it sorted, edits it in place and
    /// never grows it past its length. `count` is the live event count.
    pub fn event_track(
        &mut self,
        id: impl AsRef<str>,
        times: &mut [f64],
        count: &mut i32,
        min: f64,
        max: f64,
        o: &mut EventTrack,
    ) -> EventTrackResult {
        assert!(times.len() as i32 >= *count && !times.is_empty());
        let fo = ffi::cui_event_track_opts {
            _pad: 0,
            width: size_to_ffi(o.width),
            step: o.step,
            units: o.units.to_ffi(),
            format: o.format.map_or(core::ptr::null(), |f| self.arena.cstr(f)),
            label_step: o.label_step,
            selected: o.selected.as_deref_mut().map_or(core::ptr::null_mut(), |s| s as *mut i32 as *mut core::ffi::c_int),
            no_add: o.no_add,
            no_remove: o.no_remove,
            no_axis: o.no_axis,
        };
        let r = unsafe {
            ffi::cui_event_track_ex(self.ctx, self.cs(&id), times.as_mut_ptr(), count, times.len() as i32, min, max, &fo)
        };
        EventTrackResult {
            changed: r.changed,
            added: r.added,
            removed: r.removed,
            selected: r.selected,
            dragging: r.dragging,
        }
    }

    /// One figure as an instrument shows it (C's `cui_readout`).
    pub fn readout(&mut self, label: impl AsRef<str>, value: impl AsRef<str>) {
        unsafe { ffi::cui_readout_ex(self.ctx, self.cs(&label), self.cs(&value), &ffi::cui_readout_opts::default()) }
    }

    /// A readout with options ([`Readout`]).
    pub fn readout_with(&mut self, label: impl AsRef<str>, value: impl AsRef<str>, o: &Readout) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_readout_ex(self.ctx, self.cs(&label), self.cs(&value), &fo) }
    }

    /// Text drawn with a 5 by 7 dot font (C's `cui_dot_display`).
    pub fn dot_display(&mut self, text: impl AsRef<str>) {
        unsafe { ffi::cui_dot_display_ex(self.ctx, self.cs(&text), &ffi::cui_dot_display_opts::default()) }
    }

    /// A dot-matrix display with options ([`DotDisplay`]).
    pub fn dot_display_with(&mut self, text: impl AsRef<str>, o: &DotDisplay) {
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_dot_display_ex(self.ctx, self.cs(&text), &fo) }
    }

    // ----------------------------------------------------------------- theme

    /// Scales all text in the app (C's `cui_theme_get(ui)->font_scale`).
    pub fn set_font_scale(&mut self, scale: f32) {
        unsafe { (*ffi::cui_theme_get(self.ctx)).font_scale = scale }
    }

    /// Keeps state changes but drops travel and bounce (C's `cui_set_reduced_motion`).
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        unsafe { ffi::cui_set_reduced_motion(self.ctx, reduced) }
    }
}

/// A native file dialog (C's `cui_app_file_dialog`). Blocking; returns the
/// chosen path when the user confirms. On Linux this uses zenity or kdialog
/// when installed. Callable from a frame closure: like C's `NULL` app, it
/// targets the running app.
pub fn file_dialog(o: &FileDialog) -> Option<String> {
    let arena = Arena::new();
    let fo = o.flatten(&arena);
    let mut buf = vec![0u8; 1024];
    let ok = unsafe { ffi::cui_app_file_dialog(core::ptr::null_mut(), &fo, buf.as_mut_ptr() as *mut _, buf.len()) };
    if !ok {
        return None;
    }
    let end = buf.iter().position(|&b| b == 0).unwrap_or(0);
    Some(String::from_utf8_lossy(&buf[..end]).into_owned())
}


// ---- hand-bound second-wave functions (complex signatures) ----

impl<'a> Ui<'a> {
    /// Blame view (C's `cui_blame`). `lines` are the source lines,
    /// `line_commit[i]` the commit index for each line (-1 none).
    pub fn blame(
        &mut self,
        id: impl AsRef<str>,
        lines: &[&str],
        line_commit: &[i32],
        commits: &[ffi::cui_blame_commit],
        o: &ffi::cui_blame_opts,
    ) -> ffi::cui_blame_result {
        assert!(!lines.is_empty());
        let ids = self.arena.cstr_array(lines);
        unsafe {
            ffi::cui_blame_ex(
                self.ctx,
                self.cs(&id),
                ids as *mut c_char,
                line_commit.as_ptr(),
                line_commit.len() as i32,
                if commits.is_empty() { core::ptr::null() } else { commits.as_ptr() },
                commits.len() as i32,
                o,
            )
        }
    }

    /// Inline data-grid edit box (C's `cui_data_grid_edit`). Edits `value`.
    pub fn data_grid_edit(&mut self, value: &mut String) -> bool {
        begin_c_string(value, 127);
        let ok = unsafe {
            ffi::cui_data_grid_edit(self.ctx, value.as_mut_ptr() as *mut c_char, value.capacity())
        };
        sync_c_string(value);
        ok
    }

    /// Feature flags panel (C's `cui_feature_flags`). `flags` are read and
    /// written; `envs` are extra environment override names.
    pub fn feature_flags(
        &mut self,
        id: impl AsRef<str>,
        flags: &mut [ffi::cui_feature_flag],
        envs: &[&str],
        o: &ffi::cui_feature_flags_opts,
    ) -> ffi::cui_feature_flags_result {
        let env_ids = if envs.is_empty() { core::ptr::null() } else { self.arena.cstr_array(envs) };
        unsafe {
            ffi::cui_feature_flags_ex(
                self.ctx,
                self.cs(&id),
                flags.as_mut_ptr(),
                flags.len() as i32,
                env_ids as *mut c_char,
                envs.len() as i32,
                o,
            )
        }
    }

    /// File browser panel (C's `cui_file_browser`). Writes the chosen path.
    pub fn file_browser(&mut self, id: impl AsRef<str>, path: &mut String, o: &ffi::cui_file_browser_opts) -> ffi::cui_file_browser_result {
        begin_c_string(path, 1023);
        let res = unsafe {
            ffi::cui_file_browser_ex(self.ctx, self.cs(&id), path.as_mut_ptr() as *mut c_char, path.capacity(), o)
        };
        sync_c_string(path);
        res
    }

    /// An open/save dialog drawn by CUI (C's `cui_file_picker`). Shown while
    /// `open`; writes the chosen path into `out`.
    pub fn file_picker(&mut self, id: impl AsRef<str>, open: &mut bool, out: &mut String, o: &ffi::cui_file_picker_opts) -> bool {
        begin_c_string(out, 1023);
        let ok = unsafe {
            ffi::cui_file_picker_ex(self.ctx, self.cs(&id), open, out.as_mut_ptr() as *mut c_char, out.capacity(), o)
        };
        sync_c_string(out);
        ok
    }

    /// The path of one of the files dropped on the window this frame
    /// (C's `cui_dropped_file`); pairs with `cui_dropped_file_count`.
    pub fn dropped_file(&mut self, index: i32) -> Option<String> {
        str_of(unsafe { ffi::cui_dropped_file(self.ctx, index) })
    }

    /// A row of filter fields (C's `cui_filter_bar`). Writes the search text.
    pub fn filter_bar(
        &mut self,
        id: impl AsRef<str>,
        fields: &[ffi::cui_filter_field],
        selected: &mut u32,
        search: &mut String,
    ) -> bool {
        assert!(!fields.is_empty());
        begin_c_string(search, 63);
        let ok = unsafe {
            ffi::cui_filter_bar(
                self.ctx,
                self.cs(&id),
                fields.as_ptr(),
                fields.len() as i32,
                selected,
                search.as_mut_ptr() as *mut c_char,
                search.capacity(),
            )
        };
        sync_c_string(search);
        ok
    }

    /// Lists a folder (C's `cui_fs_list`). Low-level: the gallery's file
    /// browser calls it through its own list function.
    ///
    /// # Safety
    /// `user` is passed through to CUI's list callback and must be valid
    /// for the call.
    pub unsafe fn fs_list(
        &mut self,
        user: *mut c_void,
        path: impl AsRef<str>,
        out: &mut [ffi::cui_file_entry],
        names: &mut String,
        hidden: bool,
    ) -> i32 {
        if names.capacity() < 4096 {
            names.reserve(4096);
        }
        begin_c_string(names, 4096);
        let n = unsafe {
            ffi::cui_fs_list(
                user,
                self.cs(&path),
                out.as_mut_ptr(),
                out.len() as i32,
                names.as_mut_ptr() as *mut c_char,
                names.capacity(),
                hidden,
            )
        };
        sync_c_string(names);
        n
    }

    /// An inline editable label (C's `cui_inline_edit`). Edits `value`.
    pub fn inline_edit(&mut self, id: impl AsRef<str>, value: &mut String, o: &ffi::cui_inline_edit_opts) -> i32 {
        begin_c_string(value, 127);
        let r = unsafe { ffi::cui_inline_edit_ex(self.ctx, self.cs(&id), value.as_mut_ptr() as *mut c_char, value.capacity(), o) };
        sync_c_string(value);
        r
    }

    /// Multi select (C's `cui_multi_select`). `selected` is one flag per item.
    pub fn multi_select(&mut self, id: impl AsRef<str>, items: &[&str], selected: &mut [bool], o: &ffi::cui_multi_select_opts) -> bool {
        assert!(!items.is_empty() && selected.len() >= items.len());
        let ids = self.arena.cstr_array(items);
        unsafe {
            ffi::cui_multi_select_ex(self.ctx, self.cs(&id), ids as *mut c_char, items.len() as i32, selected.as_mut_ptr(), o)
        }
    }

    /// Pipeline chart (C's `cui_pipeline`).
    pub fn pipeline(
        &mut self,
        id: impl AsRef<str>,
        stages: &[&str],
        jobs: &[ffi::cui_pipeline_job],
        o: &ffi::cui_pipeline_opts,
    ) -> ffi::cui_pipeline_result {
        assert!(!stages.is_empty() && !jobs.is_empty());
        let ids = self.arena.cstr_array(stages);
        unsafe {
            ffi::cui_pipeline_ex(
                self.ctx,
                self.cs(&id),
                ids as *mut c_char,
                stages.len() as i32,
                jobs.as_ptr(),
                jobs.len() as i32,
                o,
            )
        }
    }

    /// Renders a query tree to text (C's `cui_query_text`).
    pub fn query_text(&mut self, fields: &[ffi::cui_query_field], nodes: &[ffi::cui_query_node], out: &mut String) -> usize {
        assert!(!nodes.is_empty());
        begin_c_string(out, 1023);
        let n = unsafe {
            ffi::cui_query_text(
                if fields.is_empty() { core::ptr::null() } else { fields.as_ptr() },
                fields.len() as i32,
                nodes.as_ptr(),
                nodes.len() as i32,
                out.as_mut_ptr() as *mut c_char,
                out.capacity(),
            )
        };
        sync_c_string(out);
        n
    }

    /// Radar chart (C's `cui_radar_chart`). One `axes` label per spoke.
    pub fn radar_chart(
        &mut self,
        id: impl AsRef<str>,
        series: &[Series],
        axes: &[&str],
        o: &ffi::cui_radar_opts,
    ) -> i32 {
        assert!(!series.is_empty() && !axes.is_empty());
        let ids = self.arena.cstr_array(axes);
        let flat: Vec<ffi::cui_series> = series.iter().map(|s| s.flatten(self.arena)).collect();
        unsafe {
            ffi::cui_radar_chart_ex(self.ctx, self.cs(&id), flat.as_ptr(), flat.len() as i32, ids as *mut c_char, axes.len() as i32, o)
        }
    }

    /// Regex tester (C's `cui_regex_tester`). Both `pattern` and `text` are
    /// editable buffers.
    pub fn regex_tester(
        &mut self,
        id: impl AsRef<str>,
        pattern: &mut String,
        text: &mut String,
        o: &ffi::cui_regex_tester_opts,
    ) -> ffi::cui_regex_tester_result {
        for s in [&mut *pattern, &mut *text] {
            begin_c_string(s, 255);
        }
        let r = unsafe {
            ffi::cui_regex_tester_ex(
                self.ctx,
                self.cs(&id),
                pattern.as_mut_ptr() as *mut c_char,
                pattern.capacity(),
                text.as_mut_ptr() as *mut c_char,
                text.capacity(),
                o,
            )
        };
        sync_c_string(pattern);
        sync_c_string(text);
        r
    }

    /// A request builder bar (C's `cui_request_bar`). Writes the URL.
    pub fn request_bar(
        &mut self,
        id: impl AsRef<str>,
        method: &mut i32,
        url: &mut String,
        o: &ffi::cui_request_bar_opts,
    ) -> ffi::cui_request_bar_result {
        begin_c_string(url, 511);
        let r = unsafe {
            ffi::cui_request_bar_ex(self.ctx, self.cs(&id), method, url.as_mut_ptr() as *mut c_char, url.capacity(), o)
        };
        sync_c_string(url);
        r
    }

    /// Sankey diagram (C's `cui_sankey`).
    pub fn sankey(&mut self, id: impl AsRef<str>, nodes: &[&str], links: &[ffi::cui_sankey_link], o: &ffi::cui_sankey_opts) -> i32 {
        assert!(!nodes.is_empty() && !links.is_empty());
        let ids = self.arena.cstr_array(nodes);
        unsafe {
            ffi::cui_sankey_ex(self.ctx, self.cs(&id), ids as *mut c_char, nodes.len() as i32, links.as_ptr(), links.len() as i32, o)
        }
    }

    /// Schedule picker (C's `cui_schedule_picker`). Writes the cron text.
    pub fn schedule_picker(&mut self, id: impl AsRef<str>, cron: &mut String, o: &ffi::cui_schedule_opts) -> ffi::cui_schedule_result {
        begin_c_string(cron, 127);
        let r = unsafe { ffi::cui_schedule_picker_ex(self.ctx, self.cs(&id), cron.as_mut_ptr() as *mut c_char, cron.capacity(), o) };
        sync_c_string(cron);
        r
    }

    /// A button with a dropdown of extra actions (C's `cui_split_button`).
    /// Returns the chosen item's index or -1.
    pub fn split_button(&mut self, label: impl AsRef<str>, items: &[&str], o: &ffi::cui_split_button_opts) -> i32 {
        assert!(!items.is_empty());
        let ids = self.arena.cstr_array(items);
        unsafe { ffi::cui_split_button_ex(self.ctx, self.cs(&label), ids as *mut c_char, items.len() as i32, o) }
    }

    /// Process steps (C's `cui_stepper`).
    pub fn stepper(&mut self, steps: &[&str], current: i32) {
        assert!(!steps.is_empty());
        let ids = self.arena.cstr_array(steps);
        unsafe { ffi::cui_stepper(self.ctx, ids as *mut c_char, steps.len() as i32, current) }
    }

    /// Tag input (C's `cui_tag_input`). Tags are comma separated in `value`.
    pub fn tag_input(&mut self, id: impl AsRef<str>, value: &mut String, o: &ffi::cui_tag_opts) -> bool {
        begin_c_string(value, 255);
        let ok = unsafe { ffi::cui_tag_input_ex(self.ctx, self.cs(&id), value.as_mut_ptr() as *mut c_char, value.capacity(), o) };
        sync_c_string(value);
        ok
    }

    /// Terminal emulator state (C's `cui_terminal_init`). `text` is the
    /// scrollback buffer, `lines` the row cache.
    pub fn terminal_init(
        &mut self,
        t: &mut ffi::cui_terminal_buffer,
        text: &mut String,
        lines: &mut [ffi::cui_terminal_line],
    ) {
        begin_c_string(text, 8191);
        unsafe {
            ffi::cui_terminal_init(t, text.as_mut_ptr() as *mut c_char, text.capacity(), lines.as_mut_ptr(), lines.len() as i32)
        }
        sync_c_string(text);
    }

    /// A row of toggles (C's `cui_toggle_group`). One flag per label.
    pub fn toggle_group(&mut self, id: impl AsRef<str>, labels: &[&str], values: &mut [bool], o: &ffi::cui_toggle_group_opts) -> bool {
        assert!(!labels.is_empty() && values.len() >= labels.len());
        let ids = self.arena.cstr_array(labels);
        unsafe {
            ffi::cui_toggle_group_ex(self.ctx, self.cs(&id), ids as *mut c_char, labels.len() as i32, values.as_mut_ptr(), o)
        }
    }

    /// Writes URL params for the web build; a no-op natively
    /// (C's `cui_url_set_params`).
    pub fn url_set_params(&mut self, url: &mut String, pairs: &[ffi::cui_kv_pair]) -> bool {
        assert!(!pairs.is_empty());
        begin_c_string(url, 511);
        let ok = unsafe {
            ffi::cui_url_set_params(url.as_mut_ptr() as *mut c_char, url.capacity(), pairs.as_ptr(), pairs.len() as i32)
        };
        sync_c_string(url);
        ok
    }

    /// A wheel picker (C's `cui_wheel_picker`). Writes the chosen index.
    pub fn wheel_picker(&mut self, id: impl AsRef<str>, items: &[&str], index: &mut i32, o: &ffi::cui_wheel_opts) -> bool {
        assert!(!items.is_empty());
        let ids = self.arena.cstr_array(items);
        unsafe { ffi::cui_wheel_picker_ex(self.ctx, self.cs(&id), ids as *mut c_char, items.len() as i32, index, o) }
    }

    /// A key/value editor (C's `cui_kv_editor`). `count` tracks the used
    /// rows; `pairs` is the fixed-capacity table.
    pub fn kv_editor(&mut self, id: impl AsRef<str>, pairs: &mut [ffi::cui_kv_pair], count: &mut i32, o: &ffi::cui_kv_editor_opts) -> bool {
        assert!(!pairs.is_empty() && *count >= 0 && (*count as usize) <= pairs.len());
        unsafe {
            ffi::cui_kv_editor_ex(self.ctx, self.cs(&id), pairs.as_mut_ptr(), count, pairs.len() as i32, o)
        }
    }

    /// A nested filter/query builder (C's `cui_query_builder`). `count`
    /// tracks the used nodes; `nodes` is the fixed-capacity arena.
    pub fn query_builder(
        &mut self,
        id: impl AsRef<str>,
        fields: &[ffi::cui_query_field],
        nodes: &mut [ffi::cui_query_node],
        count: &mut i32,
        o: &ffi::cui_query_opts,
    ) -> bool {
        assert!(!fields.is_empty() && !nodes.is_empty() && *count >= 0 && (*count as usize) <= nodes.len());
        unsafe {
            ffi::cui_query_builder_ex(self.ctx, self.cs(&id), fields.as_ptr(), fields.len() as i32, nodes.as_mut_ptr(), count, nodes.len() as i32, o)
        }
    }

    /// A spline editor (C's `cui_curve_editor`). `count` tracks the used
    /// keys; `keys` is the fixed-capacity array.
    pub fn curve_editor(
        &mut self,
        id: impl AsRef<str>,
        keys: &mut [ffi::cui_curve_key],
        count: &mut i32,
        o: &ffi::cui_curve_opts,
    ) -> ffi::cui_curve_result {
        assert!(!keys.is_empty() && *count >= 0 && (*count as usize) <= keys.len());
        unsafe {
            ffi::cui_curve_editor_ex(self.ctx, self.cs(&id), keys.as_mut_ptr(), count, keys.len() as i32, o)
        }
    }

    /// Uploads an RGBA8 image as a GPU texture (C's `cui_image_create`).
    /// The handle stays valid until [`Ui::image_destroy`](Self::image_destroy).
    pub fn image_create(&mut self, w: i32, h: i32, rgba: &[u8]) -> usize {
        assert_eq!(rgba.len(), (w * h * 4) as usize, "rgba must be w*h*4 bytes");
        unsafe { ffi::cui_image_create(self.ctx, w, h, rgba.as_ptr().cast()) }
    }

    /// Words fade and rise into place one after another (C's `cui_word_rotate`).
    pub fn word_rotate(&mut self, id: impl AsRef<str>, words: &[&str], interval: f32, o: &crate::TextFx) {
        assert!(!words.is_empty());
        let ids = self.arena.cstr_array(words);
        let fo = o.flatten(self.arena);
        unsafe { ffi::cui_word_rotate_ex(self.ctx, self.cs(&id), ids as *mut c_char, words.len() as i32, interval, &fo) }
    }
}

/// The date `days` days after the epoch CUI uses (C's `cui_date_from_days`).
#[allow(dead_code)] // used by the gallery example
pub fn date_from_days(days: i32) -> Date {
    Date::from_ffi(unsafe { ffi::cui_date_from_days(days) })
}

/// Converts a borrowed C string from the library into a Rust `String`.
fn str_of(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { core::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }
}

/// The display name of a key, e.g. `"F5"` (C's `cui_key_name`).
pub fn key_name(key: Key) -> Option<String> {
    str_of(unsafe { ffi::cui_key_name(key.to_ffi() as i32) })
}

/// The reason line for an HTTP status, e.g. `"Not Found"` (C's `cui_http_reason`).
pub fn http_reason(status: i32) -> Option<String> {
    str_of(unsafe { ffi::cui_http_reason(status) })
}

/// The name of an HTTP method token, e.g. `"GET"` (C's `cui_http_method_name`).
pub fn http_method_name(method: i32) -> Option<String> {
    str_of(unsafe { ffi::cui_http_method_name(method) })
}

/// The symbol of a query-builder operator (C's `cui_query_op_name`).
pub fn query_op_name(ty: i32, op: i32) -> Option<String> {
    str_of(unsafe { ffi::cui_query_op_name(ty, op) })
}

/// One row of a [`cui_log_buffer`](cui_sys::cui_log_buffer) (C's `cui_log_get`).
/// Returns the line's metadata and its text, or `None` when `i` is out of
/// range (C returns a NULL line pointer) or the line is empty.
pub fn log_get(buf: &ffi::cui_log_buffer, i: i32) -> Option<(ffi::cui_log_line, String)> {
    let mut text: *const c_char = core::ptr::null();
    let line = unsafe { ffi::cui_log_get(buf, i, &mut text) };
    if line.is_null() {
        return None;
    }
    let line = unsafe { *line };
    if line.len == 0 || text.is_null() {
        return None;
    }
    str_of(text).map(|s| (line, s))
}

/// Days since the CUI epoch (C's `cui_date_to_days`).
#[allow(dead_code)] // used by the gallery example
pub fn date_to_days(d: Date) -> i32 {
    unsafe { ffi::cui_date_to_days(d.to_ffi()) }
}

/// Today according to the system clock (C's `cui_date_today`).
#[allow(dead_code)] // used by the gallery example
pub fn date_today() -> Date {
    Date::from_ffi(unsafe { ffi::cui_date_today() })
}

/// Re-derives a String from a buffer CUI edited (NUL-terminated, UTF-8
/// re-validated). Used by the generated layer after a call that writes
/// into a caller-owned buffer.
pub(crate) fn sync_buffer(value: &mut String) {
    if value.capacity() == 0 {
        return; // the caller never sized a buffer for CUI to write into
    }
    crate::sync_c_string(value)
}
