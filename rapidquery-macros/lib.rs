use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

struct PyClassInput {
    name: syn::LitStr,
    module: syn::LitStr,
    meta_args: Vec<syn::Meta>,
}

impl Parse for PyClassInput {
    /// Accepts `#name, args...` format.
    ///
    /// `module = "..."` overrides the default `rapidquery._core` and is not
    /// forwarded as a duplicate `#[pyclass]` argument.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::LitStr = input.parse()?;
        let mut module = syn::LitStr::new("rapidquery._core", name.span());
        let mut meta_args = Vec::new();

        while input.peek(syn::token::Comma) {
            input.parse::<syn::token::Comma>()?;

            if input.is_empty() {
                break;
            }

            let meta = input.parse::<syn::Meta>()?;
            match meta {
                syn::Meta::NameValue(nv) if nv.path.is_ident("module") => match nv.value {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) => module = s.clone(),
                    _ => return Err(syn::Error::new(nv.span(), "module expects literal string")),
                },
                _ => meta_args.push(meta),
            }
        }

        Ok(PyClassInput {
            name,
            module,
            meta_args,
        })
    }
}

/// Shortcut for `pyo3::pyclass(module=..., name=..., immutable_type, frozen, ...)`.
///
/// Default module is `rapidquery._core`. Pass `module = "rapidquery._core.dialect"`
/// (or another path) to override.
#[proc_macro_attribute]
pub fn pyclass(attr: TokenStream, item: TokenStream) -> TokenStream {
    let PyClassInput {
        name,
        module,
        meta_args,
    } = syn::parse_macro_input!(attr as PyClassInput);
    let item = syn::parse_macro_input!(item as syn::ItemStruct);

    let extra_tokens = if meta_args.is_empty() {
        quote! {}
    } else {
        quote! { , #(#meta_args),* }
    };

    quote! {
        #[pyo3::pyclass(module=#module, name=#name, immutable_type, frozen #extra_tokens)]
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

/// Creates a new [`PyErr`] of the given exception type.
///
/// Examples:
/// ```rust,no-test
/// pyerr!(PyTypeError)
/// // => PyTypeError::new_err(())
///
/// pyerr!(PyTypeError, "hello")
/// // => PyTypeError::new_err("hello")
///
/// pyerr!(PyTypeError, "hello {}", "world")
/// // => PyTypeError::new_err(format!("hello {}", "world"))
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
