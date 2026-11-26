# Changelog — `cui` (the safe layer; package `cui-rs`)

## Unreleased

Needs the matching unreleased `cui-sys` (publish `cui-sys` first). Soundness fixes —
each was reachable from safe code — plus the follow-through for the
corrected `cui-sys` signatures (see its changelog).

Fixed:

- **Value formats are checked.** `Slider::format`, `Chart::format`,
  `DotChart::format` / `x_format` and `EventTrack::format` went to C's
  `vsnprintf` as they stood: `Some("%s")` made printf read a pointer that
  was never passed (a crash from safe code). They accept `%%` and one float
  conversion, and panic on anything else.
- **An icon array shorter than the items is refused.** `Segmented::icons`,
  `Tabs::icons` and `Select::icons` are indexed by CUI with the *item*
  count, so one icon for three items read past the slice (a crash). They
  hold one entry per item now, or none.
- **`measure_text`, `draw_emoji` and `emoji_supported` take no `len`**
  (**API change**): C read exactly `len` bytes of the string's copy, so a
  larger `len` read past it. The string's own byte length is passed.
- **`emoji_font_load_memory` is hand-written** (**API change**). It was
  `(&self, data: &c_void, size: usize, copy: bool)`: `size` was unchecked
  against `data`, and with `copy = false` CUI kept the pointer past the
  call while the signature borrowed it for the call. It is
  `(&self, data: &[u8])` and always copies; the zero-copy form is the raw
  `cui_sys` function.
- **`App` is not `Send`** (**API change**): a window and its GL context
  cannot move between threads on Win32 or Cocoa — `cui_app_destroy` would
  run on a thread that did not create them. Create it on the thread that
  runs its loop, as upstream documents.
- `scripts/gen_ffi.py` leaves a function to a hand-written method when a
  thin wrapper cannot vouch for its pointers (a `const void *` that no
  count turns into a slice, a text buffer with an `int` capacity), and its
  `&mut String` wrappers pass the capacity again (the argument was
  missing).
- The crate docs say what a raw `opts` `format` may hold, and that `App`
  stays on its thread.

Also: the headless `frame` helper is shared (`test_support`) and destroys
its context on a panic; `soundness_regressions` covers the four fixes above
(each crashed or read out of bounds before); CI runs the tests under
AddressSanitizer on Linux; the license text of the Lucide icons ships in
the package (`licenses/`).

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
- **`table_begin`** freed its column array on return, while C keeps the
  pointer and reads it in every `table_cell` / `table_cell_text` (a
  use-after-free in every table). The columns live in the frame arena now.
- **`Ui::canvas`** took any `T: Copy`, so `T` could borrow data that is
  gone when C calls the painter (at render time, in the next `App::frame`).
  **API change**: `T` must be `'static`; an over-aligned `T` panics.
- **`data_grid_begin`**, **`tree_table_begin`** and **`reorder_begin`** are
  hand-written: C keeps their arrays past the call. The column arrays and
  `order` are copied into the frame arena (`order` is copied back, as C
  only writes it during the call). **API change**: `tree_table_begin` is
  `unsafe` and takes `nodes: *mut [cui_tree_node]`, since C reads and
  writes the nodes until `tree_table_end`. C keeps the pointers inside
  `opts` as well (`selected`, `sort`, `cursor`, `anchor`; the tree table's
  `cursor`, `sort` and `label`/`user`) and reads and writes them up to the
  matching `*_end` — their docs say so now, and so must the columns'
  `title` strings outlive the call.
- **`terminal_init`** is `unsafe` (**API change**): the terminal buffer
  keeps pointers into `text` and `lines` for as long as it is used.
- **`Ui::state` ids are its own.** The id went to C as given, into the
  table CUI's widgets use, so an id equal to a widget's (the same name
  hashed, a small integer) named the widget's memory, and a widget whose
  state was all zeroes looked like fresh storage and was overwritten. Ids
  are remapped one-to-one before they reach C.
- **`App::screenshot`** (new) returns `(width, height, pixels)` with the
  size CUI rendered at, read back from CUI, retrying once when that differs
  from `framebuffer_size` (a resize between frames, a rounding difference).
  `screenshot_rgba` is built on it and returns `None` rather than a buffer
  whose shape is not `framebuffer_size`.
- **`Arena` settles on one chunk.** A frame that needed several chunks kept
  only the largest, so it freed and reallocated the rest every frame; the
  reset now leaves a single chunk big enough for the whole frame.
- **`set_strings`** is `unsafe` (**API change**): C keeps the table pointer
  — and every string pointer inside it — until a later `set_strings` or the
  context dies, reading them whenever a calendar renders. The old safe
  signature accepted a borrow that ends when the call returns.
- **`carousel_begin`** is `unsafe` (**API change**): C keeps the `index`
  pointer and reads and writes it in the matching `carousel_end` (the
  arrows and dots write the picked slide there), after the borrow has
  ended.
- The crate docs now say that strings inside the raw `cui_*` item structs
  passed by slice (`bar_list`'s `label`, `scatter_chart`'s `name`, …) —
  and the string fields inside some widgets' raw `opts`, which C's paint
  data reads at draw time (`heatmap`'s `row_labels`/`col_labels`, the
  small charts' `format`) — are kept as they stand and read at draw time,
  after your code returns: make them `'static` or pass them through
  `Ui::cstr` (the arena's `cstr_array` for label arrays).
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
