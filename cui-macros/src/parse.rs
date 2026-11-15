//! The `view!` grammar. A node list is a sequence of widgets, control flow
//! whose bodies are node lists again, and plain Rust statements that pass
//! through untouched.

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::{braced, parenthesized, token, Error, Expr, Pat, Result, Stmt, Token, Type};

pub enum Node {
    /// `Name(args) { children };`
    Widget(Widget),
    /// `let pat: ty = Name(args) { children };`
    Let { pat: Pat, ty: Option<Type>, widget: Widget },
    If(If),
    For { pat: Pat, iter: Expr, body: Vec<Node> },
    While { cond: Expr, body: Vec<Node> },
    Loop(Vec<Node>),
    Match { scrutinee: Expr, arms: Vec<Arm> },
    /// Anything else, copied as written.
    Raw(TokenStream),
}

pub struct Widget {
    pub name: Ident,
    /// Positional arguments in call order; `a => b` has already become
    /// `a, &mut b`.
    pub positional: Vec<Expr>,
    /// `key: value` options for the widget's opts struct.
    pub props: Vec<(Ident, Expr)>,
    /// `|row, clicked|` before the body: extra closure parameters of a scope
    /// widget that runs its body once per row.
    pub params: Vec<TokenStream>,
    pub block: Option<Vec<Node>>,
    /// The arguments did not parse (already reported): expand to nothing.
    pub broken: bool,
}

pub struct If {
    pub cond: Expr,
    pub then: Vec<Node>,
    pub otherwise: Option<Else>,
}

pub enum Else {
    If(Box<If>),
    Block(Vec<Node>),
}

pub struct Arm {
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Vec<Node>,
}

/// Names that look like widgets but are everyday Rust.
const NOT_WIDGETS: [&str; 4] = ["Some", "Ok", "Err", "Self"];

fn widget_start(input: ParseStream) -> bool {
    match input.cursor().ident() {
        Some((id, rest)) => {
            let s = id.to_string();
            s.starts_with(|c: char| c.is_ascii_uppercase())
                && !NOT_WIDGETS.contains(&s.as_str())
                && rest.group(proc_macro2::Delimiter::Parenthesis).is_some()
        }
        None => false,
    }
}

pub fn parse_nodes(input: ParseStream, errs: &mut Vec<Error>) -> Vec<Node> {
    let mut nodes = Vec::new();
    while !input.is_empty() {
        match parse_node(input, errs) {
            Ok(n) => nodes.push(n),
            Err(e) => {
                errs.push(e);
                // the position is unknown: drop the rest of this list
                let _ = input.parse::<TokenStream>();
                break;
            }
        }
    }
    nodes
}

fn block(input: ParseStream, errs: &mut Vec<Error>) -> Result<Vec<Node>> {
    let content;
    braced!(content in input);
    Ok(parse_nodes(&content, errs))
}

fn semi(input: ParseStream) -> Result<()> {
    let _ = input.parse::<Option<Token![;]>>()?;
    Ok(())
}

