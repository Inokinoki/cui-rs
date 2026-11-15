
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
