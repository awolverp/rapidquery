use proc_macro::TokenStream;
use quote::quote;
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

/// Defines a pyo3 class with a required module path.
///
/// ```ignore
/// #[pyclass("foo.Bar")]
/// struct Bar;
/// ```
///
/// ```ignore
/// #[pyclass("foo" + "Bar", subclass)]
/// struct Bar;
/// ```
#[proc_macro_attribute]
pub fn pyclass(attr: TokenStream, item: TokenStream) -> TokenStream {
    let PyClassInput {
        name,
        module,
        meta_args,
    } = syn::parse_macro_input!(attr as PyClassInput);
    let item = syn::parse_macro_input!(item as syn::ItemStruct);

    // Fresh tokens, no input span, so rustc cannot substitute the dotted literal.
    let args: proc_macro2::TokenStream =
        format!("module = \"{module}\", name = \"{name}\", immutable_type, frozen")
            .parse()
            .expect("pyclass args");

    let extra = if meta_args.is_empty() {
        quote! {}
    } else {
        quote! { , #(#meta_args),* }
    };

    quote! {
        #[pyo3::pyclass(#args #extra)]
        #item
    }
    .into()
}

struct PyErrInput {
    return_err_enum: bool,
    name: syn::Ident,
    msg_or_fmt: Option<syn::Expr>,
    format_args: Vec<syn::Expr>,
}

impl Parse for PyErrInput {
    /// Acceptable formats:
    /// - `#name`
    /// - `Err(#name)`
    /// - `Err(#name), #msg`
    /// - `Err(#name), #fmt, #args...`
    /// - `#name, #msg`
    /// - `#name, #fmt, #args...`
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let (wrap_in_err, name) = if ident == "Err" && input.peek(syn::token::Paren) {
            let inner;
            syn::parenthesized!(inner in input);
            let name: syn::Ident = inner.parse()?;
            inner.parse::<syn::parse::Nothing>()?;
            (true, name)
        } else {
            (false, ident)
        };

        let mut slf = Self {
            return_err_enum: wrap_in_err,
            name,
            msg_or_fmt: None,
            format_args: vec![],
        };

        if input.is_empty() {
            return Ok(slf);
        }

        input.parse::<syn::token::Comma>()?;

        slf.msg_or_fmt = Some(input.parse()?);

        while input.peek(syn::token::Comma) {
            input.parse::<syn::token::Comma>()?;

            if input.is_empty() {
                break;
            }

            slf.format_args.push(input.parse()?);
        }

        Ok(slf)
    }
}

/// Builds a [`PyErr`], or `Err(PyErr)` when the name is wrapped in `Err`.
///
/// ```ignore
/// pyerr!(PyTypeError, "bad {}", x)
/// pyerr!(Err(PyValueError))
/// ```
#[proc_macro]
pub fn pyerr(item: TokenStream) -> TokenStream {
    let PyErrInput {
        return_err_enum,
        name,
        msg_or_fmt,
        format_args,
    } = syn::parse_macro_input!(item as PyErrInput);

    let constructed = match (msg_or_fmt, format_args.is_empty()) {
        (Some(a), false) => {
            quote! { ::pyo3::exceptions::#name::new_err(format!(#a, #(#format_args),*)) }
        }
        (Some(a), true) => {
            quote! { ::pyo3::exceptions::#name::new_err(#a) }
        }
        (None, _) => quote! { ::pyo3::exceptions::#name::new_err(()) },
    };

    if return_err_enum {
        quote! { Err(#constructed) }.into()
    } else {
        constructed.into()
    }
}
