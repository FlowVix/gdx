use either::Either::{self, Left, Right};
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    AngleBracketedGenericArguments, Expr, Ident, Pat, Token, Type, braced, bracketed,
    parenthesized, parse::Parse, parse_quote, punctuated::Punctuated, token,
};

pub struct ViewBody {
    pub views: Vec<ViewType>,
}

pub struct IfView {
    cond: Expr,
    body: ViewBody,
    else_expr: Option<Either<Box<IfView>, ViewBody>>,
}

#[allow(clippy::large_enum_variant)]
pub enum ViewType {
    Element {
        name: Ident,
        modifiers: Option<Punctuated<ElemModifier, Token![,]>>,
        children: Option<ViewBody>,
    },
    Component {
        name: Ident,
        args: Punctuated<Expr, Token![,]>,
        children: Option<ViewBody>,
    },
    Expr(Expr),
    For {
        pattern: Pat,
        iter: Expr,
        key: Expr,
        body: ViewBody,
    },
    If(IfView),
    Lens {
        moves: bool,
        args: Punctuated<(Pat, Type), Token![,]>,
        result: Punctuated<(Ident, Expr), Token![,]>,
        body: ViewBody,
    },
    Map {
        moves: bool,
        args: Punctuated<(Pat, Type), Token![,]>,
        result: Punctuated<Expr, Token![,]>,
        body: ViewBody,
    },
    Dyn(ViewBody),
    Proxy {
        generics: Option<AngleBracketedGenericArguments>,
        bind: Ident,
        cb: Expr,
        moves: bool,
        body: ViewBody,
    },
}

pub struct Event {
    typ: Ident,
    arg: Option<Ident>,
}

pub enum ElemModifier {
    Attr(Ident, Expr, bool),
    Event(Event, Expr),
    ThemeOverride {
        typ: Ident,
        name: Ident,
        value: Expr,
    },
}

impl Parse for Event {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let typ = input.parse()?;
        let arg = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { typ, arg })
    }
}

impl Parse for ElemModifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let typ = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Event(typ, value))
        } else if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let typ = input.parse()?;
            input.parse::<Token![:]>()?;
            let name = input.parse()?;
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::ThemeOverride { typ, name, value })
        } else {
            let name = input.parse()?;
            let build_only = input.peek(Token![@]);
            if build_only {
                input.parse::<Token![@]>()?;
            }
            input.parse::<Token![=]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Attr(name, value, build_only))
        }
    }
}

impl Parse for IfView {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![if]>()?;
        let cond = Expr::parse_without_eager_brace(input)?;
        let inner;
        braced!(inner in input);
        let body = inner.parse()?;
        let else_expr = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            Some(if input.peek(Token![if]) {
                Left(Box::new(input.parse()?))
            } else {
                let inner;
                braced!(inner in input);
                Right(inner.parse()?)
            })
        } else {
            None
        };

        Ok(Self {
            cond,
            body,
            else_expr,
        })
    }
}

