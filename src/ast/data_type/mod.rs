use std::sync::Arc;

mod name;
pub use name::DataTypeName;

pub mod serializers;
pub use serializers::{DataTypeSerializerUnion, Serialize};

use crate::dialect::DialectUnion;
use crate::internal::alias;

implement_pyclass! {
    /// Reperesents a SQL data type. It includes a name, and a serializer.
    ///
    /// The name of a SQL data type can be any string; it is not limited to a set of predefined values.
    /// Additionally, the serializer for each data type is fully customizable.
    #[derive(Debug, Clone)]
    [skip_from_py_object] PyDataType as "DataType" {
        pub name: DataTypeName,
        pub serializer: Arc<DataTypeSerializerUnion>,
    }
}

#[pyo3::pymethods]
impl PyDataType {
    /// Creates a new SQL data type.
    ///
    /// Args:
    ///     name: The name of a data type. it is not limited to a set of predefined values.
    ///     serializer: The `serializer` is used whenever a Python object needs to be converted
    ///                 into a valid SQL value during query construction. This is commonly required
    ///                 when setting default values, inserting data, or generating other SQL statements.
    ///                 You can use one of the built-in serializers (we provide ready-to-use
    ///                 serializers for most common data types), or you can create and register your
    ///                 own custom serializer. This parameter is optional. However, it is strongly
    ///                 recommended to always specify it explicitly. If you do not provide a serializer,
    ///                 the `DataType` will attempt to automatically detect and assign one based on the
    ///                 `name`. If no suitable serializer is found, it will fall back to `UnspecifiedSerializer`.
    #[new]
    #[pyo3(signature = (name, serializer=None) )]
    fn __new__(name: String, serializer: Option<DataTypeSerializerUnion>) -> Self {
        let name = DataTypeName::from(name.as_str());

        let serializer = match serializer {
            Some(x) => Arc::new(x),
            None => Arc::new(DataTypeSerializerUnion::infer_from_name(&name)),
        };

        Self { name, serializer }
    }

    /// Returns data type name.
    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    /// Returns data type serializer.
    #[getter]
    fn serializer<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        self.serializer.to_pyobject(py)
    }

    /// Uses configured serializer to serialize `value` to SQL, depend on the specified `dialect`.
    fn serialize(
        &self,
        py: pyo3::Python,
        dialect: DialectUnion,
        value: alias::PyObject,
    ) -> pyo3::PyResult<String> {
        unsafe { self.serializer.serialize(py, value.as_ptr(), &dialect) }
    }

    fn __repr__(&self) -> String {
        format!(
            "DataType(name={}, serializer={:?})",
            self.name, self.serializer,
        )
    }
}

#[pyo3::pymodule(name = "data_type")]
pub mod data_type_module {
    #[pymodule_export]
    pub use super::PyDataType;

    #[pymodule_export]
    pub use super::serializers::{
        PyArraySerializer, PyBitSerializer, PyBooleanSerializer, PyBytesSerializer,
        PyDateSerializer, PyDatetimeSerializer, PyFloatSerializer, PyInetSerializer,
        PyIntegerSerializer, PyJSONSerializer, PyMacAddressSerializer, PyStringSerializer,
        PyTimeSerializer, PyUnspecifiedSerializer, PyUuidSerializer,
    };
}
