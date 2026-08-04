// Dialect types
pub(crate) static mut BIGQUERY_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATABRICKS_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DUCKDB_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut HIVE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MSSQL_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MYSQL_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ORACLE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut POSTGRESQL_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut REDSHIFT_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SNOWFLAKE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SPARK_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SQLITE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TERADATA_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Date type serializer types
pub(crate) static mut DATATYPE_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

// Ident types
pub(crate) static mut IDENT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

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
fn _initialize_typeref(py: pyo3::Python) {
    unsafe {
        multiple_get_type_object_for!(
            py,
            // dialects
            crate::pybindings::dialect::PyBigQuery => BIGQUERY_DIALECT_TYPE,
            crate::pybindings::dialect::PyDatabricks => DATABRICKS_DIALECT_TYPE,
            crate::pybindings::dialect::PyDuckDb => DUCKDB_DIALECT_TYPE,
            crate::pybindings::dialect::PyHive => HIVE_DIALECT_TYPE,
            crate::pybindings::dialect::PyMsSql => MSSQL_DIALECT_TYPE,
            crate::pybindings::dialect::PyMySql => MYSQL_DIALECT_TYPE,
            crate::pybindings::dialect::PyOracle => ORACLE_DIALECT_TYPE,
            crate::pybindings::dialect::PyPostgreSql => POSTGRESQL_DIALECT_TYPE,
            crate::pybindings::dialect::PyRedshift => REDSHIFT_DIALECT_TYPE,
            crate::pybindings::dialect::PySnowflake => SNOWFLAKE_DIALECT_TYPE,
            crate::pybindings::dialect::PySpark => SPARK_DIALECT_TYPE,
            crate::pybindings::dialect::PySqlite => SQLITE_DIALECT_TYPE,
            crate::pybindings::dialect::PyTeradata => TERADATA_DIALECT_TYPE,
            // data type serializers
            crate::pybindings::data_type::PySerializer => DATATYPE_SERIALIZER_TYPE,
            // Ident types
            crate::pybindings::PyIdent => IDENT_TYPE,
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

#[inline]
pub unsafe fn check_exact(
    ptr: *mut pyo3::ffi::PyObject,
    r#type: *mut pyo3::ffi::PyTypeObject,
) -> bool {
    pyo3::ffi::Py_TYPE(ptr) == r#type
}

#[inline]
pub unsafe fn check(ptr: *mut pyo3::ffi::PyObject, r#type: *mut pyo3::ffi::PyTypeObject) -> bool {
    pyo3::ffi::PyObject_TypeCheck(ptr, r#type) == 1
}
