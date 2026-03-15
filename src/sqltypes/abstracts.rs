/// All of native SQL types should implement this trait.
///
/// This is useful for generic typing.
pub trait SQLTypeTrait {
    /// Returns the related sea_query column type
    fn to_sea_query_column_type(&self) -> ::sea_query::ColumnType;

    /// Returns SQL type name in string (uses PostgreSQL builder)
    fn to_sql_type_name(&self) -> String {
        let mut name = String::with_capacity(10);
        let builder = sea_query::PostgresQueryBuilder;
        sea_query::TableBuilder::prepare_column_type(
            &builder,
            &self.to_sea_query_column_type(),
            &mut name,
        );
        name
    }

    /// Validates the `ptr` and makes sure that we can process it by using
    /// `serialize` method.
    ///
    /// ### Safety
    /// - The `ptr` should be borrowed.
    unsafe fn validate(
        &self,
        py: ::pyo3::Python,
        ptr: *mut ::pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()>;

    /// Converts PyObject into sea_query::Value
    ///
    /// ### Safety
    /// - The `ptr` should be borrowed.
    /// - The `ptr` should be validated by `validate` method.
    unsafe fn serialize(
        &self,
        py: ::pyo3::Python,
        ptr: *mut ::pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<::sea_query::Value>;

    /// Converts sea_query::Value into PyObject
    ///
    /// ### Safety
    /// - The returned object is owned and you should call Py_DECREF if you do
    ///   not need it.
    unsafe fn deserialize(
        &self,
        py: ::pyo3::Python,
        value: &::sea_query::Value,
    ) -> pyo3::PyResult<*mut ::pyo3::ffi::PyObject>;
}

crate::implement_pyclass! {
    /// Base class for all SQL column data types.
    ///
    /// This abstract base class represents SQL data types that can be used in
    /// column definitions. Each subclass implements a specific SQL data type
    /// with its particular characteristics, constraints, and backend-specific
    /// representations.
    #[derive(Debug, Clone, Copy)]
    [subclass, generic] PySQLTypeAbstract as "SQLTypeAbstract";
}

#[pyo3::pymethods]
impl PySQLTypeAbstract {
    /// Type name. e.g. `'INTEGER'`, `'STRING'`
    ///
    /// It also may be a property. This function must NOT raise any error.
    #[getter]
    fn __type_name__(&self) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(()))
    }
}

#[macro_export]
macro_rules! implement_sqltype_pymethods {
    ($name:ident) => {
        #[pyo3::pymethods]
        impl $name {
            #[new]
            fn __new__() -> (Self, PySQLTypeAbstract) {
                (Self, PySQLTypeAbstract)
            }

            /// Type name. e.g. `'INTEGER'`, `'STRING'`
            ///
            /// It also may be a property. This function must NOT raise any error.
            #[getter]
            fn __type_name__(&self) -> String {
                self.to_sql_type_name()
            }

            fn __repr__(&self) -> String {
                let result = String::from(stringify!($name));
                result[2..].to_string() + "()"
            }
        }
    };

    (
        $name:ident,
        init(|$param:ident: $param_type:ty| $init:expr),
        $return_type:literal
        $(, signature($($signature:tt)*))?
    ) => {
        #[pyo3::pymethods]
        impl $name {
            #[new]
            $(#[pyo3(signature=($($signature)*))])?
            fn __new__($param: $param_type) -> (Self, PySQLTypeAbstract) {
                ($init, PySQLTypeAbstract)
            }

            /// Type name. e.g. `'INTEGER'`, `'STRING'`
            ///
            /// It also may be a property. This function must NOT raise any error.
            #[getter]
            fn __type_name__(&self) -> String {
                self.to_sql_type_name()
            }

            #[getter]
            fn $param(&self) -> $param_type {
                self.0
            }

            fn __repr__(&self) -> String {
                let result = format!(
                    concat!(stringify!($name), "(", stringify!($param), "={:?})"),
                    self.0
                );
                result[2..].to_string()
            }
        }
    };
}
pub(super) use implement_sqltype_pymethods;
