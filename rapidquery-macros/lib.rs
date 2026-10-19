use proc_macro::TokenStream;

mod cust_protocol;
mod define_options;
mod pyclass;
mod pydialect_constructor;
mod pyerr;

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

/// Helps you to create DialectFlags.
///
/// ```ignore
/// #[rapidquery_macros::dialect_flags]
/// pub struct DialectFlags {
///     pub placeholder_character: char,
///     /// Positional parameter syntax
///     #[enum_(#[default] Simple, Numbered)]
///     pub placeholder_syntax: PlaceholderSyntax,
///     /// Like escape.
///     #[bit(SCALAR = 1 << 0, ANY = 1 << 1)]
///     pub like_escape: LikeEscape,
/// }
/// ```
#[proc_macro_attribute]
pub fn define_options(attr: TokenStream, item: TokenStream) -> TokenStream {
    match define_options::expand(attr.into(), item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn pydialect_constructor(input: TokenStream) -> TokenStream {
    match pydialect_constructor::expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
