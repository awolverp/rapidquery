use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::Expr;
use syn::Fields;
use syn::Ident;
use syn::Token;
use syn::Type;
use syn::Visibility;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::token::Comma;

struct EnumVariant {
    attrs: Vec<Attribute>,
    ident: Ident,
}

impl Parse for EnumVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
        })
    }
}

struct Bitflag {
    attrs: Vec<Attribute>,
    ident: Ident,
    value: Expr,
}

impl Parse for Bitflag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(Self { attrs, ident, value })
    }
}

struct GeneratedType {
    name: Ident,
    vis: Visibility,
    docs: Vec<Attribute>,
    kind: GeneratedKind,
}

enum GeneratedKind {
    Enum(Punctuated<EnumVariant, Comma>),
    Bitflags(Punctuated<Bitflag, Comma>),
}

impl GeneratedType {
    fn into_tokens(self) -> proc_macro2::TokenStream {
        let Self { name, vis, docs, kind } = self;
        match kind {
            GeneratedKind::Enum(variants) => {
                let variants = variants.into_iter().map(|v| {
                    let attrs = v.attrs;
                    let ident = v.ident;
                    quote!( #(#attrs)* #ident )
                });
                quote! {
                    #(#docs)*
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    #vis enum #name {
                        #(#variants),*
                    }
                }
            }
            GeneratedKind::Bitflags(flags) => {
                let flags = flags.into_iter().map(|f| {
                    let attrs = f.attrs;
                    let ident = f.ident;
                    let value = f.value;
                    quote!( #(#attrs)* const #ident = #value; )
                });
                quote! {
                    bitflags::bitflags! {
                        #(#docs)*
                        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                        #vis struct #name: u8 {
                            #(#flags)*
                        }
                    }
                }
            }
        }
    }
}

fn extract_generated_type(field: &mut syn::Field) -> syn::Result<Option<GeneratedType>> {
    let docs: Vec<Attribute> = field
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .cloned()
        .collect();

    let ty_name = match &field.ty {
        Type::Path(p) => p.path.get_ident().cloned(),
        _ => None,
    };

    let Some(name) = ty_name else {
        // Still strip any helper attributes so the compiler doesn’t complain
        field
            .attrs
            .retain(|a| !a.path().is_ident("enum_") && !a.path().is_ident("bit"));

        return Ok(None);
    };

    let mut generated = None;

    for attr in &field.attrs {
        if attr.path().is_ident("enum_") {
            let variants = attr.parse_args_with(Punctuated::<EnumVariant, Comma>::parse_terminated)?;
            generated = Some(GeneratedType {
                name: name.clone(),
                vis: field.vis.clone(),
                docs: docs.clone(),
                kind: GeneratedKind::Enum(variants),
            });
            break;
        }

        if attr.path().is_ident("bit") {
            let flags = attr.parse_args_with(Punctuated::<Bitflag, Comma>::parse_terminated)?;
            generated = Some(GeneratedType {
                name,
                vis: field.vis.clone(),
                docs,
                kind: GeneratedKind::Bitflags(flags),
            });
            break;
        }
    }

    // Strip the helper attributes after we’re done reading them
    field
        .attrs
        .retain(|a| !a.path().is_ident("enum_") && !a.path().is_ident("bit"));

    Ok(generated)
}

pub fn expand(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let mut generated = Vec::new();

    if let Data::Struct(data) = &mut input.data {
        // Only named fields are supported
        if let Fields::Named(fields) = &mut data.fields {
            for field in &mut fields.named {
                // Generate type
                if let Some(ty) = extract_generated_type(field)? {
                    generated.push(ty.into_tokens());
                }
            }
        }
    }

    Ok(quote! {
        #(#generated)*
        #input
    })
}
