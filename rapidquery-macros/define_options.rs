//! Assisted-by: Claude

use proc_macro2::Literal;
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
use syn::ext::IdentExt;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

/// Name of the helper attribute placed on fields.
const HELPER_ATTR: &str = "define";

/// Bitflags types are `u8`, so at most this many flags fit.
const MAX_FLAGS: usize = u8::BITS as usize;

// ============================================================================
// AST
// ============================================================================

struct Variant {
    attrs: Vec<Attribute>,
    ident: Ident,
}

struct Flag {
    attrs: Vec<Attribute>,
    ident: Ident,
    /// Explicit value; `None` means "next bit by position".
    value: Option<Expr>,
}

/// The parsed body of `#[define(..)]`.
enum Body {
    Enum(Vec<Variant>),
    Bits(Vec<Flag>),
}

/// A type that must be emitted next to the struct.
struct GeneratedType {
    name: Ident,
    vis: Visibility,
    docs: Vec<Attribute>,
    body: Body,
}

// ============================================================================
// Parsing
// ============================================================================

impl Parse for Variant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
        })
    }
}

impl Parse for Flag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        let value = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self { attrs, ident, value })
    }
}

/// Parses `{ item, item, .. }` with at least one item.
fn parse_braced<T: Parse>(input: ParseStream, what: &str) -> syn::Result<Vec<T>> {
    let content;
    let brace = syn::braced!(content in input);
    let items: Vec<T> = Punctuated::<T, Token![,]>::parse_terminated(&content)?
        .into_iter()
        .collect();

    if items.is_empty() {
        return Err(syn::Error::new(
            brace.span.join(),
            format!("expected at least one {what}"),
        ));
    }
    Ok(items)
}

impl Parse for Body {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            return Ok(Self::Enum(parse_braced(input, "variant")?));
        }

        let word = Ident::parse_any(input)?;
        if word != "bits" {
            return Err(syn::Error::new_spanned(word, "expected `enum { .. }` or `bits { .. }`"));
        }

        let flags: Vec<Flag> = parse_braced(input, "flag")?;
        if flags.len() > MAX_FLAGS {
            return Err(syn::Error::new_spanned(
                &flags[MAX_FLAGS].ident,
                format!("a `u8` bitflags type holds at most {MAX_FLAGS} flags"),
            ));
        }
        Ok(Self::Bits(flags))
    }
}

// ============================================================================
// Code generation
// ============================================================================

impl GeneratedType {
    fn into_tokens(self) -> TokenStream {
        let Self { name, vis, docs, body } = self;

        match body {
            Body::Enum(variants) => {
                let variants = variants
                    .into_iter()
                    .map(|Variant { attrs, ident }| quote!(#(#attrs)* #ident));
                quote! {
                    #(#docs)*
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    #vis enum #name {
                        #(#variants),*
                    }
                }
            }

            Body::Bits(flags) => {
                let flags = flags.into_iter().enumerate().map(|(index, flag)| {
                    let Flag { attrs, ident, value } = flag;
                    let value = value.map_or_else(
                        || {
                            let bit = Literal::usize_unsuffixed(index);
                            quote!(1 << #bit)
                        },
                        |expr| quote!(#expr),
                    );
                    quote!(#(#attrs)* const #ident = #value;)
                });
                quote! {
                    ::bitflags::bitflags! {
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

/// Returns `T` for `Option<T>`, otherwise the type itself.
fn unwrap_option(ty: &Type) -> &Type {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && args.args.len() == 1
        && let syn::GenericArgument::Type(inner) = &args.args[0]
    {
        return inner;
    }
    ty
}

/// Reads and strips the `#[define(..)]` attribute of `field`.
///
/// Returns the type to generate, or `None` if the field has no such attribute.
/// The attribute is always removed so the compiler never sees it.
fn take_generated_type(field: &mut syn::Field) -> syn::Result<Option<GeneratedType>> {
    let mut bodies = Vec::new();
    for attr in &field.attrs {
        if attr.path().is_ident(HELPER_ATTR) {
            bodies.push((attr.clone(), attr.parse_args::<Body>()?));
        }
    }
    field.attrs.retain(|attr| !attr.path().is_ident(HELPER_ATTR));

    let Some((attr, body)) = bodies.pop() else {
        return Ok(None);
    };
    if !bodies.is_empty() {
        return Err(syn::Error::new_spanned(
            attr,
            format!("a field can have at most one `#[{HELPER_ATTR}(..)]`"),
        ));
    }

    let name = match unwrap_option(&field.ty) {
        Type::Path(path) => path.path.get_ident().cloned(),
        _ => None,
    }
    .ok_or_else(|| {
        syn::Error::new_spanned(
            &field.ty,
            format!("`#[{HELPER_ATTR}(..)]` needs a plain type name (or `Option<Name>`)"),
        )
    })?;

    let docs = field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect();

    Ok(Some(GeneratedType {
        name,
        vis: field.vis.clone(),
        docs,
        body,
    }))
}

// ============================================================================
// Entry point
// ============================================================================

pub fn expand(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut input: syn::DeriveInput = syn::parse2(item)?;
    let mut generated = Vec::new();

    let Data::Struct(data) = &mut input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`#[define_options]` can only be used on structs",
        ));
    };
    let Fields::Named(fields) = &mut data.fields else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`#[define_options]` requires a struct with named fields",
        ));
    };

    for field in &mut fields.named {
        if let Some(ty) = take_generated_type(field)? {
            generated.push(ty.into_tokens());
        }
    }

    Ok(quote! {
        #(#generated)*
        #input
    })
}
