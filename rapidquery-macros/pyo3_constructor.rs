//! Assisted-by: Claude

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::Ident;
use syn::LitInt;
use syn::Token;
use syn::Type;
use syn::ext::IdentExt;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;

/// One placeholder of a template, e.g. `left` or `quote?`.
struct TemplateKey {
    /// Placeholder name. Empty for the positional `{}`.
    name: String,
    required: bool,
}

/// What kind of value a field holds. Decides conversion, getter and exports.
enum Kind {
    /// `name: Ty` — passed through unchanged.
    Plain(Type),
    /// `name: class Ty` — a `#[pyclass]` value, returned to Python by clone.
    Class(Type),
    /// `name: enum Ty { A, B }` — a Rust enum exposed to Python as `u8`.
    Enum { ty: Type, variants: Vec<Ident> },
    /// `name: bits Ty { X, Y }` — a bitflags type exposed to Python as `u8`.
    Bits { ty: Type, flags: Vec<Ident> },
    /// `name: template(..; n)` — `template::Template`, exposed as `str`.
    Template {
        keys: Vec<TemplateKey>,
        min: LitInt,
        max: LitInt,
    },
}

/// The `= ...` part of a field. Its syntax depends on the field's [`Kind`].
enum DefaultValue {
    /// No default: the argument is required in Python.
    Required,
    /// `None` (only valid for `Option<..>` fields).
    Null,
    /// A Rust expression (plain / class / template fields).
    Expr(Expr),
    /// An enum variant name.
    Variant(Ident),
    /// A set of bit flags. Empty means no flags (`empty`).
    Flags(Vec<Ident>),
}

struct Field {
    name: Ident,
    is_option: bool,
    kind: Kind,
    default: DefaultValue,
}

struct MacroInput {
    /// Struct to generate the constructor for.
    target: Ident,
    /// Name of the generated function that registers the `OPT_*` constants.
    export_fn: Ident,
    fields: Punctuated<Field, Token![,]>,
}

/// Returns the next token as a string if it is an identifier, without consuming it.
fn peek_word(input: ParseStream) -> Option<String> {
    input.cursor().ident().map(|(ident, _)| ident.to_string())
}

/// Parses `{ A, B, C }` (at least one item).
fn parse_braced_idents(input: ParseStream, what: &str) -> syn::Result<Vec<Ident>> {
    let content;
    let brace = syn::braced!(content in input);
    let items: Vec<Ident> = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
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

/// Errors if `item` is not listed in `members`.
fn check_member(item: &Ident, members: &[Ident], what: &str) -> syn::Result<()> {
    if members.contains(item) {
        return Ok(());
    }
    let list = members.iter().map(Ident::to_string).collect::<Vec<_>>().join(", ");
    Err(syn::Error::new_spanned(
        item,
        format!("`{item}` is not a declared {what}; expected one of: {list}"),
    ))
}

impl Parse for TemplateKey {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            String::new()
        } else {
            // `parse_any` so that keywords such as `type` can be used as keys.
            Ident::parse_any(input)?.unraw().to_string()
        };

        let optional = input.peek(Token![?]);
        if optional {
            input.parse::<Token![?]>()?;
        }

        Ok(Self {
            name,
            required: !optional,
        })
    }
}

/// Parses the `(keys; count)` part after the `template` keyword.
fn parse_template(input: ParseStream) -> syn::Result<Kind> {
    let args;
    let paren = syn::parenthesized!(args in input);

    let mut keys = Vec::new();
    while !args.peek(Token![;]) {
        keys.push(args.parse::<TemplateKey>()?);
        if args.peek(Token![,]) {
            args.parse::<Token![,]>()?;
        } else {
            break;
        }
    }
    if keys.is_empty() {
        return Err(syn::Error::new(
            paren.span.join(),
            "expected at least one template key (use `_` for a positional `{}`)",
        ));
    }

    args.parse::<Token![;]>()?;
    let min: LitInt = args.parse()?;
    let max: LitInt = if args.peek(Token![..=]) {
        args.parse::<Token![..=]>()?;
        args.parse()?
    } else {
        min.clone()
    };

    Ok(Kind::Template { keys, min, max })
}

