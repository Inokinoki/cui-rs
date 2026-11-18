# Changelog — `cui-sys` (raw FFI + the vendored C amalgamation)

## Unreleased

0.1.3 shipped a batch of generated declarations that no caller could use
correctly. All of them came from `scripts/gen_ffi.py`, and none could be
seen by the tests (`tests/link.rs` took its parameter types from the Rust
declarations themselves; `tests/layout.rs` checked sizes and offsets only).
Fixed at the root in the generator, with the checks that were missing. Per
the README's versioning exception (a signature that could not have had a
working caller) these can ship in a patch release; every changed signature
is listed.

Fixed — struct fields:

- **Arrays of `const char *` were `[*mut c_char; N]`**: `cui_strings`'
  `month`, `month_short`, `weekday`, `weekday_short` and `weekday_min` are
  `[*const c_char; N]` now, as C declares them (same layout). This one is
  declaration correctness, not the exception above — a `*mut` array built
  from `CString::into_raw` did work — so code filling those fields swaps
  its `*mut` for `*const` (`CString::as_ptr` gives one directly).

Fixed — extern signatures (each is what C declares in `cui.h`):

- **Dropped function-pointer parameter** — C calls whatever sat in the
  missing argument's register: `cui_state_on_free` (lost `fn`) and
  `cui_tree_sort` (lost `cmp`; the caller's `user` landed in `cmp`'s slot and
  was called as code). Both now take
  `Option<unsafe extern "C" fn(..)>`.
- **`const char *const *` was `*mut c_char`** (one pointer level lost):
  `cui_blame_ex`, `cui_feature_flags_ex`, `cui_input_file_drop`,
  `cui_multi_select_ex`, `cui_path_bar_ex`, `cui_pipeline_ex`,
  `cui_prop_enum_ex`, `cui_query_match`, `cui_radar_chart_ex`,
  `cui_sankey_ex`, `cui_split_button_ex`, `cui_stepper`,
  `cui_toggle_group_ex`, `cui_wheel_picker_ex`, `cui_word_rotate_ex` — now
  `*const *const c_char`. The same bug in 14 opts-struct fields (`labels`,
  `group_names`, `places`, `options`, `row_labels`, `col_labels`,
  `categories`, `month_names`, `day_names`, `icons` ×2, `choices`,
  `item_icons`, `source`).
- **Pointer out-parameters lost a level**: `cui_regex_search`'s `error`
  (`*mut *const c_char`) and `cui_signature_points`' `points` /
  `stroke_ends` (`*mut *const T`).
- **Variadics were declared non-variadic** (undefined behaviour to call a
  variadic C function through a fixed prototype): `cui_textf`,
  `cui_table_cell_textf`, `cui_terminal_printf`, `cui_log_append` now end in
  `...`.
- **`cui_log_appendv` is no longer bound**: its `va_list` was declared as
  `c_void`, an ABI mismatch with no portable Rust spelling. Use the
  variadic `cui_log_append` or `cui_log_append_text`.
- **`cui_create` takes `*const cui_ctx_desc`** (was `*const c_void`); the
  `cui_ctx_desc` and `cui_allocator` structs are now bound (with layout
  checks).

Fixed — constants:

- **`1 << N` enum initializers were evaluated as the running counter**:
  `CUI_TREE_HAS_CHILDREN/OPEN/SELECTED/LOADING` were 0/1/2/3 instead of
  1/2/4/8 (`CUI_MOD_*` had the same bug in `gen.rs` and was only hidden by
  hand-written copies). The generator now evaluates constant expressions and
  fails on anything it cannot evaluate.
- The 59 constants duplicated by hand in `lib.rs` are gone (`gen.rs` is the
  single source, checked against C). **Removed**: `CUI_COMPOSER_RECORD`,
  `CUI_COMPOSER_CANCEL`, `CUI_COMPOSER_VOICE` (32/64/128) — they do not
  exist upstream, whose `cui_composer` flags end at `MODEL` (16).

