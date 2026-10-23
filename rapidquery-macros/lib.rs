use proc_macro::TokenStream;

mod cust_protocol;
mod define_options;
mod pyclass;
mod pyerr;
mod pyo3_constructor;

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
    match pyclass::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Builds a [`PyErr`], or `Err(PyErr)` when the name is wrapped in `Err`.
///
/// ```ignore
/// pyerr!(PyTypeError, "bad {}", x)
/// pyerr!(Err(PyValueError))
/// ```
#[proc_macro]
pub fn pyerr(input: TokenStream) -> TokenStream {
    match pyerr::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Builds a customization protocol checking.
///
/// Note: you can access to attribute result using `protocol` variable.
///
/// For example:
/// ```ignore
/// cust_protocol! {
///     on: bound,
///     attr: c"__serializer__",
///     ty: crate::typeref::SERIALIZER_TYPE,
///     matches: {
///         // OK expr
///     },
///     mismatches: {
///         // ERR expr
///     }
/// }
/// ```
///
/// becomes:
/// ```ignore
/// unsafe {
///     const PROTOCOL: &'static std::ffi::CStr = c"__serializer__";
///
///     let result = {
///         // Uses pyo3::ffi to get attribute
///     };
///
///     if let Some(protocol) = result {
///         if protocol is crate::typeref::SERIALIZER_TYPE {
///             // OK expr
///         } else {
///             // ERR expr
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn cust_protocol(input: TokenStream) -> TokenStream {
    match cust_protocol::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// generates the `enum` / `bitflags` types that the
/// fields of a config struct refer to.
///
/// Each field declares its type's body with a `#[define(..)]` attribute. The
/// type name is taken from the field's type (`Option<..>` is looked through),
/// and the field's doc comments become the type's docs.
///
/// ```ignore
/// #[define_options]
/// pub struct LikeConfig {
///     /// Where an inlined `ESCAPE` clause is legal.
///     #[define(bits { SCALAR, ANY })]
///     pub escape: LikeEscape,
///
///     /// How `LIKE ANY` is rendered.
///     #[define(enum { #[default] OrExpansion, ArrayConstructor, PatternList })]
///     pub any: LikeAny,
/// }
/// ```
///
/// This mirrors `pyo3_constructor!`, where the same fields are written as
/// `bits LikeEscape { SCALAR, ANY }` / `enum LikeAny { .. }`.
///
/// * `enum { A, B }` — a fieldless enum (derives `Default`, so mark one
///   variant `#[default]` if the enum is ever defaulted).
/// * `bits { A, B }` — a `u8` bitflags type. Flags get bit `1 << index`
///   automatically; write `A = <expr>` to override the value.
///
/// Attributes (e.g. docs, `#[default]`) on variants and flags are kept.
#[proc_macro_attribute]
pub fn define_options(attr: TokenStream, item: TokenStream) -> TokenStream {
    match define_options::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// generates the PyO3 keyword-only constructor, getters,
/// `Default` impl and exported `OPT_*` constants for a "config" struct.
///
/// # Syntax
///
/// ```ignore
/// pyo3_constructor! {
///     StructName => export_function_name {
///         field: <spec> [= <default>],
///         ...
///     }
/// }
/// ```
///
/// ## Field specs
///
/// | Spec                                   | Python type | Default syntax          |
/// |----------------------------------------|-------------|-------------------------|
/// | `Type` (e.g. `bool`, `u32`)            | `Type`      | any Rust expression     |
/// | `class Type`                           | pyclass     | any Rust expression     |
/// | `enum Type { A, B }`                   | `int`       | a variant: `A`          |
/// | `bits Type { X, Y }`                   | `int`       | `empty` or `X \| Y`     |
/// | `template(key, opt?; 1)`               | `str`       | a string literal        |
///
/// * Wrap any spec in `Option<..>` to make the field optional. For optional
///   fields the default is `None`, or the *inner* value (it is wrapped in
///   `Some` for you).
/// * `enum` / `bits` also export one `OPT_<FIELD>_<ITEM>` module constant per
///   variant/flag.
/// * `template(..)` lists the placeholders. `_` is the positional `{}`,
///   `name?` marks an optional placeholder (otherwise it is required).
///   After `;` comes how many placeholders the template may contain: `2` or
///   a range `1..=3`. A template field always needs a default.
///
/// ## Example
///
/// ```ignore
/// pyo3_constructor! {
///     LikeConfig => export_like_config_options {
///         escape:   bits LikeEscape { SCALAR, ANY } = empty,
///         any:      enum LikeAny { OrExpansion, PatternList } = OrExpansion,
///         bit:      Option<template(_; 1)> = None,
///         concat:   template(left, right; 2) = "{left} || {right}",
///         strict:   bool = false,
///         nested:   class OtherConfig = OtherConfig::default(),
///     }
/// }
/// ```
#[proc_macro]
pub fn pyo3_constructor(input: TokenStream) -> TokenStream {
    match pyo3_constructor::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
