use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;

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

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let PyErrInput {
        return_err_enum,
        name,
        msg_or_fmt,
        format_args,
    } = syn::parse2(input)?;

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
        Ok(quote! { Err(#constructed) })
    } else {
        Ok(constructed)
    }
}
