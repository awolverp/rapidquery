// Serializer types
pub(crate) static mut INTEGER_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut FLOAT_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BOOLEAN_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut STRING_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut UUID_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BYTES_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut BIT_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut JSON_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATE_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TIME_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATETIME_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut INET_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MAC_ADDRESS_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject =
    std::ptr::null_mut();
pub(crate) static mut ARRAY_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut UNSPECIFIED_SERIALIZER_TYPE: *mut pyo3::ffi::PyTypeObject =
    std::ptr::null_mut();

// Dialect types
pub(crate) static mut BIGQUERY_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DATABRICKS_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut DUCKDB_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut HIVE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MS_SQL_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut MYSQL_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut ORACLE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut POSTGRESQL_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut REDSHIFT_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SNOWFLAKE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SPARK_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut SQLITE_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();
pub(crate) static mut TERADATA_DIALECT_TYPE: *mut pyo3::ffi::PyTypeObject = std::ptr::null_mut();

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
#[optimize(size)]
fn _initialize_typeref(py: pyo3::Python) {
    unsafe {
        multiple_get_type_object_for!(
            py,
            // serializers
            crate::ast::data_type::serializers::PyIntegerSerializer => INTEGER_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyFloatSerializer => FLOAT_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyBooleanSerializer => BOOLEAN_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyStringSerializer => STRING_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyUuidSerializer => UUID_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyBytesSerializer => BYTES_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyBitSerializer => BIT_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyJSONSerializer => JSON_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyDateSerializer => DATE_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyTimeSerializer => TIME_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyDatetimeSerializer => DATETIME_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyInetSerializer => INET_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyMacAddressSerializer => MAC_ADDRESS_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyArraySerializer => ARRAY_SERIALIZER_TYPE,
            crate::ast::data_type::serializers::PyUnspecifiedSerializer => UNSPECIFIED_SERIALIZER_TYPE,
            // dialects
            crate::dialect::PyBigQueryDialect => BIGQUERY_DIALECT_TYPE,
            crate::dialect::PyDatabricksDialect => DATABRICKS_DIALECT_TYPE,
            crate::dialect::PyDuckDbDialect => DUCKDB_DIALECT_TYPE,
            crate::dialect::PyHiveDialect => HIVE_DIALECT_TYPE,
            crate::dialect::PyMsSqlDialect => MS_SQL_DIALECT_TYPE,
            crate::dialect::PyMySqlDialect => MYSQL_DIALECT_TYPE,
            crate::dialect::PyOracleDialect => ORACLE_DIALECT_TYPE,
            crate::dialect::PyPostgreSqlDialect => POSTGRESQL_DIALECT_TYPE,
            crate::dialect::PyRedshiftDialect => REDSHIFT_DIALECT_TYPE,
            crate::dialect::PySnowflakeDialect => SNOWFLAKE_DIALECT_TYPE,
            crate::dialect::PySparkDialect => SPARK_DIALECT_TYPE,
            crate::dialect::PySQLiteDialect => SQLITE_DIALECT_TYPE,
            crate::dialect::PyTeradataDialect => TERADATA_DIALECT_TYPE,
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
