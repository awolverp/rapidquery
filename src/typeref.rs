// Column types
pub(crate) static mut BLOB_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BINARY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut VAR_BINARY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut VAR_BIT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATETIME_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TIMESTAMP_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TIME_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATE_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut JSON_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut JSON_BINARY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DECIMAL_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut UUID_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INET_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MAC_ADDRESS_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BOOLEAN_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TINY_INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SMALL_INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject =
    std::ptr::null_mut();
pub(crate) static mut INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIG_INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TINY_UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject =
    std::ptr::null_mut();
pub(crate) static mut SMALL_UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject =
    std::ptr::null_mut();
pub(crate) static mut UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIG_UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut FLOAT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DOUBLE_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TEXT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut CHAR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STRING_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut VECTOR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ARRAY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Useful types
pub(crate) static mut VALUE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ASTERISK_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut EXPR_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut COLUMN_REF_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut FUNC_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TABLE_NAME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INDEX_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INDEX_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut FOREIGN_KEY_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TABLE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Python standard libraries types
pub(crate) static mut STD_DECIMAL_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_UUID_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_DATETIME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_DATE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_TIME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_ENUM_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

unsafe fn get_type_object_for<T: pyo3::PyTypeInfo>(
    py: pyo3::Python,
) -> *mut pyo3::ffi::PyTypeObject {
    T::type_object_raw(py)
}

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
#[cfg_attr(feature = "optimize", optimize(size))]
fn _initialize_typeref(py: pyo3::Python) {
    unsafe {
        multiple_get_type_object_for!(
            py,
            crate::sqltypes::PyBlobType => BLOB_COLUMN_TYPE,
            crate::sqltypes::PyBinaryType => BINARY_COLUMN_TYPE,
            crate::sqltypes::PyVarBinaryType => VAR_BINARY_COLUMN_TYPE,
            crate::sqltypes::PyBitType => BIT_COLUMN_TYPE,
            crate::sqltypes::PyVarBitType => VAR_BIT_COLUMN_TYPE,
            crate::sqltypes::PyDateTimeType => DATETIME_COLUMN_TYPE,
            crate::sqltypes::PyTimestampType => TIMESTAMP_COLUMN_TYPE,
            crate::sqltypes::PyTimeType => TIME_COLUMN_TYPE,
            crate::sqltypes::PyDateType => DATE_COLUMN_TYPE,
            crate::sqltypes::PyJSONType => JSON_COLUMN_TYPE,
            crate::sqltypes::PyJSONBinaryType => JSON_BINARY_COLUMN_TYPE,
            crate::sqltypes::PyDecimalType => DECIMAL_COLUMN_TYPE,
            crate::sqltypes::PyUUIDType => UUID_COLUMN_TYPE,
            crate::sqltypes::PyINETType => INET_COLUMN_TYPE,
            crate::sqltypes::PyMacAddressType => MAC_ADDRESS_COLUMN_TYPE,
            crate::sqltypes::PyBooleanType => BOOLEAN_COLUMN_TYPE,
            crate::sqltypes::PyTinyIntegerType => TINY_INTEGER_COLUMN_TYPE,
            crate::sqltypes::PySmallIntegerType => SMALL_INTEGER_COLUMN_TYPE,
            crate::sqltypes::PyIntegerType => INTEGER_COLUMN_TYPE,
            crate::sqltypes::PyBigIntegerType => BIG_INTEGER_COLUMN_TYPE,
            crate::sqltypes::PyTinyUnsignedType => TINY_UNSIGNED_COLUMN_TYPE,
            crate::sqltypes::PySmallUnsignedType => SMALL_UNSIGNED_COLUMN_TYPE,
            crate::sqltypes::PyUnsignedType => UNSIGNED_COLUMN_TYPE,
            crate::sqltypes::PyBigUnsignedType => BIG_UNSIGNED_COLUMN_TYPE,
            crate::sqltypes::PyFloatType => FLOAT_COLUMN_TYPE,
            crate::sqltypes::PyDoubleType => DOUBLE_COLUMN_TYPE,
            crate::sqltypes::PyTextType => TEXT_COLUMN_TYPE,
            crate::sqltypes::PyCharType => CHAR_COLUMN_TYPE,
            crate::sqltypes::PyStringType => STRING_COLUMN_TYPE,
            crate::sqltypes::PyVectorType => VECTOR_COLUMN_TYPE,
            crate::sqltypes::PyArrayType => ARRAY_COLUMN_TYPE,
            crate::value::PyValue => VALUE_TYPE,
            crate::common::Py_AsteriskType => ASTERISK_TYPE,
            crate::common::PyColumnRef => COLUMN_REF_TYPE,
            crate::common::PyTableName => TABLE_NAME_TYPE,
            crate::expression::PyExpr => EXPR_TYPE,
            crate::expression::PyFunc => FUNC_TYPE,
            crate::column::PyColumn => COLUMN_TYPE,
            crate::index::PyIndexColumn => INDEX_COLUMN_TYPE,
            crate::index::PyIndex => INDEX_TYPE,
            crate::foreign_key::PyForeignKey => FOREIGN_KEY_TYPE,
            crate::table::PyTable => TABLE_TYPE,
        );

        STD_DECIMAL_TYPE = look_up_type_object(c"decimal", c"Decimal");
        STD_UUID_TYPE = look_up_type_object(c"uuid", c"UUID");
        STD_ENUM_TYPE = look_up_type_object(c"enum", c"EnumMeta");

        pyo3::ffi::PyDateTime_IMPORT();
        let datetime_capsule = pyo3::ffi::PyCapsule_Import(c"datetime.datetime_CAPI".as_ptr(), 1)
            .cast::<pyo3::ffi::PyDateTime_CAPI>();

        STD_DATETIME_TYPE = (*datetime_capsule).DateTimeType;
        STD_DATE_TYPE = (*datetime_capsule).DateType;
        STD_TIME_TYPE = (*datetime_capsule).TimeType;
    }
}

pub fn initialize_typeref(py: pyo3::Python) {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| _initialize_typeref(py));
}
