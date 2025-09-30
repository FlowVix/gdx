use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, Ident, Pat, Token, Type, braced, bracketed, parenthesized, parse::Parse,
    punctuated::Punctuated, token,
};

pub struct ViewBody {
    pub views: Vec<ViewType>,
}

pub enum ViewType {
    Element {
        typ: Ident,
        modifiers: Option<Punctuated<ElemModifier, Token![,]>>,
        children: Option<ViewBody>,
    },
    Expr(Expr),
    For {
        pattern: Pat,
        iter: Expr,
        key: Expr,
        body: ViewBody,
    },
}

pub enum ElemModifier {
    Attr(Ident, Expr),
    Event(Ident, Expr),
}

impl Parse for ElemModifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let name = input.parse()?;
            input.parse::<Token![:]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Event(name, value))
        } else {
            let name = input.parse()?;
            input.parse::<Token![:]>()?;
            let value = input.parse()?;
            Ok(ElemModifier::Attr(name, value))
        }
    }
}

impl Parse for ViewType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let expr = input.parse()?;
            input.parse::<Token![,]>()?;
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
        } else {
            let typ = input.parse()?;
            let modifiers = if input.peek(token::Bracket) {
                let inner;
                bracketed!(inner in input);
                Some(Punctuated::parse_terminated(&inner)?)
            } else {
                None
            };
            let mut children = None;
            if input.peek(token::Brace) {
                let inner;
                braced!(inner in input);
                children = Some(inner.parse()?);
            }
            Ok(ViewType::Element {
                typ,
                modifiers,
                children,
            })
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

impl ViewType {
    pub fn gen_rust(&self) -> TokenStream {
        match self {
            ViewType::Element {
                typ,
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
                        ElemModifier::Attr(ident, expr) => {
                            out.extend(quote! { .attr(stringify!(#ident), #expr) })
                        }
                        ElemModifier::Event(ident, expr) => {
                            out.extend(quote! { .on(stringify!(#ident), #expr) })
                        }
                    }
                }
                out
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
        }
    }
}

impl ViewBody {
    pub fn gen_rust(&self) -> TokenStream {
        let views = self.views.iter().map(|v| v.gen_rust());
        quote! { ( #(#views),* ) }
    }
}