/// Parses a [`Kind`] (without the `Option<..>` wrapper).
fn parse_kind(input: ParseStream) -> syn::Result<Kind> {
    if input.peek(Token![enum]) {
        input.parse::<Token![enum]>()?;
        let ty: Type = input.parse()?;
        let variants = parse_braced_idents(input, "variant")?;
        return Ok(Kind::Enum { ty, variants });
    }

    match peek_word(input).as_deref() {
        Some("bits") => {
            input.parse::<Ident>()?;
            let ty: Type = input.parse()?;
            let flags = parse_braced_idents(input, "flag")?;
            Ok(Kind::Bits { ty, flags })
        }
        Some("class") => {
            input.parse::<Ident>()?;
            Ok(Kind::Class(input.parse()?))
        }
        Some("template") => {
            input.parse::<Ident>()?;
            parse_template(input)
        }
        _ => Ok(Kind::Plain(input.parse()?)),
    }
}

/// Parses a field spec; returns `(is_option, kind)`.
fn parse_spec(input: ParseStream) -> syn::Result<(bool, Kind)> {
    if peek_word(input).as_deref() == Some("Option") && input.peek2(Token![<]) {
        input.parse::<Ident>()?;
        input.parse::<Token![<]>()?;
        let kind = parse_kind(input)?;
        input.parse::<Token![>]>()?;
        Ok((true, kind))
    } else {
        Ok((false, parse_kind(input)?))
    }
}

impl DefaultValue {
    fn parse(input: ParseStream, kind: &Kind, is_option: bool) -> syn::Result<Self> {
        if !input.peek(Token![=]) {
            return Ok(Self::Required);
        }
        input.parse::<Token![=]>()?;

        if peek_word(input).as_deref() == Some("None") {
            let none: Ident = input.parse()?;
            if !is_option {
                return Err(syn::Error::new_spanned(
                    none,
                    "`None` is only valid for `Option<..>` fields",
                ));
            }
            return Ok(Self::Null);
        }

        match kind {
            Kind::Enum { variants, .. } => {
                let variant: Ident = input.parse()?;
                check_member(&variant, variants, "variant")?;
                Ok(Self::Variant(variant))
            }
            Kind::Bits { flags, .. } => {
                if peek_word(input).as_deref() == Some("empty") {
                    input.parse::<Ident>()?;
                    return Ok(Self::Flags(Vec::new()));
                }
                let used = Punctuated::<Ident, Token![|]>::parse_separated_nonempty(input)?;
                let used: Vec<Ident> = used.into_iter().collect();
                for flag in &used {
                    check_member(flag, flags, "flag")?;
                }
                Ok(Self::Flags(used))
            }
            _ => Ok(Self::Expr(input.parse()?)),
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let (is_option, kind) = parse_spec(input)?;
        let default = DefaultValue::parse(input, &kind, is_option)?;

        Ok(Self {
            name,
            is_option,
            kind,
            default,
        })
    }
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let export_fn: Ident = input.parse()?;

        let content;
        syn::braced!(content in input);
        let fields = Punctuated::parse_terminated(&content)?;

        Ok(Self {
            target,
            export_fn,
            fields,
        })
    }
}

/// `placeholder_syntax` -> `PLACEHOLDER_SYNTAX`, `FooBar` -> `FOO_BAR`,
/// `SCALAR` -> `SCALAR`, `HTTPServer` -> `HTTP_SERVER`.
fn screaming_snake(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && c.is_uppercase() {
            let prev = chars[i - 1];
            let next_is_lower = chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            let word_boundary = prev.is_lowercase() || prev.is_ascii_digit() || (prev.is_uppercase() && next_is_lower);
            if word_boundary {
                out.push('_');
            }
        }
        out.extend(c.to_uppercase());
    }
    out
}

