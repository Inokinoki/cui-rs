# Changelog — `cui-sys` (raw FFI + the vendored C amalgamation)

## 0.1.3+cui1.4.0

Fixed:

- **`cui_log_get` was declared as `cui_log_get_raw`** — a symbol the C
  library does not export, so downstream `cui::log_get` failed to link.
- **A link test now covers every extern declaration**
  (`tests/link.rs`, generated): each of the 500 symbols is materialized as a
  typed function pointer in the test binary, so a declaration naming
  something the vendored CUI does not export fails `cargo test -p cui-sys`
  instead of a user's build.
- **The layout probe and tests are exhaustive.** The probe used to print its
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
