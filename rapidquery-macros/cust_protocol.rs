use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;

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

pub fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let CustomizationProtocolInput {
        bound_object,
        attribute_name,
        pyobject_type,
        ok_expr,
        err_expr,
    } = syn::parse2(input)?;

    Ok(quote! {
        unsafe {
            const PROTOCOL: &'static std::ffi::CStr = #attribute_name;

            #[cfg(Py_3_13)]
            let result = {
                let mut attribute: *mut pyo3::ffi::PyObject = std::ptr::null_mut();

                match pyo3::ffi::PyObject_GetOptionalAttrString(
                    #bound_object.as_ptr(),
                    PROTOCOL.as_ptr(),
                    &mut attribute
                ) {
                    1 => Some(pyo3::Bound::from_owned_ptr(#bound_object.py(), attribute)),
                    0 => None,
                    _ => {
                        return Err(pyo3::PyErr::fetch(#bound_object.py()).into())
                    }
                }
            };

            #[cfg(not(Py_3_13))]
            let result = {
                let attribute: *mut pyo3::ffi::PyObject =
                    pyo3::ffi::PyObject_GetAttrString(#bound_object.as_ptr(), PROTOCOL.as_ptr());

                if attribute.is_null() {
                    if pyo3::ffi::PyErr_ExceptionMatches(pyo3::ffi::PyExc_AttributeError) != 1 {
                        return Err(pyo3::PyErr::fetch(#bound_object.py()).into());
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
    })
}