fn parse_node(input: ParseStream, errs: &mut Vec<Error>) -> Result<Node> {
    if input.peek(Token![if]) {
        let n = Node::If(parse_if(input, errs)?);
        semi(input)?;
        Ok(n)
    } else if input.peek(Token![for]) {
        input.parse::<Token![for]>()?;
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        input.parse::<Token![in]>()?;
        let iter = Expr::parse_without_eager_brace(input)?;
        let body = block(input, errs)?;
        semi(input)?;
        Ok(Node::For { pat, iter, body })
    } else if input.peek(Token![while]) {
        input.parse::<Token![while]>()?;
        let cond = Expr::parse_without_eager_brace(input)?;
        let body = block(input, errs)?;
        semi(input)?;
        Ok(Node::While { cond, body })
    } else if input.peek(Token![loop]) {
        input.parse::<Token![loop]>()?;
        let body = block(input, errs)?;
        semi(input)?;
        Ok(Node::Loop(body))
    } else if input.peek(Token![match]) {
        input.parse::<Token![match]>()?;
        let scrutinee = Expr::parse_without_eager_brace(input)?;
        let content;
        braced!(content in input);
        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(parse_arm(&content, errs)?);
        }
        semi(input)?;
        Ok(Node::Match { scrutinee, arms })
    } else if input.peek(Token![let]) && let_widget_ahead(input) {
        input.parse::<Token![let]>()?;
        let pat = Pat::parse_single(input)?;
        let ty = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse::<Type>()?)
        } else {
            None
        };
        input.parse::<Token![=]>()?;
        let widget = parse_widget(input, errs)?;
        Ok(Node::Let { pat, ty, widget })
    } else if widget_start(input) {
        Ok(Node::Widget(parse_widget(input, errs)?))
    } else {
        let stmt: Stmt = input.parse()?;
        Ok(Node::Raw(quote!(#stmt)))
    }
}

/// Looks past `let pat [: ty] =` for `Name(..)` followed by `;` or a block,
/// so `let x = Some(1);` stays an ordinary statement.
fn let_widget_ahead(input: ParseStream) -> bool {
    let fork = input.fork();
    let shape = (|| -> Result<bool> {
        fork.parse::<Token![let]>()?;
        Pat::parse_single(&fork)?;
        if fork.peek(Token![:]) {
            fork.parse::<Token![:]>()?;
            fork.parse::<Type>()?;
        }
        fork.parse::<Token![=]>()?;
        if !widget_start(&fork) {
            return Ok(false);
        }
        fork.call(Ident::parse_any)?;
        let args;
        parenthesized!(args in fork);
        Ok(fork.peek(Token![;]) || fork.peek(token::Brace) || fork.peek(Token![|]))
    })();
    shape.unwrap_or(false)
}

fn parse_if(input: ParseStream, errs: &mut Vec<Error>) -> Result<If> {
    input.parse::<Token![if]>()?;
    let cond = Expr::parse_without_eager_brace(input)?;
    let then = block(input, errs)?;
    let otherwise = if input.peek(Token![else]) {
        input.parse::<Token![else]>()?;
        if input.peek(Token![if]) {
            Some(Else::If(Box::new(parse_if(input, errs)?)))
        } else {
            Some(Else::Block(block(input, errs)?))
        }
    } else {
        None
    };
    Ok(If { cond, then, otherwise })
}

fn parse_arm(input: ParseStream, errs: &mut Vec<Error>) -> Result<Arm> {
    let pat = Pat::parse_multi_with_leading_vert(input)?;
    let guard = if input.peek(Token![if]) {
        input.parse::<Token![if]>()?;
        Some(input.parse::<Expr>()?)
    } else {
        None
    };
    input.parse::<Token![=>]>()?;
    let body = if input.peek(token::Brace) {
        block(input, errs)?
    } else if widget_start(input) {
        vec![Node::Widget(parse_widget(input, errs)?)]
    } else {
        let e: Expr = input.parse()?;
        vec![Node::Raw(quote!(#e;))]
    };
    let _ = input.parse::<Option<Token![,]>>()?;
    Ok(Arm { pat, guard, body })
}

/// `Name(args)` then an optional `{ children }`, then an optional `;`.
fn parse_widget(input: ParseStream, errs: &mut Vec<Error>) -> Result<Widget> {
    let name: Ident = input.call(Ident::parse_any)?;
    let content;
    parenthesized!(content in input);
    let mut positional = Vec::new();
    let mut props = Vec::new();
    let mut broken = false;
    if let Err(e) = parse_args(&content, &mut positional, &mut props) {
        broken = true;
        errs.push(e);
        let _ = content.parse::<TokenStream>();
    }
    let mut params = Vec::new();
    if input.peek(Token![|]) {
        input.parse::<Token![|]>()?;
        while !input.peek(Token![|]) {
            let pat = Pat::parse_single(input)?;
            params.push(if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                let ty: Type = input.parse()?;
                quote!(#pat: #ty)
            } else {
                quote!(#pat)
            });
            if !input.peek(Token![|]) {
                input.parse::<Token![,]>()?;
            }
        }
        input.parse::<Token![|]>()?;
        if !input.peek(token::Brace) {
            return Err(input.error("expected the `{ ... }` body after its `|parameters|`"));
        }
    }
    let block = if input.peek(token::Brace) { Some(block(input, errs)?) } else { None };
    semi(input)?;
    Ok(Widget { name, positional, props, params, block, broken })
}

fn parse_args(
    input: ParseStream,
    positional: &mut Vec<Expr>,
    props: &mut Vec<(Ident, Expr)>,
) -> Result<()> {
    while !input.is_empty() {
        if input.peek(Ident::peek_any) && input.peek2(Token![:]) && !input.peek2(Token![::]) {
            let key = input.call(Ident::parse_any)?;
            input.parse::<Token![:]>()?;
            props.push((key, input.parse()?));
        } else {
            let value: Expr = input.parse()?;
            positional.push(value);
            if input.peek(Token![=>]) {
                // `a => b` is `a, &mut b`
                input.parse::<Token![=>]>()?;
                let target: Expr = input.parse()?;
                positional.push(syn::parse_quote!(&mut #target));
            }
        }
        if input.is_empty() {
            break;
        }
        input.parse::<Token![,]>()?;
    }
    Ok(())
}
