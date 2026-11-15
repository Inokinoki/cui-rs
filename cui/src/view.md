
# The `view!` macro

`view!` describes a frame declaratively and expands, in place, to the nested
closures and opts structs you would write by hand: no runtime, no tree.

```no_run
use cui::{view, App, AppDesc, ButtonVariant, Size};

let mut app = App::new(&AppDesc::default()).unwrap();
let (mut notifications, mut clicks) = (true, 0);

while app.frame() {
    let mut ui = app.ui();
    view! { ui,
        Column(pad: 32., gap: 16., width: Size::FILL) {
            Heading("Hello, CUI", 2);
            Switch("Notifications" => notifications);
            Row(gap: 8.) {
                Button("Click me", variant: ButtonVariant::Primary) { clicks += 1; }
                Text(format!("Clicked {clicks} times"));
            }
        }
    }
}
```

The first token is the `Ui` handle. Inside a body it names the handle of the
current scope, so plain Rust statements can use it.

- **Calls.** `Name(args)` is `ui.name(args)` (`IconButton` is `icon_button`,
  `Box` is `box_`). The arguments are the method's own, in order.
  `a => b` means `a, &mut b`, so `Select("id", &items => index)` is
  `ui.select("id", &items, &mut index)`.
- **Options.** `key: value` entries fill the opts struct and switch the call
  to its `_with` form: `Button("Save", variant: ButtonVariant::Primary)` is
  `ui.button_with("Save", &Button { variant: .., ..Default::default() })`.
  The opts type is looked up in this module (`cui::view::Name`).
- **Bodies.** Scope widgets (`Column`, `Row`, `Box`, `Card`, `Scroll`,
  `Dialog`, `DialogFooter`, `Menu`, `UserMenu`, `ContextMenu`, `Split`,
  `Sheet`, `Reasoning`) take a body of nodes. Any other widget's body runs
  when its call returns `true`: `Button("Go") { .. }` is `if ui.button(..) { .. }`.
- **Results.** `let typed = Input("id" => draft);` and
  `let clicked = Card(hoverable: true) { .. };` keep the return value.
- **Control flow.** `if`, `else`, `for`, `while`, `loop` and `match` take
  bodies of nodes; any other statement is copied as written.
  `let x = Some(1);` stays a plain statement (`Some`, `Ok`, `Err` and `Self`
  are never widgets).

```no_run
use cui::{view, Ui};

fn list(ui: &mut Ui, rows: &[String], draft: &mut String) {
    view! { ui,
        let typed = Input("new" => *draft, placeholder: Some("Add"));
        Scroll(id: Some("rows")) {
            for (i, row) in rows.iter().enumerate() {
                let _id = ui.push_id_int(i as i32);
                Row(gap: 8.) {
                    Text(row);
                    Spacer();
                    IconButton(cui::ICON_X, tooltip: Some("Delete")) { ui.toast("deleted"); }
                }
            }
            if rows.is_empty() { Muted("Nothing here"); }
        }
        if typed.submitted() { Toast("Added"); }
    }
}
```

# Built-in widgets

Every `Ui` method is a widget: `Name(args)` is `ui.name(args)`. Options
(`key: value`) and scope bodies work for:

- the hand-written widgets (`Button`, `Input`, `Column`, `Dialog`, ...);
- the generated ones, which `scripts/gen_ffi.py` derives from the C header:
  - **scope widgets** with a children closure, made from the C library's own
  scope macros (`CUI__SCOPE` runs `end` always, `CUI__SCOPE_IF` only when
  `begin` returned true, and the closure form keeps that rule and the panic
  safety of the hand-written scopes): `Accordion`, `ButtonGroup`, `CardStackItem`, `Carousel`, `CarouselSlide`, `Dock`, `DockPanel`, `Dockspace`, `ExpandableCard`, `HoverCard`, `Inspector`, `MenuBar`, `MenuBarMenu`, `NodeContent`, `Popover`, `ReorderItem`, `Spotlight`, `StatusBar`, `Toolbar`, `Tree`;
  - **leaf widgets with options**, e.g. `Alert("Saved", tone: Tone::Success)`.
  The C functions take a raw `cui_*_opts` struct, so these have no plain form:
  `view!` always calls `name_with`, with default options when you write none.
  The option types are in this module (`cui::view::Alert`, ...), with string
  fields as `&str`.

Options that are more than plain values:

- **Values CUI writes back** (`selected`, `cursor`, `toggle`, `sort`, ...) are
  `Option<&mut T>`: `CommitGraph("log", &commits, selected: Some(&mut sel))`.
  Such options are passed as `&mut`, which `view!` does for you.
- **Arrays** are slices and carry their own length:
  `Fader("gain" => v, -60., 6., ticks: &[-12., 0.])`,
  `RadialGauge(.., bands: &[GaugeBand { to: 60., tone: Tone::Success }])`.
  Where C takes the length from the call (`Candlestick` labels, `Heatmap`
  row and column labels) a slice that is too short panics before the call;
  fixed lengths are in the type (`month_names: Some(&[..; 12])`).

