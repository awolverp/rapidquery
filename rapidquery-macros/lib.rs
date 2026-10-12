use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;

mod dialect_options;

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
    let args: proc_macro2::TokenStream = format!("module = \"{module}\", name = \"{name}\", immutable_type, frozen")
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

/// Constructor arguments, getters, and `OPT_*` constants for dialect flags.
///
/// Invoked from `src/dialect/base.rs`. Each item is `char name = '?';`,
/// `enum Type name { Variant => OPT_..., }`, or `bool name = false;`.
#[proc_macro]
pub fn dialect_options(input: TokenStream) -> TokenStream {
    match dialect_options::generate(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct CustomizationProtocolInput {
    bound_object: syn::Ident,
    attribute_name: syn::LitCStr,
    pyobject_type: syn::Path,
    ok_expr: syn::Expr,
    err_expr: syn::Expr,
}

impl Parse for CustomizationProtocolInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // on: $bound:ident,
        let on_label: syn::Ident = input.parse()?;
        if on_label != "on" {
            return Err(input.error("expected `on`"));
        }
        input.parse::<syn::Token![:]>()?;
        let bound_object: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        // attr: $attribute_name:cstr,
        let attr_label: syn::Ident = input.parse()?;
        if attr_label != "attr" {
            return Err(input.error("expected `attr`"));
        }
        input.parse::<syn::Token![:]>()?;
        let attribute_name: syn::LitCStr = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        // ty: $pyobject_type:path,
        let ty_label: syn::Ident = input.parse()?;
        if ty_label != "ty" {
            return Err(input.error("expected `ty`"));
        }
        input.parse::<syn::Token![:]>()?;
        let pyobject_type: syn::Path = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        // matches: $ok_expr:expr,
        let matches_label: syn::Ident = input.parse()?;
        if matches_label != "matches" {
            return Err(input.error("expected `matches`"));
        }
        input.parse::<syn::Token![:]>()?;
        let ok_expr: syn::Expr = input.parse()?;
        input.parse::<syn::Token![,]>()?;

        // mismatches: $err_expr:expr
        let mismatches_label: syn::Ident = input.parse()?;
        if mismatches_label != "mismatches" {
            return Err(input.error("expected `mismatches`"));
        }
        input.parse::<syn::Token![:]>()?;
        let err_expr: syn::Expr = input.parse()?;

        // Optional trailing comma
        if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
        }

        Ok(Self {
            bound_object,
            attribute_name,
            pyobject_type,
            ok_expr,
            err_expr,
        })
    }
}

/// Builds a customization protocol checking.
///
/// Note: you can access to attribute result using `protocol` variable.
///
/// For example:
/// ```ignore
/// customization_protocol! {
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
///         let attribute: *mut pyo3::ffi::PyObject =
///             pyo3::ffi::PyObject_GetAttrString(bound.as_ptr(), PROTOCOL.as_ptr());
///
///         if attribute.is_null() {
///             if pyo3::ffi::PyErr_ExceptionMatches(pyo3::ffi::PyExc_AttributeError) != 1 {
///                 return Err(pyo3::PyErr::fetch(bound.py()));
///             } else {
///                 pyo3::ffi::PyErr_Clear();
///                 None
///             }
///         } else {
///             Some(pyo3::Bound::from_owned_ptr(bound.py(), attribute))
///         }
///     };
///
///     if let Some(protocol) = result {
///         if crate::typeref::check(protocol.as_ptr(), crate::typeref::SERIALIZER_TYPE) {
///             // OK expr
///         } else {
///             // ERR expr
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn cust_protocol(item: TokenStream) -> TokenStream {
    let CustomizationProtocolInput {
        bound_object,
        attribute_name,
        pyobject_type,
        ok_expr,
        err_expr,
    } = syn::parse_macro_input!(item as CustomizationProtocolInput);

    quote! {
        unsafe {
            const PROTOCOL: &'static std::ffi::CStr = #attribute_name;

            let result = {
                let attribute: *mut pyo3::ffi::PyObject =
                    pyo3::ffi::PyObject_GetAttrString(#bound_object.as_ptr(), PROTOCOL.as_ptr());

                if attribute.is_null() {
                    if pyo3::ffi::PyErr_ExceptionMatches(pyo3::ffi::PyExc_AttributeError) != 1 {
                        return Err(pyo3::PyErr::fetch(#bound_object.py()));
                    } else {
                        pyo3::ffi::PyErr_Clear();
                        None
                    }
                } else {
                    Some(pyo3::Bound::from_owned_ptr(#bound_object.py(), attribute))
                }
            };

            if let Some(protocol) = result {
                if pyo3::ffi::PyObject_TypeCheck(protocol.as_ptr(), #pyobject_type) == 1 #ok_expr
                else #err_expr
            }
        }
    }
    .into()
}
