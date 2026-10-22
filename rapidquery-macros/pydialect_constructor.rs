use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::Ident;
use syn::Token;
use syn::Type;
use syn::ext::IdentExt;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

enum Kind {
    /// `name: Ty = default`
    Plain,
    /// `#[enum_(A, B)] name: Ty = A`
    Enum { variants: Vec<Ident> },
    /// `#[bit(A, B)] name: Ty = 0u8`
    Bit { flags: Vec<Ident> },
}

struct Field {
    name: Ident,
    ty: Type,
    default: Expr,
    kind: Kind,
}

struct Fields {
    fields: Punctuated<Field, Token![,]>,
}

impl Parse for Fields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            fields: Punctuated::parse_terminated(input)?,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let default: Expr = input.parse()?;

        let mut kind = Kind::Plain;
        let mut seen = false;

        for attr in &attrs {
            if seen {
                return Err(Error::new_spanned(
                    attr,
                    "a field can have at most one of `#[enum_(..)]` / `#[bit(..)]`",
                ));
            }
            let is_enum = if attr.path().is_ident("enum_") {
                true
            } else if attr.path().is_ident("bit") {
                false
            } else {
                return Err(Error::new_spanned(
                    attr,
                    "unknown attribute, expected `#[enum_(..)]` or `#[bit(..)]`",
                ));
            };

            let names: Vec<Ident> = attr
                .parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)?
                .into_iter()
                .collect();

            if names.is_empty() {
                return Err(Error::new_spanned(attr, "expected at least one name"));
            }

            kind = if is_enum {
                Kind::Enum { variants: names }
            } else {
                Kind::Bit { flags: names }
            };

            seen = true;
        }

        Ok(Self {
            name,
            ty,
            default,
            kind,
        })
    }
}

/// `placeholder_syntax` -> `PLACEHOLDER_SYNTAX`, `FooBar` -> `FOO_BAR`, `SCALAR` -> `SCALAR`,
/// `HTTPServer` -> `HTTP_SERVER`.
fn screaming_snake(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() {
            let prev = chars[i - 1];
            let next_is_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if prev.is_lowercase() || prev.is_ascii_digit() || (prev.is_uppercase() && next_is_lower) {
                out.push('_');
            }
        }
        out.extend(c.to_uppercase());
    }
    out
}

fn const_name(field: &Ident, item: &Ident) -> String {
    format!(
        "OPT_{}_{}",
        screaming_snake(&field.unraw().to_string()),
        screaming_snake(&item.unraw().to_string()),
    )
}

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let input: Fields = syn::parse2(input)?;

    if input.fields.is_empty() {
        return Err(Error::new(Span::call_site(), "expected at least one field declaration"));
    }

    let mut exports = Vec::<TokenStream>::new(); // `module.add(..)?;` statements
    let mut sig = Vec::<TokenStream>::new(); // entries of `signature = (*, ..)`
    let mut params = Vec::<TokenStream>::new(); // parameters of `__new__`
    let mut inits = Vec::<TokenStream>::new(); // fields of the `DialectConfig { .. }` literal
    let mut getters = Vec::<TokenStream>::new(); // `#[getter]` methods

    for f in &input.fields {
        let name = &f.name;
        let ty = &f.ty;
        let name_str = name.unraw().to_string();

        match &f.kind {
            Kind::Plain => {
                let default = &f.default;
                sig.push(quote!(#name = #default));
                params.push(quote!(#name: #ty));
                inits.push(quote!(#name));
                getters.push(quote! {
                    #[getter]
                    fn #name(&self) -> #ty { self.0.#name }
                });
            }

            Kind::Enum { variants } => {
                for v in variants {
                    let c = const_name(name, v);
                    exports.push(quote!(module.add(#c, #ty::#v as u8)?;));
                }

                let default = &f.default;

                sig.push(quote!(#name = #default as u8));
                params.push(quote!(#name: u8));

                // `X as u8` is not a valid pattern, so compare in match guards instead.
                inits.push(quote! {
                    #name: match #name {
                        #( __v if __v == #ty::#variants as u8 => #ty::#variants, )*
                        __v => return ::core::result::Result::Err(
                            pyo3::exceptions::PyValueError::new_err(
                                format!("invalid value {} for `{}`", __v, #name_str),
                            ),
                        ),
                    }
                });

                getters.push(quote! {
                    #[getter]
                    fn #name(&self) -> u8 { self.0.#name as u8 }
                });
            }

            Kind::Bit { flags } => {
                let default = &f.default;

                for flag in flags {
                    let c = const_name(name, flag);
                    exports.push(quote!(module.add(#c, #ty::#flag.bits())?;));
                }

                sig.push(quote!(#name = #default));
                params.push(quote!(#name: u8));

                inits.push(quote! {
                    #name: #ty::from_bits(#name).ok_or_else(|| {
                        pyo3::exceptions::PyValueError::new_err(
                            format!("invalid bits {:#b} for `{}`", #name, #name_str),
                        )
                    })?
                });

                getters.push(quote! {
                    #[getter]
                    fn #name(&self) -> u8 { self.0.#name.bits() }
                });
            }
        }
    }

    Ok(quote! {
        pub(super) fn export_options(
            module: &pyo3::Bound<'_, pyo3::types::PyModule>,
        ) -> pyo3::PyResult<()> {
            use pyo3::types::PyModuleMethods;
            #(#exports)*
            Ok(())
        }

        #[pyo3::pymethods]
        impl PyDialect {
            #[new]
            #[pyo3(signature = (*, #(#sig),*))]
            #[allow(
                clippy::too_many_arguments,
                clippy::fn_params_excessive_bools,
                clippy::cognitive_complexity
            )]
            fn __new__(#(#params),*) -> pyo3::PyResult<Self> {
                let inner = DialectConfig { #(#inits),* };
                Ok(Self(inner))
            }

            #(#getters)*
        }
    })
}