**Per-row scopes** pass the row to their body, written as closure
parameters before it:

```no_run
# use cui::{view, Size, Ui, view::GridColumn};
# fn f(ui: &mut Ui, names: &[String], picked: &mut [bool]) {
# let cols = [GridColumn { title: Some("Name"), ..Default::default() }];
view! { ui,
    VirtualList("list", names.len() as i32, 28., Size::FILL) |i| {
        Text(&names[i as usize]);
    }
    DataGrid("grid", &cols, names.len() as i32, selected: Some(picked)) |row, clicked| {
        ui.data_grid_cell_text(&names[row as usize]);
    }
}
# }
```

`TreeTable(..) |node| { .. }` works the same way and its result (bind it with
`let`) is a [`TreeTableEvents`](crate::TreeTableEvents). `Table(..) { .. }`
is the closure form of `table_begin`. The borrows in a grid's or tree
table's options are written until the scope ends, which is why these are
closures. `HexEdit(id, &mut bytes, &mut sel)` edits bytes in place and
`EmojiPicker(id, recent: Some(&mut text))` returns the emoji picked.

Not covered: `TreeTable`'s type-ahead callback (use the raw
`tree_table_begin`), and the begin/end pairs the C header does not define as
scope macros (`kanban`, `node_editor`, `reorder`, `card_stack`, `swipe_item`,
`path_bar_menu`): call their `_begin` / `_end` methods from plain statements.

# Your own widgets

`view!` asks very little of a widget, so anything you write fits: an
extension trait on [`Ui`](crate::Ui) with a `name` method, optionally a
`name_with` method taking your own opts type called `Name`.

```no_run
use cui::{view, Ui};

#[derive(Default)]
struct StatCard { accent: bool }
#[derive(Default)]
struct Panel { padded: bool }

trait MyWidgets {
    fn stat_card(&mut self, title: &str, value: i32) -> bool;
    fn stat_card_with(&mut self, title: &str, value: i32, o: &StatCard) -> bool;
    fn panel_with(&mut self, title: &str, o: &Panel, f: impl FnOnce(&mut Ui<'_>));
    fn panel(&mut self, title: &str, f: impl FnOnce(&mut Ui<'_>));
}
# impl MyWidgets for Ui<'_> {
#     fn stat_card(&mut self, t: &str, v: i32) -> bool { self.stat_card_with(t, v, &StatCard::default()) }
#     fn stat_card_with(&mut self, t: &str, v: i32, o: &StatCard) -> bool { self.label(format!("{t}: {v}")); o.accent }
#     fn panel_with(&mut self, t: &str, _: &Panel, f: impl FnOnce(&mut Ui<'_>)) { self.heading(t, 3); f(self) }
#     fn panel(&mut self, t: &str, f: impl FnOnce(&mut Ui<'_>)) { self.panel_with(t, &Panel::default(), f) }
# }

fn screen(ui: &mut Ui) {
    view! { ui,
        // children go in a closure: declare such widgets once, here
        #![containers(Panel)]
        Panel("Stats", padded: true) {
            StatCard("Open", 3, accent: true) { /* runs when it returns true */ }
        }
    }
}
```

- `Name(args)` calls `ui.name(args)`; with `key: value` options it calls
  `ui.name_with(args, &Name { key: value, ..Default::default() })`.
- The opts type `Name` is looked up in `cui::view` first, then in the scope
  of the `view!` call, so your own types need no registration. If you define a
  type with the name of a built-in opts type (`Button`, `Text`, ...), the
  built-in one wins.
- A widget whose last parameter is the children closure is declared with
  `#![containers(Name, ..)]`, right after the handle. To avoid repeating it,
  wrap the macro once: `macro_rules! my_view { ($ui:ident, $($t:tt)*) =>
  { cui::view! { $ui, #![containers(Panel)] $($t)* } } }`.
- Option values are evaluated before the call, outside `cui::view`'s names,
  so they mean what they say in your code.

An unknown widget name is a rustc "no method named .." error on the name, an
unknown option is an error on the field, and a body on a widget that does
not return `bool` is a type error at the call. Use `cargo expand` to see the
generated code.

```compile_fail
use cui::{view, Ui};
fn f(ui: &mut Ui, text: &mut String) {
    // Input returns InputFlags: bind it with `let`
    view! { ui, Input("id" => text) { } }
}
```

```compile_fail
use cui::{view, Ui};
fn f(ui: &mut Ui) {
    // a scope widget needs a body
    view! { ui, Column(gap: 4.); }
}
```

```compile_fail
use cui::{view, Ui};
fn f(ui: &mut Ui) {
    // not a widget
    view! { ui, Frobnicate("x"); }
}
```
