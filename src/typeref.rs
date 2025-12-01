use once_cell::race::OnceBool;

// Column types
pub(crate) static mut TINY_INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SMALL_INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIG_INTEGER_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TINY_UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SMALL_UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIG_UNSIGNED_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TEXT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut FLOAT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DOUBLE_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DECIMAL_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATETIME_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TIMESTAMP_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TIMESTAMP_WITH_TIMEZONE_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject =
    std::ptr::null_mut();
pub(crate) static mut TIME_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATE_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut YEAR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BLOB_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BINARY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut VAR_BINARY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut VAR_BIT_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BOOLEAN_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MONEY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut JSON_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut JSON_BINARY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut UUID_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut CIDR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INET_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MAC_ADDR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut LTREE_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut CHAR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STRING_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut VECTOR_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INTERVAL_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ENUM_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ARRAY_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Other types
pub(crate) static mut ASTERISK_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ADAPTED_VALUE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut COLUMN_REF_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut FUNCTION_CALL_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut EXPR_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TABLE_NAME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INDEX_COLUMN_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SELECT_STATEMENT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut CASE_STATEMENT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Python standard libraries types
pub(crate) static mut STD_DECIMAL_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_UUID_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_DATETIME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_DATE_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_TIME_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STD_ENUM_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

unsafe fn get_type_object_for<T: pyo3::PyTypeInfo>(py: pyo3::Python) -> *mut pyo3::ffi::PyTypeObject {
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

#[cold]
#[optimize(size)]
fn _initialize_typeref(py: pyo3::Python) -> bool {
    unsafe {
        CHAR_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyCharType>(py);
        STRING_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyStringType>(py);
        TEXT_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyTextType>(py);
        TINY_INTEGER_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyTinyIntegerType>(py);
        SMALL_INTEGER_COLUMN_TYPE = get_type_object_for::<crate::column::types::PySmallIntegerType>(py);
        INTEGER_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyIntegerType>(py);
        BIG_INTEGER_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyBigIntegerType>(py);
        TINY_UNSIGNED_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyTinyUnsignedType>(py);
        SMALL_UNSIGNED_COLUMN_TYPE = get_type_object_for::<crate::column::types::PySmallUnsignedType>(py);
        UNSIGNED_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyUnsignedType>(py);
        BIG_UNSIGNED_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyBigUnsignedType>(py);
        FLOAT_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyFloatType>(py);
        DOUBLE_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyDoubleType>(py);
        DECIMAL_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyDecimalType>(py);
        DATETIME_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyDateTimeType>(py);
        TIMESTAMP_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyTimestampType>(py);
        TIMESTAMP_WITH_TIMEZONE_COLUMN_TYPE =
            get_type_object_for::<crate::column::types::PyTimestampWithTimeZoneType>(py);
        TIME_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyTimeType>(py);
        DATE_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyDateType>(py);
        YEAR_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyYearType>(py);
        BLOB_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyBlobType>(py);
        BINARY_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyBinaryType>(py);
        VAR_BINARY_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyVarBinaryType>(py);
        BIT_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyBitType>(py);
        VAR_BIT_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyVarBitType>(py);
        BOOLEAN_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyBooleanType>(py);
        MONEY_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyMoneyType>(py);
        JSON_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyJsonType>(py);
        JSON_BINARY_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyJsonBinaryType>(py);
        UUID_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyUuidType>(py);
        VECTOR_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyVectorType>(py);
        CIDR_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyCidrType>(py);
        INET_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyInetType>(py);
        MAC_ADDR_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyMacAddressType>(py);
        LTREE_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyLTreeType>(py);
        INTERVAL_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyIntervalType>(py);
        ENUM_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyEnumType>(py);
        ARRAY_COLUMN_TYPE = get_type_object_for::<crate::column::types::PyArrayType>(py);

        ASTERISK_TYPE = get_type_object_for::<crate::common::PyAsteriskType>(py);
        ADAPTED_VALUE_TYPE = get_type_object_for::<crate::adaptation::PyAdaptedValue>(py);
        COLUMN_REF_TYPE = get_type_object_for::<crate::common::PyColumnRef>(py);
        FUNCTION_CALL_TYPE = get_type_object_for::<crate::expression::PyFunctionCall>(py);
        EXPR_TYPE = get_type_object_for::<crate::expression::PyExpr>(py);
        TABLE_NAME_TYPE = get_type_object_for::<crate::common::PyTableName>(py);
        COLUMN_TYPE = get_type_object_for::<crate::column::PyColumn>(py);
        INDEX_COLUMN_TYPE = get_type_object_for::<crate::common::PyIndexColumn>(py);
        SELECT_STATEMENT_TYPE = get_type_object_for::<crate::query::select::PySelect>(py);
        CASE_STATEMENT_TYPE = get_type_object_for::<crate::query::case::PyCase>(py);

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

    true
}

static INIT: OnceBool = OnceBool::new();

pub fn initialize_typeref(py: pyo3::Python) {
    INIT.get_or_init(|| _initialize_typeref(py));
}
