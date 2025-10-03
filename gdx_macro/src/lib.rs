mod view;

use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Index, parse_macro_input};

use crate::view::ViewBody;

/// view! { ... }
#[proc_macro]
pub fn view(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let b = parse_macro_input!(item as ViewBody);

    b.gen_rust().into()
}

/// map_to_tuples! { ... }
#[proc_macro]
pub fn impl_arg_tuple(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut out = quote! {};
    for n in 0..10 {
        let idents = (0..n)
            .map(|v| Ident::new(&format!("V{}", v), Span::call_site()))
            .collect::<Vec<_>>();
        let ret_idents = (0..n)
            .map(|v| Ident::new(&format!("r_{}", v), Span::call_site()))
            .collect::<Vec<_>>();
        let value_idents = (0..n)
            .map(|v| Ident::new(&format!("v_{}", v), Span::call_site()))
            .collect::<Vec<_>>();
        let indices = (0..n).map(|v| Index::from(v)).collect::<Vec<_>>();

        let mut inner = quote! {
            let mut v = ( #(#value_idents,)* );
            ret = Some(f(&mut v));
            #( #ret_idents = Some(v.#indices); )*
        };
        for i in (0..n).rev() {
            let ret_ident = &ret_idents[i];
            let value_ident = &value_idents[i];
            let index = &indices[i];
            inner = quote! {
                replace_with_or_abort(r.#index, |#value_ident| {
                    #inner
                    #ret_ident.unwrap()
                });
            }
        }

        out.extend(quote! {
            impl< #(#idents,)* > ArgTuple for ( #(#idents,)* ) {
                type Ref<'a> = ( #(&'a mut #idents,)* ) where #(#idents: 'a,)*;

                fn extract_call<'a, R>(r: Self::Ref<'a>, f: impl FnOnce(&mut Self) -> R) -> R {
                    let mut ret = None;
                    #( let mut #ret_idents = None; )*
                    #inner
                    ret.unwrap()
                }
            }
        });
    }

    out.into()
}