impl Parse for ViewType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(token::Paren) {
            let inner;
            parenthesized!(inner in input);
            let expr = inner.parse()?;
            Ok(ViewType::Expr(expr))
        } else if input.peek(Token![for]) {
            input.parse::<Token![for]>()?;
            let pattern = Pat::parse_single(input)?;
            input.parse::<Token![in]>()?;
            let iter = input.parse()?;
            input.parse::<Token![=>]>()?;
            let key = Expr::parse_without_eager_brace(input)?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;

            Ok(ViewType::For {
                pattern,
                iter,
                key,
                body,
            })
        } else if input.peek(Token![if]) {
            Ok(ViewType::If(input.parse()?))
        } else if input.peek(Token![become]) {
            input.parse::<Token![become]>()?;

            let moves = input.peek(Token![move]);
            if moves {
                input.parse::<Token![move]>()?;
            }

            let inner;
            parenthesized!(inner in input);
            let args = Punctuated::parse_terminated_with(&inner, |inner| {
                let name = Pat::parse_single(inner)?;
                let typ = if inner.peek(Token![:]) {
                    inner.parse::<Token![:]>()?;
                    inner.parse()?
                } else {
                    parse_quote! { _ }
                };
                Ok((name, typ))
            })?;
            input.parse::<Token![=>]>()?;
            let inner;
            parenthesized!(inner in input);
            if inner.peek(Ident) && inner.peek2(Token![:]) {
                let result = Punctuated::parse_terminated_with(&inner, |inner| {
                    let name = inner.parse()?;
                    inner.parse::<Token![:]>()?;
                    let expr = inner.parse()?;
                    Ok((name, expr))
                })?;
                let inner;
                braced!(inner in input);
                let body = inner.parse()?;
                Ok(ViewType::Lens {
                    moves,
                    args,
                    result,
                    body,
                })
            } else {
                let result = Punctuated::parse_terminated(&inner)?;
                let inner;
                braced!(inner in input);
                let body = inner.parse()?;
                Ok(ViewType::Map {
                    moves,
                    args,
                    result,
                    body,
                })
            }
        } else if input.peek(Token![dyn]) {
            input.parse::<Token![dyn]>()?;
            let inner;
            braced!(inner in input);
            let body = inner.parse()?;
            Ok(ViewType::Dyn(body))
        } else if input.peek(Token![use]) {
            input.parse::<Token![use]>()?;
            let generics = if input.peek(Token![<]) || input.peek(Token![<<]) {
                Some(AngleBracketedGenericArguments::parse(input)?)
            } else {
                None
            };
            let bind = input.parse()?;
            input.parse::<Token![in]>()?;
            let cb = Expr::parse_without_eager_brace(input)?;

            let moves = input.peek(Token![move]);
            if moves {
                input.parse::<Token![move]>()?;
            }

            let inner;
            braced!(inner in input);
            let body = inner.parse()?;
            Ok(ViewType::Proxy {
                generics,
                bind,
                cb,
                moves,
                body,
            })
        } else {
            let name = input.parse()?;

            if input.peek(token::Paren) {
                let inner;
                parenthesized!(inner in input);
                let args = Punctuated::parse_terminated(&inner)?;
                let children = if input.peek(token::Brace) {
                    let inner;
                    braced!(inner in input);
                    Some(inner.parse()?)
                } else {
                    None
                };
                Ok(ViewType::Component {
                    name,
                    args,
                    children,
                })
            } else {
                let modifiers = if input.peek(token::Bracket) {
                    let inner;
                    bracketed!(inner in input);
                    Some(Punctuated::parse_terminated(&inner)?)
                } else {
                    None
                };
                let children = if input.peek(token::Brace) {
                    let inner;
                    braced!(inner in input);
                    Some(inner.parse()?)
                } else {
                    None
                };
                Ok(ViewType::Element {
                    name,
                    modifiers,
                    children,
                })
            }
        }
    }
}

impl Parse for ViewBody {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut views = vec![];
        while !input.is_empty() {
            views.push(input.parse()?);
        }
        Ok(ViewBody { views })
    }
}

