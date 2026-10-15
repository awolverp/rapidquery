use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Token;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

struct Options {
    flags: Vec<Flag>,
}

enum Flag {
    Char { field: syn::Ident, default: syn::LitChar },
    Enum(EnumFlag),
    Flags(FlagsFlag),
    Bool { field: syn::Ident, default: syn::Expr },
}

struct FlagsFlag {
    ty: syn::Ident,
    field: syn::Ident,
    default: syn::Expr,
    variants: Vec<Variant>,
}

struct EnumFlag {
    ty: syn::Ident,
    field: syn::Ident,
    variants: Vec<Variant>,
}

struct Variant {
    name: syn::Ident,
    opt: syn::Ident,
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let opt: syn::Ident = input.parse()?;
        Ok(Self { name, opt })
    }
}

impl Parse for FlagsFlag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: syn::Ident = input.parse()?;
        let field: syn::Ident = input.parse()?;
        let default = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            input.parse()?
        } else {
            syn::parse_quote!(0)
        };
        let body;
        syn::braced!(body in input);
        let variants: Punctuated<Variant, Token![,]> = body.parse_terminated(Variant::parse, Token![,])?;
        if variants.is_empty() {
            return Err(syn::Error::new(field.span(), "flag set needs at least one flag"));
        }
        Ok(Self {
            ty,
            field,
            default,
            variants: variants.into_iter().collect(),
        })
    }
}

impl Parse for EnumFlag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: syn::Ident = input.parse()?;
        let field: syn::Ident = input.parse()?;
        let body;
        syn::braced!(body in input);
        let variants: Punctuated<Variant, Token![,]> = body.parse_terminated(Variant::parse, Token![,])?;
        if variants.is_empty() {
            return Err(syn::Error::new(field.span(), "flag enum needs at least one variant"));
        }
        Ok(Self {
            ty,
            field,
            variants: variants.into_iter().collect(),
        })
    }
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut flags = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![enum]) {
                input.parse::<Token![enum]>()?;
                flags.push(Flag::Enum(input.parse()?));
                continue;
            }

            let kind: syn::Ident = input.parse()?;
            if kind == "char" {
                let field: syn::Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let default: syn::LitChar = input.parse()?;
                input.parse::<Token![;]>()?;
                flags.push(Flag::Char { field, default });
            } else if kind == "flags" {
                flags.push(Flag::Flags(input.parse()?));
            } else if kind == "bool" {
                let field: syn::Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let default: syn::Expr = input.parse()?;
                input.parse::<Token![;]>()?;
                flags.push(Flag::Bool { field, default });
            } else {
                return Err(syn::Error::new(
                    kind.span(),
                    "expected `char`, `enum`, `flags`, or `bool`",
                ));
            }
        }
        Ok(Self { flags })
    }
}

fn setter(field: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&format!("with_{field}"), field.span())
}

pub fn generate(input: TokenStream) -> syn::Result<TokenStream> {
    let options: Options = syn::parse2(input)?;

    let mut consts = TokenStream::new();
    let mut exports = TokenStream::new();
    let mut params = TokenStream::new();
    let mut args = TokenStream::new();
    let mut checks = TokenStream::new();
    let mut setters = TokenStream::new();
    let mut getters = TokenStream::new();

    for flag in &options.flags {
        match flag {
            Flag::Char { field, default } => {
                let set = setter(field);
                let get = format_ident!("get_{field}");
                params.extend(quote! { #field = #default, });
                args.extend(quote! { #field: char, });
                setters.extend(quote! { .#set(#field as u32) });
                getters.extend(quote! {
                    #[getter]
                    fn #get(&self) -> char {
                        // SAFETY: the constructor stores a `char` cast to `u32`.
                        unsafe { char::from_u32_unchecked(self.0.#field()) }
                    }
                });
            }
            Flag::Enum(flag) => {
                let ty = &flag.ty;
                let field = &flag.field;
                let set = setter(field);
                let get = format_ident!("get_{field}");
                params.extend(quote! { #field = 0, });
                args.extend(quote! { #field: u8, });
                checks.extend(quote! { let #field = flags::#ty::from_bits(#field); });
                setters.extend(quote! { .#set(#field) });
                getters.extend(quote! {
                    #[getter]
                    fn #get(&self) -> u8 {
                        self.0.#field().into_bits()
                    }
                });
                for variant in &flag.variants {
                    let name = &variant.name;
                    let opt = &variant.opt;
                    consts.extend(quote! {
                        pub(super) const #opt: u8 = flags::#ty::#name.into_bits();
                    });
                    exports.extend(quote! {
                        module.add(stringify!(#opt), #opt)?;
                    });
                }
            }
            Flag::Flags(flag) => {
                let ty = &flag.ty;
                let field = &flag.field;
                let default = &flag.default;
                let set = setter(field);
                let get = format_ident!("get_{field}");
                params.extend(quote! { #field = #default, });
                args.extend(quote! { #field: u8, });
                checks.extend(quote! {
                    let #field = {
                        let bits = #field;
                        match flags::#ty::from_bits(bits) {
                            Some(value) => value,
                            None => {
                                return rapidquery_macros::pyerr!(
                                    Err(PyValueError),
                                    "unknown {} bits: {}",
                                    stringify!(#field),
                                    bits
                                );
                            }
                        }
                    };
                });
                setters.extend(quote! { .#set(#field) });
                getters.extend(quote! {
                    #[getter]
                    fn #get(&self) -> u8 {
                        self.0.#field().bits()
                    }
                });
                for variant in &flag.variants {
                    let name = &variant.name;
                    let opt = &variant.opt;
                    consts.extend(quote! {
                        pub(super) const #opt: u8 = flags::#ty::#name.bits();
                    });
                    exports.extend(quote! {
                        module.add(stringify!(#opt), #opt)?;
                    });
                }
            }
            Flag::Bool { field, default } => {
                let set = setter(field);
                let get = format_ident!("get_{field}");
                params.extend(quote! { #field = #default, });
                args.extend(quote! { #field: bool, });
                setters.extend(quote! { .#set(#field) });
                getters.extend(quote! {
                    #[getter]
                    fn #get(&self) -> bool {
                        self.0.#field()
                    }
                });
            }
        }
    }

    Ok(quote! {
        #consts

        pub(super) fn export_options(
            module: &pyo3::Bound<'_, pyo3::types::PyModule>,
        ) -> pyo3::PyResult<()> {
            use pyo3::types::PyModuleMethods;
            #exports
            Ok(())
        }

        #[pyo3::pymethods]
        impl PyDialect {
            #[new]
            #[pyo3(signature = (*, #params))]
            #[allow(
                clippy::too_many_arguments,
                clippy::fn_params_excessive_bools,
                clippy::cognitive_complexity
            )]
            fn __new__(#args) -> pyo3::PyResult<Self> {
                #checks
                Ok(Self(flags::DialectFlags::new() #setters))
            }

            #getters
        }
    })
}
