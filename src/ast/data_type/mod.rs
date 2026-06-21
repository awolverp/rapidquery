use std::sync::Arc;

mod name;
pub use name::DataTypeName;

pub mod serializers;
pub use serializers::DataTypeSerializer;

use crate::internal::alias;

implement_pyclass! {
    #[derive(Debug, Clone)]
    [skip_from_py_object] PyDataType as "DataType" {
        pub name: DataTypeName,
        pub serializer: Arc<DataTypeSerializer>,
    }
}

#[pyo3::pymethods]
impl PyDataType {
    #[new]
    #[pyo3(signature = (name, serializer=None) )]
    fn __new__(name: String, serializer: Option<DataTypeSerializer>) -> Self {
        let name = DataTypeName::from(name.as_str());

        let serializer = match serializer {
            Some(x) => Arc::new(x),
            None => Arc::new(DataTypeSerializer::infer_from_name(&name)),
        };

        Self { name, serializer }
    }

    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    #[getter]
    fn serializer<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        self.serializer.to_pyobject(py)
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
