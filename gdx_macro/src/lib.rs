mod view;

use syn::parse_macro_input;

use crate::view::ViewBody;

/// view! { ... }
#[proc_macro]
pub fn view(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let b = parse_macro_input!(item as ViewBody);

    b.gen_rust().into()
}
