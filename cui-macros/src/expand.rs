//! Turns parsed nodes into the call sequence you would write by hand.

use crate::parse::{Arm, Else, If, Node, Widget};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Error;

#[derive(Clone, Copy, PartialEq)]
enum Opts {
    /// The method always takes an opts struct (`column`, `sheet`, ...).
    Always,
    /// `name(..)` without options, `name_with(.., &opts, ..)` with.
    Optional,
    /// No opts struct at all.
    No,
}

/// The scope-style widgets: their trailing block becomes a closure. Every
/// other widget with a trailing block becomes `if call { block }`.
fn builtin_container(name: &str) -> Option<(&'static str, Opts)> {
    Some(match name {
        "Column" => ("column", Opts::Always),
        "Row" => ("row", Opts::Always),
        "Box" => ("box_", Opts::Always),
        "Card" => ("card", Opts::Always),
        "Scroll" => ("scroll", Opts::Always),
        "Split" => ("split", Opts::Always),
        "Sheet" => ("sheet", Opts::Always),
        "Reasoning" => ("reasoning", Opts::Always),
        "Dialog" => ("dialog", Opts::Optional),
        "Menu" => ("menu", Opts::Optional),
        "UserMenu" => ("user_menu", Opts::Optional),
        "DialogFooter" => ("dialog_footer", Opts::No),
        "Table" => ("table", Opts::Always),
        "VirtualList" => ("virtual_list", Opts::No),
        "DataGrid" => ("data_grid", Opts::Optional),
        "TreeTable" => ("tree_table", Opts::Optional),
        "ContextMenu" => ("context_menu", Opts::No),
        _ => return None,
    })
}

/// Hand-written leaf widgets whose method always takes options and has no
/// `_with` twin: `Typewriter(..)` is `ui.typewriter(.., &opts)`.
const PLAIN_OPTS: [&str; 13] = [
    "Typewriter", "NumberRoll", "WordRotate", "ToolCall", "AgentPlan", "AgentStatus",
    "PromptSuggestions", "ModelPicker", "ContextMeter", "DotChart", "ChatComposer", "ResponseActions", "EventTrack",
];

/// Hand-written leaf widgets with only a `_with` form.
const WITH_ONLY: [&str; 1] = ["EmojiPicker"];

/// Hand-written widgets whose options hold `&mut` borrows.
const HAND_MUT: [&str; 7] =
    ["Table", "DataGrid", "TreeTable", "ChatComposer", "ResponseActions", "EmojiPicker", "EventTrack"];

fn mut_opts(name: &str) -> bool {
    HAND_MUT.contains(&name) || crate::generated::MUT_OPTS.contains(&name)
}

/// Widgets that return a flags type rather than `bool`, so a `{ ... }` body
/// cannot run on their result. Returns the flags type, for the error message.
fn flags_type(name: &str) -> Option<&'static str> {
    match name {
        "Input" | "Textarea" | "Search" => Some("InputFlags"),
        "ChatComposer" => Some("ComposerFlags"),
        "ResponseActions" => Some("ResponseActionsFlags"),
        _ => None,
    }
}

fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

pub struct Ctx<'a> {
    pub ui: &'a Ident,
    pub errs: &'a mut Vec<Error>,
    /// Names declared with `#![containers(..)]`.
    pub custom: &'a [Ident],
}

/// The method name and opts shape of a scope widget: a built-in one, or one
/// declared in the header (which follows the `name` / `name_with` convention).
fn container_of(cx: &Ctx, name: &str) -> Option<(String, Opts)> {
    if let Some((m, k)) = builtin_container(name) {
        return Some((m.to_string(), k));
    }
    let declared = cx.custom.iter().any(|c| c == name);
    (declared || crate::generated::SCOPES.contains(&name)).then(|| (snake(name), Opts::Optional))
}