/// Single-segment type name used for `OPT_*` constants.
fn enum_type_ident(ty: &Type) -> &Ident {
    match ty {
        Type::Path(type_name) => type_name.path.get_ident().expect("enum/bits type is a single ident"),
        _ => panic!("enum/bits type is a single ident"),
    }
}

/// Name of the exported module constant: `OPT_<TYPE_NAME>_<ITEM>`.
fn const_name(field: &Ident, item: &Ident) -> String {
    format!(
        "OPT_{}_{}",
        screaming_snake(&field.unraw().to_string()),
        screaming_snake(&item.unraw().to_string()),
    )
}

/// Everything one field contributes to the final expansion.
struct Generated {
    /// `module.add(..)` statements for the export function.
    exports: Vec<TokenStream>,
    /// Entry in `#[pyo3(signature = (*, ..))]`: `name` or `name = default`.
    signature: TokenStream,
    /// Parameter of `__new__`: `name: Type`.
    param: TokenStream,
    /// Field initializer inside `__new__`: `name: <converted value>`.
    init: TokenStream,
    /// The `#[getter]` method.
    getter: TokenStream,
    /// Field initializer inside `Default::default`.
    default_init: TokenStream,
}

/// `&[("key", required), ...]`, the form `Template::parse` expects.
fn template_keys(keys: &[TemplateKey]) -> TokenStream {
    let items = keys.iter().map(|k| {
        let (name, required) = (&k.name, k.required);
        quote!((#name, #required))
    });
    quote!(&[#(#items),*])
}

impl Field {
    /// The Rust type written by the user, for kinds that have one.
    fn ty(&self) -> Option<&Type> {
        match &self.kind {
            Kind::Plain(ty) | Kind::Class(ty) | Kind::Enum { ty, .. } | Kind::Bits { ty, .. } => Some(ty),
            Kind::Template { .. } => None,
        }
    }

    fn wrap_option(&self, inner: TokenStream) -> TokenStream {
        if self.is_option { quote!(Option<#inner>) } else { inner }
    }

    /// Type of the struct field.
    fn rust_ty(&self) -> TokenStream {
        let inner = match self.ty() {
            Some(ty) => quote!(#ty),
            None => quote!(template::Template),
        };
        self.wrap_option(inner)
    }

    /// Type of the `__new__` parameter (what Python passes in).
    fn param_ty(&self) -> TokenStream {
        let inner = match &self.kind {
            Kind::Plain(ty) | Kind::Class(ty) => quote!(#ty),
            Kind::Enum { .. } | Kind::Bits { .. } => quote!(u8),
            Kind::Template { .. } => quote!(&str),
        };
        self.wrap_option(inner)
    }

    /// Turns the Python-side `value` expression into the Rust field value.
    /// May `return Err(..)` / use `?`, so it must be used inside `__new__`.
    fn convert(&self, value: &TokenStream) -> TokenStream {
        let name_str = self.name.unraw().to_string();

        match &self.kind {
            Kind::Plain(_) | Kind::Class(_) => value.clone(),

            Kind::Enum { ty, variants } => {
                let type_ident = enum_type_ident(ty);
                let names: Vec<_> = variants.iter().map(|v| const_name(type_ident, v)).collect();
                let values = 0..variants.len() as u8;
                quote! {
                    match #value {
                        #( __x if __x == <#ty>::#variants as u8 => <#ty>::#variants, )*
                        __x => return ::core::result::Result::Err(
                            error::Error::InvalidEnumOptionField {
                                field: #name_str,
                                value: __x,
                                expected: error::format_options(&[#(#names),*], &[#(#values),*]),
                            },
                        ),
                    }
                }
            }

            Kind::Bits { ty, flags } => {
                let type_ident = enum_type_ident(ty);
                let names: Vec<_> = flags.iter().map(|flag| const_name(type_ident, flag)).collect();
                quote! {
                    <#ty>::from_bits(#value).ok_or_else(|| {
                        error::Error::InvalidBitOptionField {
                            field: #name_str,
                            value: #value,
                            expected: error::format_options(
                                &[#(#names),*],
                                &[#(<#ty>::#flags.bits()),*],
                            ),
                        }
                    })?
                }
            }

            Kind::Template { keys, min, max } => {
                let keys = template_keys(keys);
                quote! {
                    template::Template::parse(#value, #keys, #min, #max)
                        .map_err(|reason| error::Error::InvalidTemplate {
                            field: #name_str,
                            reason,
                        })?
                }
            }
        }
    }

    /// `name: <value>` for `__new__`; unwraps `Option` around the conversion.
    fn init(&self) -> TokenStream {
        let name = &self.name;

        if matches!(self.kind, Kind::Plain(_) | Kind::Class(_)) {
            return quote!(#name: #name);
        }

        if self.is_option {
            let converted = self.convert(&quote!(__value));
            quote!(#name: match #name { None => None, Some(__value) => Some(#converted) })
        } else {
            let converted = self.convert(&quote!(#name));
            quote!(#name: #converted)
        }
    }

    /// A getter that maps `&field` through `body` (written in terms of `__x`).
    fn mapped_getter(&self, ret: TokenStream, body: TokenStream) -> TokenStream {
        let name = &self.name;
        if self.is_option {
            quote! {
                #[getter]
                fn #name(&self) -> Option<#ret> { self.#name.as_ref().map(|__x| #body) }
            }
        } else {
            quote! {
                #[getter]
                fn #name(&self) -> #ret { let __x = &self.#name; #body }
            }
        }
    }

    fn getter(&self) -> TokenStream {
        let name = &self.name;

        match &self.kind {
            Kind::Plain(_) => {
                let ty = self.rust_ty();
                quote! { #[getter] fn #name(&self) -> #ty { self.#name } }
            }

            Kind::Class(_) => {
                let signature = quote! {
                    fn #name<'a>(&self, py: ::pyo3::Python<'a>)
                        -> ::pyo3::PyResult<::pyo3::Bound<'a, ::pyo3::PyAny>>
                };
                if self.is_option {
                    quote! {
                        #[getter]
                        #signature {
                            match self.#name.as_ref() {
                                Some(x) => ::pyo3::Bound::new(py, x.clone()).map(|x| x.into_any()),
                                None => Ok(py.None().into_bound(py)),
                            }
                        }
                    }
                } else {
                    quote! {
                        #[getter]
                        #signature {
                            ::pyo3::Bound::new(py, self.#name.clone()).map(|x| x.into_any())
                        }
                    }
                }
            }

            Kind::Enum { .. } => self.mapped_getter(quote!(u8), quote!(*__x as u8)),
            Kind::Bits { .. } => self.mapped_getter(quote!(u8), quote!(__x.bits())),
            Kind::Template { .. } => self.mapped_getter(quote!(&str), quote!(__x.as_str())),
        }
    }

    /// `(python_default, rust_default)` or `None` if the field is required.
    ///
    /// * `python_default` is what goes into the `#[pyo3(signature)]`
    ///   (same type as the `__new__` parameter).
    /// * `rust_default` is the struct-field value used by `Default`.
    fn defaults(&self) -> Option<(TokenStream, TokenStream)> {
        let (py, rust) = match (&self.kind, &self.default) {
            (_, DefaultValue::Required) => return None,
            (_, DefaultValue::Null) => return Some((quote!(None), quote!(None))),

            (Kind::Enum { ty, .. }, DefaultValue::Variant(v)) => (quote!(<#ty>::#v as u8), quote!(<#ty>::#v)),

            (Kind::Bits { ty, .. }, DefaultValue::Flags(flags)) => {
                let bits = if flags.is_empty() {
                    quote!(0u8)
                } else {
                    quote!(#(<#ty>::#flags.bits())|*)
                };
                (bits.clone(), quote!(<#ty>::from_bits_truncate(#bits)))
            }

            (Kind::Template { keys, min, max }, DefaultValue::Expr(e)) => {
                let keys = template_keys(keys);
                let msg = format!("invalid default template for `{}`", self.name.unraw());
                (
                    quote!(#e),
                    quote!(template::Template::parse(#e, #keys, #min, #max).expect(#msg)),
                )
            }

            (_, DefaultValue::Expr(e)) => (quote!(#e), quote!(#e)),

            _ => unreachable!("the parser ties the default syntax to the field kind"),
        };

        if self.is_option {
            Some((quote!(Some(#py)), quote!(Some(#rust))))
        } else {
            Some((py, rust))
        }
    }

    fn exports(&self) -> Vec<TokenStream> {
        match &self.kind {
            Kind::Enum { ty, variants } => {
                let Type::Path(type_name) = ty else { unreachable!() };

                variants
                    .iter()
                    .map(|v| {
                        let c = const_name(type_name.path.get_ident().unwrap(), v);
                        quote!(module.add(#c, <#ty>::#v as u8)?;)
                    })
                    .collect()
            }
            Kind::Bits { ty, flags } => {
                let Type::Path(type_name) = ty else { unreachable!() };

                flags
                    .iter()
                    .map(|f| {
                        let c = const_name(type_name.path.get_ident().unwrap(), f);
                        quote!(module.add(#c, <#ty>::#f.bits())?;)
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn generate(&self) -> syn::Result<Generated> {
        let name = &self.name;
        let defaults = self.defaults();

        let signature = match &defaults {
            Some((py, _)) => quote!(#name = #py),
            None => quote!(#name),
        };

        let default_init = match defaults {
            Some((_, rust)) => quote!(#name: #rust),
            None if matches!(self.kind, Kind::Template { .. }) => {
                return Err(syn::Error::new_spanned(
                    name,
                    "a default value is required for template fields",
                ));
            }
            None => {
                let ty = self.rust_ty();
                quote!(#name: <#ty as ::core::default::Default>::default())
            }
        };

        let param_ty = self.param_ty();

        Ok(Generated {
            exports: self.exports(),
            signature,
            param: quote!(#name: #param_ty),
            init: self.init(),
            getter: self.getter(),
            default_init,
        })
    }
}

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let MacroInput {
        target,
        export_fn,
        fields,
    } = syn::parse2(input)?;

    if fields.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected at least one field declaration",
        ));
    }

    let generated = fields.iter().map(Field::generate).collect::<syn::Result<Vec<_>>>()?;

    let exports: Vec<_> = generated.iter().flat_map(|g| g.exports.iter()).collect();
    let signatures: Vec<_> = generated.iter().map(|g| &g.signature).collect();
    let params: Vec<_> = generated.iter().map(|g| &g.param).collect();
    let inits: Vec<_> = generated.iter().map(|g| &g.init).collect();
    let getters: Vec<_> = generated.iter().map(|g| &g.getter).collect();
    let default_inits: Vec<_> = generated.iter().map(|g| &g.default_init).collect();

    Ok(quote! {
        impl ::core::default::Default for #target {
            fn default() -> Self {
                Self { #(#default_inits),* }
            }
        }

        /// Registers the `OPT_*` constants of this type on the Python module.
        #[cold]
        fn #export_fn(
            module: &::pyo3::Bound<'_, ::pyo3::types::PyModule>,
        ) -> pyo3::PyResult<()> {
            use ::pyo3::types::PyModuleMethods;
            #(#exports)*
            Ok(())
        }

        #[::pyo3::pymethods]
        impl #target {
            #[new]
            #[pyo3(signature = (*, #(#signatures),*))]
            #[allow(
                clippy::too_many_arguments,
                clippy::fn_params_excessive_bools,
                clippy::cognitive_complexity
            )]
            fn __new__(#(#params),*) -> error::Result<Self> {
                Ok(Self { #(#inits),* })
            }

            #(#getters)*
        }
    })
}
