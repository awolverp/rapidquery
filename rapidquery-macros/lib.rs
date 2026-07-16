use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, ItemStruct, LitStr, Meta, Token};

struct PyclassArgs {
    name: LitStr,
    extra: Vec<Meta>,
}

impl Parse for PyclassArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;

        let mut extra = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            extra.push(input.parse::<Meta>()?);
        }

        Ok(PyclassArgs { name, extra })
    }
}

#[proc_macro_attribute]
pub fn pyclass_shortcut(attr: TokenStream, item: TokenStream) -> TokenStream {
    let PyclassArgs { name, extra } = parse_macro_input!(attr as PyclassArgs);
    let item = parse_macro_input!(item as ItemStruct);

    let extra_tokens = if extra.is_empty() {
        quote! {}
    } else {
        quote! { , #(#extra),* }
    };

    quote! {
        #[pyo3::pyclass(module = "rapidquery._native", name = #name, immutable_type, frozen #extra_tokens)]
        #item
    }
    .into()
}
