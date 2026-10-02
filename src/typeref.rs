//! Interned `PyTypeObject*` cache.

// Dialect types
pub(crate) static mut DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Data-type / serializer types
pub(crate) static mut DATATYPE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Ident types
pub(crate) static mut IDENT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut UNQUOTE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ASTERISK_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Value type
pub(crate) static mut VALUE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Expr types
pub(crate) static mut EXPR_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Python standard-library types (serializers)
pub(crate) static mut STD_DECIMAL_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_UUID_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_DATETIME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_DATE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_TIME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

unsafe fn get_type_object_for<T: pyo3::PyTypeInfo>(
    py: pyo3::Python,
) -> *mut pyo3::ffi::PyTypeObject {
    // Borrowed type object; lives as long as the `#[pyclass]` is registered
    // on `rapidquery._core`, which is the process lifetime of this extension.
    T::type_object_raw(py)
}

/// Look up `module_name.member_name` and return its type object.
///
/// # Safety
///
/// `PyImport_ImportModule` / dict lookup are not null-checked. A missing
/// stdlib module currently yields a null pointer (and a later crash if
/// dereferenced). Call only during module init, when the stdlib is present.
unsafe fn look_up_type_object(
    module_name: &std::ffi::CStr,
    member_name: &std::ffi::CStr,
) -> *mut pyo3::ffi::PyTypeObject {
    let module = pyo3::ffi::PyImport_ImportModule(module_name.as_ptr());
    let module_dict = pyo3::ffi::PyObject_GenericGetDict(module, std::ptr::null_mut());
    let ptr = pyo3::ffi::PyMapping_GetItemString(module_dict, member_name.as_ptr())
        .cast::<pyo3::ffi::PyTypeObject>();

    pyo3::ffi::Py_DECREF(module_dict);
    pyo3::ffi::Py_DECREF(module);
    ptr
}

macro_rules! multiple_get_type_object_for {
    ($py:expr, $($type:ty => $name:ident,)*) => {
        $($name = get_type_object_for::<$type>($py);)*
    };
}

#[cold]
fn _initialize_typeref(py: pyo3::Python) {
    unsafe {
        // SAFETY: called once from [`initialize_typeref`] while the GIL is
        // held (module init). The `static mut` cells are still null; we
        // write each pointer exactly once. After this function returns they
        // are only read.
        multiple_get_type_object_for!(
            py,
            // dialects
            crate::dialect::PyDialect => DIALECT_TYPE,
            // data types / serializers
            crate::data_type::PyDataType => DATATYPE_TYPE,
            crate::data_type::PySerializer => SERIALIZER_TYPE,
            crate::expr::base::PyExpr => EXPR_TYPE,
            crate::expr::ident::PyIdent => IDENT_TYPE,
            crate::expr::ident::PyUnquote => UNQUOTE_TYPE,
            crate::expr::ident::PyAsterisk => ASTERISK_TYPE,
            crate::expr::value::PyValue => VALUE_TYPE,
        );

        STD_DECIMAL_TYPE = look_up_type_object(c"decimal", c"Decimal");
        STD_UUID_TYPE = look_up_type_object(c"uuid", c"UUID");

        pyo3::ffi::PyDateTime_IMPORT();
        let datetime_capsule = pyo3::ffi::PyCapsule_Import(c"datetime.datetime_CAPI".as_ptr(), 1)
            .cast::<pyo3::ffi::PyDateTime_CAPI>();

        STD_DATETIME_TYPE = (*datetime_capsule).DateTimeType;
        STD_DATE_TYPE = (*datetime_capsule).DateType;
        STD_TIME_TYPE = (*datetime_capsule).TimeType;
    }
}

/// Fill the type-pointer cache. Safe to call more than once; only the first
/// call does work. Must run from `#[pymodule_init]` before any extractor
/// that reads the `static mut` cells.
pub fn initialize_typeref(py: pyo3::Python) {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| _initialize_typeref(py));
}

/// `type(obj) is T` — no subclass match.
///
/// # Safety
///
/// `ptr` must be a valid, non-null `PyObject*`. `r#type` must be a valid
/// `PyTypeObject*` (interned pointers from [`initialize_typeref`] are fine).
/// The GIL must be held.
#[inline(always)]
pub unsafe fn check_exact(
    ptr: *mut pyo3::ffi::PyObject,
    r#type: *mut pyo3::ffi::PyTypeObject,
) -> bool {
    pyo3::ffi::Py_TYPE(ptr) == r#type
}

/// `isinstance(obj, T)`.
///
/// # Safety
///
/// `ptr` must be a valid, non-null `PyObject*`. `r#type` must be a valid
/// `PyTypeObject*` (interned pointers from [`initialize_typeref`] are fine).
/// The GIL must be held.
#[inline(always)]
pub unsafe fn check(ptr: *mut pyo3::ffi::PyObject, r#type: *mut pyo3::ffi::PyTypeObject) -> bool {
    pyo3::ffi::PyObject_TypeCheck(ptr, r#type) == 1
}