Fixed — generated docs: rustdoc comments were attached to the wrong
function (comment offsets were taken from the wrong text); each item now
carries the comment directly above it in the header.

Added — the checks that would have caught all of the above:

- **`probe/sigcheck.c`** (generated) assigns every Rust extern, mapped back
  to C, to a function pointer; `scripts/sigcheck.sh` builds it with
  `-Werror`, so a parameter count, pointer level, qualifier or variadic
  mismatch against `cui.h` fails. Runs in CI and from `vendor.sh`.
- **Every enum value is checked**: all 314 header members and every variant
  of the hand-written `#[repr]` enums against the C probe (`tests/layout.rs`;
  the old probe spot-checked 18).
- The generator now fails (instead of skipping) on any parameter, field or
  enum initializer it cannot parse, and CI fails when the committed
  generated files are stale.

Changed:

- `build.rs` links Cocoa/QuartzCore/OpenGL (macOS) and X11 (Linux) only with
  the `app` feature, the only code that uses them.
- `build.rs` error messages no longer send crates.io users to `./vendor.sh`
  (it is not in the package) when it is absent.

## 0.1.3+cui1.4.0

Fixed:

- **`cui_log_get` was declared as `cui_log_get_raw`** — a symbol the C
  library does not export, so downstream `cui::log_get` failed to link.
- **A link test now covers every extern declaration**
  (`tests/link.rs`, generated): each of the 500 symbols is materialized as a
  typed function pointer in the test binary, so a declaration naming
  something the vendored CUI does not export fails `cargo test -p cui-sys`
  instead of a user's build.
- **The layout probe and tests cover every struct size and field offset**
  (*not* enum values or function signatures: the next release found 25 wrong
  declarations (24 changed, 1 removed) that this did not see, and adds those checks).
  The probe used to print its
  generated `SZ()` lines after `base_probe` had switched the TOML section to
  `[enum]`, so 233 struct sizes landed in the enum table and were never
  checked (the tests also spot-checked only 69 sizes / 16 offsets by hand).
  `gen_probe.c` now emits proper `[size]`/`[offset]` sections, and
  `tests/layout.rs` (generated) checks **every field offset of every FFI
  struct** — 302 sizes and 1716 offsets — against the C compiler's answers.
  That caught three real bugs, fixed with it:
  - `cui_chart_opts` and `cui_progress_opts` carried a `dither` field the
    C struct does not have (silently ignored by C, wrong `size_of`);
  - `cui_tree_table_opts` was missing the `label` callback field, so its
    `user` field sat at C's `label` offset — a passed function pointer
    would have been called as data;
  - `cui_platform_iface` and `cui_renderer_iface` were missing all their
    callback fields (function-pointer fields now map to
    `Option<unsafe extern "C" fn(..)>` instead of being dropped).
- **Packaging**: `c/keep.c` and `c/keep.h` — byte-identical duplicates of
  the amalgamation left over from an earlier fetching scheme — are gone;
  the packaged crate halves from 7.5 MiB to 3.8 MiB.
- `cui-rs` now requires `cui-sys = "0.1.3"` (the 0.1.0 requirement could
  have combined with the pre-0.1.2 wrong `cui_file_entry` layout).

## 0.1.2+cui1.4.0

- macOS window-layer build fixed: the amalgamation is compiled as
  Objective-C with `_DARWIN_C_SOURCE` (the amalgamation defines
  `_POSIX_C_SOURCE` mid-file, hiding the BSD types `netinet/ip.h` needs on
  the Cocoa → PrintCore → CUPS include path).

## 0.1.1+cui1.4.0

- Generated FFI extended to the complete release API (500 `CUI_API`
  functions).

## 0.1.0+cui1.4.0

- First release: digest-checked vendored sources (offline, crates.io-
  compatible), the `links = "cui"` guarantee, capability cfgs and the
  per-version ABI probe plumbing.
