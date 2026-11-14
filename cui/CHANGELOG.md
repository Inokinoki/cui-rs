# Changelog — `cui` (the safe layer; package `cui-rs`)

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
