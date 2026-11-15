# Changelog — `cui` (the safe layer; package `cui-rs`)

## Unreleased

Added:

- **`view!`**, a declarative front end (new crate `cui-rs-macros`, behind the
  default `macros` feature; publish it first). It expands to the nested
  closures and opts structs you would write by hand. Opts types are also
  reachable as `cui::view::*`. See the `view` module docs and
  `examples/todo_view.rs`.
- **`view!` and your own widgets**: opts types are looked up in `cui::view`, then in the caller's scope, and `#![containers(..)]` declares custom scope widgets.
- **Generated widgets work in `view!`.** `scripts/gen_ffi.py` now emits closure
  methods for the 20 begin/end pairs the C header defines as scope macros
  (`accordion`, `popover`, `toolbar`, `dock_panel`, ...), safe opts structs
  (`cui::view::Alert`, ...) and `name_with` methods for the generated widgets
  whose opts hold plain data. The begin/end methods stay.
- **Safe options for the rest of the generated widgets.** Values CUI writes
  back are `Option<&mut T>` (the options go in as `&mut`), arrays are slices
  that carry their length, and lengths C takes from the call are asserted.
  New hand-written closure scopes hold those borrows until the scope ends:
  `Ui::table`, `virtual_list`, `data_grid` / `data_grid_with`,
  `tree_table` / `tree_table_with` (returns `TreeTableEvents`), plus
  `emoji_picker_with` and `hex_edit` / `hex_edit_with`.
- `view!`: `|row|` parameters before a scope's body, `&mut` options, and the
  hand-written widgets whose method always takes options (`Typewriter`,
  `ChatComposer`, `ToolCall`, ...).

Needs the matching unreleased `cui-sys` (publish `cui-sys` first). Soundness fixes —
each was reachable from safe code — plus the follow-through for the
corrected `cui-sys` signatures (see its changelog).

Fixed:

- **`Arena` is chunked.** It was one growable `Vec`, so any growth moved
  every pointer already handed out in the frame: `cstr_array` wrote through
  its own freed base pointer, and every `*_with` method that combines
  arena strings (`Button::flatten` + the label, `chart_impl`, `blame`, …)
  could hand C a dangling pointer once a frame outgrew 8 KiB. Chunks never
  move now.
- **`Ui::state`** returned `&'a mut T` not tied to `&mut self`, so two calls
  gave two live `&mut` to the same memory, and it trusted zero bits to be a
  valid `T`. It now borrows `self`, starts from `T::default()`, and panics
  when an id is reused with a different `T` (the check lives in C's
  allocation, so it survives CUI's garbage collection). **API change**:
  `T` must be `'static`, and the returned reference's lifetime is shorter.
- **`signature_points`** passed `&cui_vec2` / `&i32` where C writes a
  pointer (an 8-byte write into 4 bytes). It is hand-written now and returns
  owned copies: `(Vec<cui_vec2>, Vec<i32>)`. **API change**: it no longer
  returns the point count.
- **`otp_input`** now grows the `String` to the `min(length, 12) + 1` bytes
  C writes (a short buffer was a heap overflow). `begin_c_string` leaves
  room for the NUL when a `String` is exactly full (C's `strlen` ran off
  the end).
- **`TableSort`** is `#[repr(C)]` (C writes `column`/`descending` through a
  pointer cast from it) with compile-time layout assertions.
- **`textf` / `table_cell_textf`** passed their text as a printf format, so
  a `%` read variadic arguments that do not exist. The text is now the
  argument of a `"%s"` format and shown verbatim.
- **`blame`** asserts `lines.len() == line_commit.len()` (C indexes both
  with one count).
- **`App` has a `Drop`** that destroys the window and GL context (an early
  return or panic in a manual loop leaked them); `App::destroy` is now just
  `drop`.
- **`screenshot_rgba`** truncated the DPI scale, so 125% / 150% displays
  got `None` (and the logical size was used for the buffer). It uses the
  framebuffer size C uses; new `App::framebuffer_size()` gives it, and the
  gallery's PNG writer uses it.
- The gallery passed NUL-packed bytes for `options` / `choices` where C
  indexes a `const char *const *` (the corrected types exposed it).

Removed:

- `Ui::state_on_free`: it passed no callback, so C called garbage when the
  state was freed; there is no sound closure story yet.

Also: `scripts/check_png.py` now requires IHDR first, IEND last and a
pixel-data length that matches the dimensions.

## 0.1.3+cui1.4.0

The crate is published to crates.io as **`cui-rs`** (crates.io's `cui` name
belongs to an unrelated yanked 2017 crate). The library name stays `cui`, so
`use cui::…` is unchanged.

Fixed:

- **`cui::log_get` did not link** — it referenced `cui_log_get_raw`, a symbol
  the C library does not export; any program calling it failed at link time.
  It now calls C's `cui_log_get` and returns `None` for an out-of-range line
  (C answers those with a NULL pointer, which was also dereferenced).
- **Generated signatures that passed a single reference where C indexes a
  buffer** (undefined behaviour for any real caller, so no working call
  site existed — fixed in a patch release on that basis):
  - `Ui::hex_view` takes `data: &[u8]` (was `&u8` plus a separate `size`);
  - `Ui::gradient_editor` takes `stops: &mut [cui_gradient_stop]` and sizes
    the call from the slice (was a single element plus a raw `capacity`);
  - `Ui::diff_view_rows` takes `rows: &mut [cui_diff_row]` likewise.
- **Slice dimension asserts**: `heatmap`, `calendar_heatmap`, `level_meter`
  and `prop_vec` now assert up front that the slice covers the dimensions
  they pass to C, instead of letting C read past a short slice.
- `Arena` is exported (`pub use arena::Arena`): `Ui::arena` returned it, but
  the type could not be named outside the crate.
- All 22 broken intra-doc links resolved; the crate now denies
  `rustdoc::broken_intra_doc_links`.

## 0.1.2+cui1.4.0

- The c-string safety work in the text-field helpers (NUL-terminated, zeroed
  buffers in; UTF-8 re-validated, no self-overlapping copy out) extended to
  the generated layer; macOS window-layer build fixed
  (`_DARWIN_C_SOURCE`); the component gallery example — 12/12 upstream
  examples ported, 500/500 `CUI_API` functions bound.
- Known issues (all fixed in 0.1.3): `log_get` link error; the three broken
  generated signatures above; the gallery's screenshot PNGs carried a CRC-32
  where the zlib stream trailer requires Adler-32, so strict decoders
  rejected them.

## 0.1.1+cui1.4.0

- The component gallery port; complete release-API bindings.

## 0.1.0+cui1.4.0

- First release: safe bindings for the 11 base examples, digest-checked
  vendored sources, per-version layout tables, CI on all three desktop OSes.

Fixed (view!):

- The `view!` module docs gained a "Sharp edges" section (dropped `Card`
  click results, evaluation order, uppercase-call capture, error recovery,
  struct-literal conditions, the `::cui` path). `gen_ffi.py` now fails loudly
  on a C enum it cannot turn into a Rust enum, instead of emitting a struct
  that would not compile.