/// Wraps `build(opts)` for a widget that has `key: value` options. The values
/// are evaluated first, outside the glob import, so user expressions never
/// see `cui::view`'s names; then the opts struct is built where `Name`
/// resolves to the built-in opts type of that name, or else to a type of that
/// name in the caller's scope.
fn with_opts(w: &Widget, force: bool, build: impl FnOnce(Option<TokenStream>) -> TokenStream) -> TokenStream {
    if w.props.is_empty() && !force {
        return build(None);
    }
    let name = &w.name;
    let keys = w.props.iter().map(|(k, _)| k);
    let vals = w.props.iter().map(|(_, v)| v);
    let ids: Vec<Ident> = w
        .props
        .iter()
        .enumerate()
        .map(|(i, (_, v))| Ident::new(&format!("__cui_p{i}"), syn::spanned::Spanned::span(v)))
        .collect();
    let is_mut = mut_opts(&name.to_string());
    let (opts, mutability) = if is_mut { (quote!(&mut __cui_opts), quote!(mut)) } else { (quote!(&__cui_opts), quote!()) };
    let binding = quote! {
        #[allow(clippy::needless_update)]
        let #mutability __cui_opts = {
            #[allow(unused_imports)]
            use ::cui::view::*;
            #name { #(#keys: #ids,)* ..::core::default::Default::default() }
        };
    };
    let call = build(Some(opts));
    quote!(match (#(#vals,)*) { (#(#ids,)*) => { #binding #call } })
}

pub fn nodes(cx: &mut Ctx, list: &[Node]) -> TokenStream {
    let parts: Vec<TokenStream> = list.iter().map(|n| node(cx, n)).collect();
    quote!(#(#parts)*)
}

fn node(cx: &mut Ctx, n: &Node) -> TokenStream {
    match n {
        Node::Widget(w) => {
            let call = widget(cx, w, false);
            quote!(#call)
        }
        Node::Let { pat, ty, widget: w } => {
            let call = widget(cx, w, true);
            match ty {
                Some(ty) => quote!(let #pat: #ty = #call;),
                None => quote!(let #pat = #call;),
            }
        }
        Node::If(i) => if_node(cx, i),
        Node::For { pat, iter, body } => {
            let body = nodes(cx, body);
            quote!(for #pat in #iter { #body })
        }
        Node::While { cond, body } => {
            let body = nodes(cx, body);
            quote!(while #cond { #body })
        }
        Node::Loop(body) => {
            let body = nodes(cx, body);
            quote!(loop { #body })
        }
        Node::Match { scrutinee, arms } => {
            let arms: Vec<TokenStream> = arms.iter().map(|a| arm(cx, a)).collect();
            quote!(match #scrutinee { #(#arms)* })
        }
        Node::Raw(t) => t.clone(),
    }
}

fn if_node(cx: &mut Ctx, i: &If) -> TokenStream {
    let cond = &i.cond;
    let then = nodes(cx, &i.then);
    let otherwise = match &i.otherwise {
        None => quote!(),
        Some(Else::If(e)) => {
            let e = if_node(cx, e);
            quote!(else #e)
        }
        Some(Else::Block(b)) => {
            let b = nodes(cx, b);
            quote!(else { #b })
        }
    };
    quote!(if #cond { #then } #otherwise)
}

fn arm(cx: &mut Ctx, a: &Arm) -> TokenStream {
    let pat = &a.pat;
    let guard = a.guard.as_ref().map(|g| quote!(if #g));
    let body = nodes(cx, &a.body);
    quote!(#pat #guard => { #body })
}

/// One widget. `as_value`: the call is the right-hand side of a `let`, so it
/// is an expression and ends without a `;`.
fn widget(cx: &mut Ctx, w: &Widget, as_value: bool) -> TokenStream {
    if w.broken {
        return quote!(());
    }
    let ui = cx.ui;
    let name = &w.name;
    let name_s = name.to_string();
    let span = name.span();
    let has_props = !w.props.is_empty();
    let positional = &w.positional;

    let ident = |s: &str| Ident::new(s, span);

    if let Some((method, kind)) = container_of(cx, &name_s) {
        let Some(body) = &w.block else {
            cx.errs.push(Error::new(span, format!("`{name_s}` is a container: give it a `{{ ... }}` body")));
            return quote!();
        };
        if kind == Opts::No && has_props {
            cx.errs.push(Error::new(span, format!("`{name_s}` takes no options")));
            return quote!();
        }
        let takes_opts = kind == Opts::Always || has_props;
        let m = if kind == Opts::Optional && has_props { ident(&format!("{method}_with")) } else { ident(&method) };
        let children = nodes(cx, body);
        let ui = cx.ui;
        let params = &w.params;
        let closure = quote!(|#ui #(, #params)*| { #children });
        let call = if takes_opts {
            // `Always` containers take opts even when no option was written
            with_opts(w, true, |o| quote!(#ui.#m(#(#positional,)* #o, #closure)))
        } else {
            quote!(#ui.#m(#(#positional,)* #closure))
        };
        return if as_value { call } else { quote!(#call;) };
    }

    let base = snake(&name_s);
    if !w.params.is_empty() {
        cx.errs.push(Error::new(span, format!("`{name_s}` is not a container, so its body takes no `|parameters|`")));
        return quote!();
    }
    // widgets with no option-less form: generated ones go through
    // `name_with`, a few hand-written ones take the options in `name` itself
    let plain = PLAIN_OPTS.contains(&name_s.as_str());
    let forced = plain || WITH_ONLY.contains(&name_s.as_str()) || crate::generated::LEAF_OPTS.contains(&name_s.as_str());
    let m = ident(&if (has_props || forced) && !plain { format!("{base}_with") } else { base });
    // `Name("a", 1, k: v)` -> `ui.name_with("a", 1, &Opts { k: v, .. })`
    let call = with_opts(w, forced, |o| quote!(#ui.#m(#(#positional,)* #o)));

    match &w.block {
        None => {
            if as_value {
                call
            } else {
                quote!(#call;)
            }
        }
        Some(body) => {
            if as_value {
                cx.errs.push(Error::new(span, format!("`{name_s}` is not a container, so it takes no `{{ ... }}` body here")));
                return quote!();
            }
            if let Some(ty) = flags_type(&name_s) {
                cx.errs.push(Error::new(
                    span,
                    format!("`{name_s}` returns {ty}, not bool: bind it with `let flags = {name_s}(..);` and use {ty}'s methods"),
                ));
                return quote!();
            }
            let children = nodes(cx, body);
            quote!(if #call { #children })
        }
    }
}
