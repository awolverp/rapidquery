use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;
use syn::parse::Parse;
use syn::parse::ParseStream;

struct PyClassInput {
    name: String,
    module: String,
    meta_args: Vec<syn::Meta>,
}

impl Parse for PyClassInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first: syn::LitStr = input.parse()?;
        let (module, name) = if input.peek(syn::Token![+]) {
            // Concat syntax
            //
            // "module" + "name"
            input.parse::<syn::Token![+]>()?;
            let class_name: syn::LitStr = input.parse()?;
            let module = first.value();
            let module = module.strip_suffix('.').unwrap_or(&module);
            if module.is_empty() || class_name.value().is_empty() {
                return Err(syn::Error::new(
                    class_name.span(),
                    "expected \"module\" + \"Class\" with non-empty module and class name",
                ));
            }
            (module.to_owned(), class_name.value())
        } else {
            // Literal string syntax
            //
            // "module.name"
            let value = first.value();
            let Some((module, name)) = value.rsplit_once('.') else {
                return Err(syn::Error::new(
                    first.span(),
                    "expected \"module.Class\" or \"module\" + \"Class\"; module is required",
                ));
            };
            if module.is_empty() || name.is_empty() || module.split('.').any(|s| s.is_empty()) {
                return Err(syn::Error::new(
                    first.span(),
                    "expected \"module.Class\" with non-empty module and class name",
                ));
            }
            (module.to_owned(), name.to_owned())
        };

        let mut meta_args = Vec::new();
        while input.peek(syn::token::Comma) {
            input.parse::<syn::token::Comma>()?;
            if input.is_empty() {
                break;
            }
            meta_args.push(input.parse()?);
        }

        Ok(Self {
            name,
            module,
            meta_args,
        })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let PyClassInput {
        name,
        module,
        meta_args,
    } = syn::parse2(attr)?;

    let item: ItemStruct = syn::parse2(item)?;

    // Fresh tokens, no input span, so rustc cannot substitute the dotted literal.
    let args: TokenStream = format!("module = \"{module}\", name = \"{name}\", immutable_type, frozen")
        .parse()
        .expect("pyclass args");

    let extra = if meta_args.is_empty() {
        quote! {}
    } else {
        quote! { , #(#meta_args),* }
    };

    Ok(quote! {
        #[pyo3::pyclass(#args #extra)]
        #item
    })
}
