# cui-rs — Rust bindings for CUI

Safe, idiomatic Rust bindings for [CUI](https://github.com/yappologistic/CUI),
an immediate-mode UI library written in C99 that draws its own components.

```toml
[dependencies]
cui = { path = "path/to/cui-rs/cui" }
```

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

## What is bound (v0.1)

- **App layer**: window lifecycle, `run`, quit, title/size, backend queries,
  software-renderer detection, screenshot buffer.
- **Layout**: `column` / `row` / `box_` / `card` / `scroll`, spacer, gap,
  separator; the full `BoxOpts` field set (pad/gap/align/justify/bg/border/
  radius/elevation/scroll/clip/opacity/direction/grid).
- **Text**: `text`, heading, label, secondary, muted, code, kbd, link, icon.
- **Buttons**: button, icon button (all variants, sizes, icons, tooltips).
- **Selection**: checkbox, switch, radio, radio group, slider, chip, segmented.
- **Input**: input, textarea, search (with all `Input` options), number, select.
- **Navigation**: tabs, nav item, nav section.
- **Overlays**: dialog (+ footer), toast (+ handles), tooltip.
- **Display**: badge, empty state, metric, progress.
- **Charts**: line, bar, donut, sparkline (series, labels, annotations).
- **Table**: begin/row/cell/end with sorting through `TableSort`.
- **Identity**: push/pop id scopes, rects, hover/disabled queries.
- **Input queries**: keys, shortcuts, mouse, clipboard.
- **Theme**: light/dark, accents, color resolution, reduced motion.

Not bound yet (the raw FFI in `cui-sys` already declares some of them): menus
and popovers, sheets, alerts, tables' virtualized data grid, the code editor,
markdown, docking, terminal, calendar/date picker, custom canvas painting, the
renderer SPI for bringing your own window. Adding a component is mechanical —
one `extern "C"` line in `cui-sys`, one options struct in `opts.rs`, one
method in `lib.rs` — see the git history for the pattern.

## Platform notes

The vendored amalgamation keeps CUI's own window backends: Win32, Cocoa, X11,
plus SDL2/GLFW builds upstream. This crate compiles and links:

- **Linux (tested)**: needs `libx11-dev` (and a GL or Mesa runtime at run
  time; CUI falls back to its software renderer without a GPU).
- **Windows**: links user32/gdi32/opengl32/... automatically.
- **macOS**: compiles the amalgamation as Objective-C and links the Cocoa
  frameworks (config mirrors upstream CI; untested here).

Features map to the C macros: `default-fonts` (Inter/Geist/Lucide embedded),
`stb-image` (PNG/JPEG loading), `app` (window layer; off = bring your own
window), `text-shaping` (Arabic shaping + bidi).

## Repository layout

```
cui-sys/   raw FFI + the vendored C amalgamation (cui.h, cui.c), built by cc
cui/       the safe API (App, Ui, options structs, icons)
cui/examples/  hello.rs, todo.rs — ports of the C examples
vendor.sh  regenerates the vendored C + icons from an upstream CUI checkout
```

## Building and testing

```sh
cargo build --examples       # needs libx11-dev on Linux
cargo test                   # layout tests against C sizes, doctests
cargo run -p cui --example hello
cargo run -p cui --example todo
```

To update the vendored C sources after upstream changes:
`./vendor.sh path/to/CUI-checkout`.

## License

MIT, like CUI. The vendored C sources keep upstream's license notes; the
bundled fonts, stb libraries and Lucide icons keep their own licenses (see
`LICENSE` and the notes at the end of `cui-sys/c/cui.h`).
