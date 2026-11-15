# cui-rs — Rust bindings for CUI

Safe, idiomatic Rust bindings for [CUI](https://github.com/yappologistic/CUI),
an immediate-mode UI library written in C99 that draws its own components.

```toml
[dependencies]
cui-rs = "0.1.3"
```

The package is `cui-rs` (crates.io's `cui` is taken); the library keeps
the name `cui`, so the code below is `use cui::…` unchanged.

```rust
use cui::{App, AppDesc, Button, ButtonVariant, Column, Row, Size};

fn main() {
    let mut app = App::new(&AppDesc {
        title: Some("Hello"), width: 420, height: 300, ..Default::default()
    }).expect("could not create a window");
    let mut notifications = true;
    let mut clicks = 0;

    while app.frame() {
        let mut ui = app.ui();
        ui.column(&Column { pad: 32., gap: 16., width: Size::FILL, ..Default::default() }, |ui| {
            ui.heading("Hello, CUI", 2);
            ui.secondary("A native window, a few components and no dependencies.");
            ui.switch("Notifications", &mut notifications);
            ui.row(&Row { gap: 8., ..Default::default() }, |ui| {
                if ui.button_with("Click me", &Button { variant: ButtonVariant::Primary, ..Default::default() }) {
                    clicks += 1;
                }
                ui.text(format!("Clicked {clicks} times"));
            });
        });
    }
}
```

No other dependencies: the vendored C amalgamation is compiled by `cc` into
your binary. A release build of the app above is ~550 KB (the C equivalent is
~270 KB; the difference is mostly Rust's standard library and panic/formatting
machinery).

## How the C API maps to Rust

The binding mirrors the C API one-to-one, so the C documentation, headers and
examples translate directly.

**The frame loop and app state** — your state lives in plain variables; widgets
read and write it through references and return what happened this frame:

| C | Rust |
|---|---|
| `cui_app_create(&(cui_app_desc){ .title = "Hello" })` | `App::new(&AppDesc { title: Some("Hello"), ..Default::default() })` |
| `while (cui_app_frame(app)) { cui_ctx *ui = cui_app_ui(app); ... }` | `while app.frame() { let mut ui = app.ui(); ... }` |
| `cui_app_run(&desc, frame, &state)` | `App::run(&desc, \|ui\| state.frame(ui))` |
| `cui_app_destroy(app)` | `app.destroy()` |

**Widgets** — every component has a short method (no options) and a `_with`
method taking an options struct. The struct has the same fields as its C
`*_opts`, and `..Default::default()` plays the role of C's omitted named
arguments (zero means "default" in both):

| C | Rust |
|---|---|
| `cui_button(ui, "Save")` | `ui.button("Save")` |
| `cui_button(ui, "Save", .variant = CUI_BUTTON_PRIMARY)` | `ui.button_with("Save", &Button { variant: ButtonVariant::Primary, ..Default::default() })` |
| `cui_switch(ui, "Notifications", &on)` | `ui.switch("Notifications", &mut on)` |
| `cui_textf(ui, "Clicked %d times", n)` | `ui.text(format!("Clicked {n} times"))` |
| `cui_icon_button(ui, CUI_ICON_X, ...)` | `ui.icon_button_with(ICON_X, &Button { .. })` |

**Containers** — C's scope macros become closures. The container is ended even
when the closure panics:

| C | Rust |
|---|---|
| `cui_column(ui, .gap = 16) { ... }` | `ui.column(&Column { gap: 16., ..Default::default() }, \|ui\| { ... })` |
| `cui_row(ui, .gap = 8) { ... }` | `ui.row(&Row { gap: 8., ..Default::default() }, \|ui\| { ... })` |
| `cui_scroll(ui, .id = "list") { ... }` | `ui.scroll(&Scroll { id: Some("list"), .. }, \|ui\| { ... })` |
| `cui_dialog(ui, "Title?", &open) { ... }` | `ui.dialog("Title?", &mut open, \|ui\| { ... })` — the closure runs only while it is open |

**Text input** — C's caller-owned `char buf[96]` becomes a `String`; CUI edits
it in place and the binding re-validates UTF-8 afterwards. The capacity is the
input limit, so size it with `String::with_capacity`:

| C | Rust |
|---|---|
| `int t = cui_input(ui, "id", a->draft, sizeof a->draft);` | `let t = ui.input("id", &mut state.draft);` |
| `t & CUI_SUBMITTED` | `t.submitted()` |

**Identities in loops** — same convention and `##`/`###` suffixes as C; the
guards pop the scope automatically:

| C | Rust |
|---|---|
| `cui_push_id_int(ui, i); ...; cui_pop_id(ui);` | `let _g = ui.push_id_int(i); ...` |

**Sizes and colors** — `CUI_FILL` / `CUI_PX(v)` / `CUI_PCT(p)` become
`Size::FILL` / `Size::px(v)` / `Size::pct(p)`; `CUI_COLOR_ACCENT` becomes
`Color::ACCENT`, and the token-encoding trick is kept so semantic colors stay
theme-aware. Icons are `ICON_*` constants (`cui::ICON_PLUS`).

A working port of every example above is in [`cui/examples/`](cui/examples):
[`hello.rs`](cui/examples/hello.rs) and [`todo.rs`](cui/examples/todo.rs),
line-for-line ports of the C originals — including the filter, the
confirmation dialog and saving to `todo.txt`. The C docs
([docs/tutorial.md](https://github.com/yappologistic/CUI/blob/master/docs/tutorial.md))
can be followed along in Rust with only syntax changes.

## What is bound

**499 of the 500 `CUI_API` functions in the v1.4.0 release have a callable
Rust binding.** The exception is `cui_log_appendv`: it takes a C `va_list`,
which has no portable Rust spelling (use `cui_log_append` or
`cui_log_append_text`). Across two layers:

- **Hand-written safe layer** (~220 methods on `Ui` plus free functions):
  the components covered by the 12 ported examples — app lifecycle, layout,
  text, buttons, selection, input, navigation, overlays, display, charts,
  canvas painting, conversation, table, identity, input queries, theme, and
  native file dialogs. String parameters go through a per-frame arena; text
  fields edit `&mut String` in place — the buffer is NUL-terminated and
  zeroed before the call and re-validated as UTF-8 after it, the same
  discipline C's `char buf[96]` gives; container scopes are closures with
  panic-safe begin/end guards.
- **Generated layer** (`cui-sys/src/gen.rs` + `cui/src/gen_safe.rs`, 319
  extern functions + 235 structs, 237 safe methods): the full second-wave
  gallery surface — kanban, dock, terminal, data grid (with editing),
  carousel, card stacks, code blocks, JSON viewer, blame, coverage, API
  explorer, feature flags, test explorer, stack traces, trace viewer,
  network waterfall, dependency graph, commit graph, pipeline, problems,
  key-value editor, query builder, schedule picker, keymap, file browser,
  file picker, dropzone, emoji picker, color picker, date/time/range
  pickers, image viewer/comparer/cropper, signature pad, presence,
  reactions, ticker, now playing, waveform, tab bar, waterfall, waffle,
  funnel, sankey, treemap, radar, candlestick, gauge, activity rings, bar
  list, knob, level meter, fader, scrub, money input, inline edit, shortcut
  recorder, url params, theme switcher, spinner, loader, gradient text,
  spotlight, expandable text/cards, hover card, banner, alert, checklist,
  countdown, meter, split button, status button, swipe actions, speed dial,
  avatar upload, word rotate, shimmer text, text reveal, typing indicator,
  voice orb, week/month/range calendars, gantt, heatmap, calendar heatmap,
  filter bar, and more — including the raw name/format helpers
  (`cui_key_name`, `cui_http_reason`, `cui_shortcut_label`, …) and the
  renderer introspection (`cui_get_draw_data`).

The generated methods are "thin safe": `impl AsRef<str>` for strings (via
the frame arena), `&mut String` for editable buffers (UTF-8 re-validated
after the call), slices for pointer+count pairs, and raw `ffi::*_opts`
structs for options — construct them with `..Default::default()` and the
public [`Arena`](cui/src/arena.rs) for any string pointers. The renderer SPI
(bring your own window) is declared in the FFI. Adding a component is
mechanical — extend `scripts/ffi-manifest.txt`, re-run `scripts/gen_ffi.py`
— see the git history for the pattern.

## Platform notes

The vendored amalgamation keeps CUI's own window backends: Win32, Cocoa, X11,
plus SDL2/GLFW builds upstream. This crate compiles and links:

- **Linux (tested)**: needs `libx11-dev` (and a GL or Mesa runtime at run
  time; CUI falls back to its software renderer without a GPU).
- **Windows**: links user32/gdi32/opengl32/... automatically.
- **macOS (tested)**: compiles the amalgamation as Objective-C and links the
  Cocoa frameworks. The amalgamation defines `_POSIX_C_SOURCE` mid-file, so
  `cui-sys/build.rs` defines `_DARWIN_C_SOURCE` as well — without it the SDK's
  `netinet/ip.h` cannot see the BSD types (`u_char`, `u_int`) that the
  `Cocoa → PrintCore → CUPS` include chain pulls in.

Features map to the C macros: `default-fonts` (Inter/Geist/Lucide embedded),
`stb-image` (PNG/JPEG loading), `app` (window layer; off = bring your own
window), `text-shaping` (Arabic shaping + bidi).

## Examples

All 12 upstream examples have Rust twins in [`cui/examples/`](cui/examples):
`hello`, `todo`, `settings`, `notes`, `dashboard`, `mail`, `finance`,
`music`, `tracker`, `assistant`, `caffeine` and the **component gallery** —
line-for-line ports of the C originals, each verified by the CI smoke test
(every example runs for a few seconds under Xvfb; a crash or an early exit
fails the build, and screenshots are uploaded as artifacts).

The gallery ([`cui/examples/gallery/`](cui/examples/gallery)) covers every
component in the library: the nine base pages plus the eighteen extended
pages (kanban, data grid, terminal, charts, motion, AI, audio, developer
tools, instruments, …), ~2,900 lines of Rust across three modules. It also
doubles as a screenshot tool: `CUI_GALLERY_SHOT=path:page` renders that page
and writes a PNG, which the smoke test uses to eyeball every page.

## Repository layout

```
cui-sys/           raw FFI + the vendored C amalgamation (cui.h, cui.c), built by cc
  src/gen.rs       generated FFI: structs, enum constants, extern decls
  tests/link.rs    generated: forces every extern symbol to resolve at link
  tests/layout.rs  generated: checks every size/offset and every enum value
                   (header members and hand-written #[repr] variants) against
                   the C probe
  layout/          the C ABI facts per pinned release (written by vendor.sh)
  probe/           the C probe that prints them, and sigcheck.c (generated):
                   every Rust extern assigned to a C function pointer, so the
                   C compiler checks parameter counts, pointer levels, consts
                   and variadics against cui.h
cui/               the safe API (App, Ui, options structs, icons)
  src/gen_safe.rs  generated safe methods for the generated FFI
  examples/        12 ported C examples (the 11 base ones + the gallery)
scripts/
  gen_ffi.py       header-to-Rust generator (extends the manifest, re-run)
  sigcheck.sh      compiles probe/sigcheck.c with -Werror (CI runs it)
  check_png.py     PNG decoder check CI runs over the gallery screenshots
  smoke.sh         Xvfb smoke test for every example
vendor.sh          pins/fetches the upstream release (--pin, --fetch) or
                   amalgamates a local checkout into the offline cache
```

## Versioning

**The Rust crates version independently.** `cui-rs` and `cui-sys` follow
normal semver for their *binding* API; breaking a Rust signature bumps
`cui-rs`'s minor version. One exception: fixing a signature that could not
have had a working caller (a generated form that passed a single reference
where C indexes a buffer, dropped a callback or variadic parameter, or lost
a pointer level — undefined behaviour by construction) lands in a patch
release and is called out in the changelog.

The crates.io package is named `cui-rs` — crates.io's `cui` name has
belonged to an unrelated, yanked 2017 crate since before this binding
existed. The library name stays `cui`, so `use cui::…` reads the same as
the C API.

**The support window.** Binding 0.1.x targets CUI 1.4 only (declared in
`UPSTREAM.toml` and enforced by `build.rs`). The plumbing for multi-version
support is in place: capability cfgs (`cui_ge_1_5` etc.), a C ABI probe per
pinned release, and per-version layout tables that the tests check against.

**The bound upstream release rides in the build metadata.** Both crates are
versioned `0.1.3+cui1.4.0`: the `+cui…` suffix — the same convention as
`zstd-sys 2.0.9+zstd.1.5.5` or `libgit2-sys 0.16.2+1.7.2` — states which
upstream release the binding targets. It shows up in `cargo pkgid`, crates.io
and docs.rs without affecting dependency resolution. When the pin moves to a
new upstream release, the crates re-release as e.g. `0.1.3+cui1.5.0`: one
binding release per upstream release, so a crate version always names the
exact CUI it was written for.

**The pinned C sources are committed, digest-checked at every build.**
`cui-sys/UPSTREAM.toml` records the upstream release (version, tag and the
SHA-256 of its amalgamated release assets), and `cui-sys/c/` holds exactly
those assets. At build time `build.rs` verifies both digests **and** the
`CUI_VERSION_STRING` define inside the header, then compiles — no network
access, and compatible with crates.io, whose guidelines forbid build scripts
from touching the network. `./vendor.sh --fetch` restores the cache after a
pin change without a local checkout, and `cui-sys` uses Cargo's
`links = "cui"`, which guarantees at most one copy of the C library in any
dependency graph.

If you amalgamate a local upstream checkout instead (`./vendor.sh
<checkout>`, for hacking on upstream), `build.rs` compiles it with a
digest-mismatch warning as long as the version matches the pin.

**Support window.** Binding 0.1.x supports CUI **1.4** only — declared in
`UPSTREAM.toml` (`supported = [...]`) and enforced by `build.rs`. 0.2.0 will
widen the window to recent releases, and the plumbing is already in place:
`cui-sys` reports the compiled version through Cargo's `links` metadata
channel, `cui/build.rs` turns it into `cui_ge_*` capability cfgs (so
version-specific APIs can be `#[cfg(cui_ge_1_5)]`-gated), and every pinned
release carries a generated C ABI probe (struct sizes, field offsets, enum
values) in `cui-sys/layout/` that the layout tests check the Rust FFI
against. The same probe verifies upstream's field-append discipline, which
is what makes a moving support window safe.

**At runtime**, `cui::CUI_VERSION` (re-exported from `cui-sys`, where
`build.rs` extracts it from `c/cui.h` at compile time) always tells you which
CUI is inside your binary — e.g. for a diagnostic line or an `about` box.

To move to a new upstream release: run `./vendor.sh --pin v1.5.0` (it
downloads that release's assets, checks the embedded version, computes the
digests and rewrites `UPSTREAM.toml`), then commit the updated pin. When the
upstream major/minor version moved, mention it in both crates' changelogs
and review the upstream changelog for API changes to port into the safe
layer — the layout tests in `cui-sys` fail loudly if the C ABI moved.

## Building and testing

```sh
cargo build --examples       # needs libx11-dev on Linux; no network needed
cargo test                   # layout tests against C sizes, doctests
cargo run -p cui-rs --example hello
cargo run -p cui-rs --example todo
```

Updating the pinned upstream release: `./vendor.sh --pin <tag>`.

## License

MIT, like CUI. The vendored C sources keep upstream's license notes; the
bundled fonts, stb libraries and Lucide icons keep their own licenses (see
`LICENSE` and the notes at the end of `cui-sys/c/cui.h`).
