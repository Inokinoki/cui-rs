//! The `view!` macro for [`cui-rs`](https://docs.rs/cui-rs): a declarative
//! front end that expands, in place, to the nested closures and opts
//! structs you would otherwise write by hand. There is no runtime: the
//! expansion is the same call sequence, and scope panic-safety stays with
//! `cui`'s own scope guards. Use it through `cui::view!`.

mod expand;
mod parse;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::Parser;

/// Describes one frame of UI declaratively.
///
/// ```ignore
/// cui::view! { ui,
///     Column(pad: 32., gap: 16., width: Size::FILL) {
///         Heading("Hello, CUI", 2);
///         Switch("Notifications" => notifications);
///         Row(gap: 8.) {
///             Button("Click me", variant: ButtonVariant::Primary) { clicks += 1; }
///             Text(format!("Clicked {clicks} times"));
///         }
///     }
/// }
/// ```
///
/// The first token is the name of the `Ui` handle (an identifier). Then:
///
/// - `Name(args) { block }` calls `ui.name(args)`. `Name` is the PascalCase
///   of the method (`IconButton` → `icon_button`, `Box` → `box_`).
/// - Arguments are the method's own, in order. `a => b` means `a, &mut b`.
///   `key: value` entries go into the widget's opts struct
///   (`cui::view::Name { key: value, ..Default::default() }`), and the call
///   becomes `ui.name_with(.., &opts)`.
/// - A scope widget (`Column`, `Row`, `Box`, `Card`, `Scroll`, `Dialog`,
///   `DialogFooter`, `Menu`, `UserMenu`, `ContextMenu`, `Split`, `Sheet`,
///   `Reasoning`) needs a `{ ... }` body of nodes: the closure.
/// - Any other widget with a `{ ... }` body expands to `if call { body }`:
///   the body runs when the widget reports a click or change.
/// - `let x = Name(..);` binds the result (`let typed = Input(..)`,
///   `let clicked = Card(..) { .. }`).
/// - `if`/`else`, `for`, `while`, `loop` and `match` have node lists as
///   bodies; every other Rust statement is copied as written.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let mut errs = Vec::new();
    let mut ui: Option<Ident> = None;
    let mut list = Vec::new();
    let parser = |input: syn::parse::ParseStream| -> syn::Result<()> {
        let id = input.parse::<Ident>().map_err(|e| {
            syn::Error::new(e.span(), "view! starts with the name of the Ui handle, e.g. `view! { ui, ... }`")
        })?;
        input.parse::<syn::Token![,]>()?;
        ui = Some(id);
        list = parse::parse_nodes(input, &mut errs);
        Ok(())
    };
    if let Err(e) = parser.parse(input) {
        errs.push(e);
    }
    let body = match &ui {
        Some(ui) => {
            let mut cx = expand::Ctx { ui, errs: &mut errs };
            expand::nodes(&mut cx, &list)
        }
        None => quote!(),
    };
    let errors = errs.iter().map(|e| e.to_compile_error());
    quote!({ #body #(#errors)* }).into()
}