impl IfView {
    pub fn gen_rust(&self) -> TokenStream {
        let body = self.body.gen_rust();
        let else_expr = match &self.else_expr {
            Some(Left(v)) => v.gen_rust(),
            Some(Right(v)) => v.gen_rust(),
            None => quote! { () },
        };
        let cond = &self.cond;
        quote! { if #cond { ::gdx::either::Either::Left(#body) } else { ::gdx::either::Either::Right(#else_expr) } }
    }
}

impl ViewType {
    pub fn gen_rust(&self) -> TokenStream {
        match self {
            ViewType::Element {
                name: typ,
                modifiers,
                children,
            } => {
                let mut out = quote! { ::gdx::el::<#typ>() };

                if let Some(children) = children {
                    let inner = children.gen_rust();
                    out.extend(quote! { .children(#inner) });
                }
                for m in modifiers.iter().flatten() {
                    match m {
                        ElemModifier::Attr(ident, expr, build_only) => out.extend(
                            quote! { .attr::<_, _, #build_only>(stringify!(#ident), #expr) },
                        ),
                        ElemModifier::Event(event, expr) => {
                            let func_name =
                                Ident::new(&format!("on_{}", event.typ), event.typ.span());
                            let arg = event.arg.as_ref().map(|v| quote! { stringify!(#v), });
                            out.extend(quote! { .#func_name(#arg #expr) })
                        }
                        ElemModifier::ThemeOverride { typ, name, value } => {
                            let typ = Ident::new(
                                &format!("ThemeOverride{}", typ.to_string().to_upper_camel_case()),
                                typ.span(),
                            );
                            out.extend(
                                        quote! { .theme_override::<::gdx::#typ, _>(stringify!(#name), #value) },
                                    )
                        }
                    }
                }
                out
            }
            ViewType::Component {
                name,
                args,
                children,
            } => {
                let children = children.as_ref().map(|v| v.gen_rust());
                let children = if let Some(children) = children {
                    quote! { , #children }
                } else {
                    quote! {}
                };
                quote! { #name(#args #children) }
            }
            ViewType::Expr(expr) => quote! { #expr },
            ViewType::For {
                pattern,
                iter,
                key,
                body,
            } => {
                let body = body.gen_rust();
                quote! { (#iter).into_iter().map(|#pattern| (#key, #body) ).collect::<Vec<_>>() }
            }
            ViewType::If(if_view) => if_view.gen_rust(),
            ViewType::Lens {
                moves,
                args,
                result,
                body,
            } => {
                let (arg_names, arg_types): (Vec<_>, Vec<_>) = args.iter().cloned().unzip();
                let (result_names, result_exprs): (Vec<_>, Vec<_>) = result.iter().cloned().unzip();
                let body = body.gen_rust();

                let moves = if *moves {
                    quote! { move }
                } else {
                    quote! {}
                };

                quote! { ::gdx::lens(#moves |(#(#arg_names,)*): &mut (#(#arg_types,)*)| (#(#result_exprs,)*), |(#(#result_names,)*)| #body) }
            }
            ViewType::Map {
                moves,
                args,
                result,
                body,
            } => {
                let (arg_names, arg_types): (Vec<_>, Vec<_>) = args.iter().cloned().unzip();
                let result_expr = result.iter();
                let body = body.gen_rust();

                let moves = if *moves {
                    quote! { move }
                } else {
                    quote! {}
                };

                quote! { ::gdx::map(#moves |(#(#arg_names,)*): &mut (#(#arg_types,)*)| (#(#result_expr,)*), #body) }
            }
            ViewType::Dyn(view_body) => {
                let body = view_body.gen_rust();
                quote! { ( Box::new(#body) as Box<dyn ::gdx::AnyView<_>> ) }
            }
            ViewType::Proxy {
                generics,
                bind,
                cb,
                moves,
                body,
            } => {
                let generics = generics
                    .as_ref()
                    .map(|g| {
                        let args = &g.args;
                        quote! { ::<_, #args, _, _, _> }
                    })
                    .unwrap_or(quote! {});
                let body = body.gen_rust();
                let moves = if *moves {
                    quote! { move }
                } else {
                    quote! {}
                };
                quote! { ::gdx::proxy #generics (#cb, #moves |#bind| #body) }
            }
        }
    }
}

impl ViewBody {
    pub fn gen_rust(&self) -> TokenStream {
        let views = self.views.iter().map(|v| v.gen_rust());
        if self.views.len() == 1 {
            quote! { #(#views),* }
        } else {
            quote! { ( #(#views),* ) }
        }
    }
}
