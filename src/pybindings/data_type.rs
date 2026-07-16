use crate::internal::alias;
use crate::native;

#[rapidquery_macros::pyclass_shortcut("Serializer", subclass, from_py_object)]
#[derive(Debug, Clone)]
pub struct PySerializer(pub native::data_type::DataTypeSerializer);

impl PySerializer {
    pub fn into_subclasses<'py>(
        self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        macro_rules! _match {
            (
                $value:expr,
                $py:expr,
                $self:expr,
                {
                    $(
                        $variant:pat => $py_ty:ty
                    ),* $(,)?
                }
            ) => {
                match $value {
                    $(
                        $variant => {
                            let result = pyo3::Bound::new(
                                $py,
                                pyo3::PyClassInitializer::from($self)
                                    .add_subclass(<$py_ty>::default()),
                            );
                            result.map(|x| x.into_any())
                        }
                    ),*
                }
            };
        }

        _match!(
            &self.0,
            py,
            self,
            {
                native::data_type::DataTypeSerializer::Integer => PyIntegerSerializer,
                native::data_type::DataTypeSerializer::Float => PyFloatSerializer,
                native::data_type::DataTypeSerializer::Boolean => PyBooleanSerializer,
                native::data_type::DataTypeSerializer::String => PyStringSerializer,
                native::data_type::DataTypeSerializer::Uuid => PyUuidSerializer,
                native::data_type::DataTypeSerializer::Bytes => PyBytesSerializer,
                native::data_type::DataTypeSerializer::Bit => PyBitSerializer,
                native::data_type::DataTypeSerializer::JSON => PyJSONSerializer,
                native::data_type::DataTypeSerializer::Date => PyDateSerializer,
                native::data_type::DataTypeSerializer::Time => PyTimeSerializer,
                native::data_type::DataTypeSerializer::Datetime => PyDatetimeSerializer,
                native::data_type::DataTypeSerializer::Inet => PyInetSerializer,
                native::data_type::DataTypeSerializer::MacAddress => PyMacAddressSerializer,
                native::data_type::DataTypeSerializer::Unspecified => PyUnspecifiedSerializer,
                native::data_type::DataTypeSerializer::Custom(_) => PyCustomSerializer,
                native::data_type::DataTypeSerializer::Array(_) => PyArraySerializer,
            }
        )
    }
}

#[pyo3::pymethods]
impl PySerializer {
    fn serialize(
        &self,
        dialect: super::dialect::PySqlDialect,
        object: alias::BoundObject,
    ) -> pyo3::PyResult<String> {
        self.0.serialize(&dialect.0, &object)
    }
}

macro_rules! impl_no_arg_serializers {
    ($($native:ident => $name:ident => $pyname:literal,)+) => {
        $(
            #[rapidquery_macros::pyclass_shortcut($pyname, extends=PySerializer, skip_from_py_object)]
            #[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name;

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[inline]
                fn __new__() -> pyo3::PyClassInitializer<Self> {
                    pyo3::PyClassInitializer::from(PySerializer(native::data_type::DataTypeSerializer::$native))
                        .add_subclass(Self)
                }
            }
        )+
    };
}

impl_no_arg_serializers! {
    Integer => PyIntegerSerializer => "IntegerSerializer",
    Float => PyFloatSerializer => "FloatSerializer",
    Boolean => PyBooleanSerializer => "BooleanSerializer",
    String => PyStringSerializer => "StringSerializer",
    Uuid => PyUuidSerializer => "UuidSerializer",
    Bytes => PyBytesSerializer => "BytesSerializer",
    Bit => PyBitSerializer => "BitSerializer",
    JSON => PyJSONSerializer => "JSONSerializer",
    Date => PyDateSerializer => "DateSerializer",
    Time => PyTimeSerializer => "TimeSerializer",
    Datetime => PyDatetimeSerializer => "DatetimeSerializer",
    Inet => PyInetSerializer => "InetSerializer",
    MacAddress => PyMacAddressSerializer => "MacAddressSerializer",
    Unspecified => PyUnspecifiedSerializer => "UnspecifiedSerializer",
}

#[rapidquery_macros::pyclass_shortcut("ArraySerializer", extends=PySerializer, skip_from_py_object)]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyArraySerializer;

#[pyo3::pymethods]
impl PyArraySerializer {
    #[new]
    #[inline]
    fn __new__(element: PySerializer) -> pyo3::PyClassInitializer<Self> {
        let inner = native::data_type::DataTypeSerializer::Array(Box::new(element.0));
        pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self)
    }

    #[getter]
    fn get_element<'py>(slf: pyo3::PyRef<'py, Self>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let baseclass = slf.as_super();

        if let native::data_type::DataTypeSerializer::Array(element) = &baseclass.0 {
            PySerializer(*element.clone()).into_subclasses(slf.py())
        } else {
            unreachable!()
        }
    }
}

#[rapidquery_macros::pyclass_shortcut("CustomSerializer", extends=PySerializer, skip_from_py_object)]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyCustomSerializer;

#[pyo3::pymethods]
impl PyCustomSerializer {
    #[new]
    #[inline]
    fn __new__(function: alias::PyObject) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        unsafe {
            if pyo3::ffi::PyCallable_Check(function.as_ptr()) != 1 {
                return new_pyerr!(
                    Err(PyTypeError),
                    "the 'function' parameter must be callable"
                );
            }
        }

        let inner = native::data_type::DataTypeSerializer::Custom(function);
        Ok(pyo3::PyClassInitializer::from(PySerializer(inner)).add_subclass(Self))
    }

    #[getter]
    #[inline]
    fn get_function<'py>(slf: pyo3::PyRef<'py, Self>) -> alias::PyObject {
        let baseclass = slf.as_super();

        if let native::data_type::DataTypeSerializer::Custom(function) = &baseclass.0 {
            function.clone()
        } else {
            unreachable!()
        }
    }
}

#[rapidquery_macros::pyclass_shortcut("DataType", skip_from_py_object)]
#[derive(Debug)]
pub struct PyDataType {
    name: native::data_type::DataTypeName,
    serializer: PySerializer,
}

#[pyo3::pymethods]
impl PyDataType {
    #[new]
    #[pyo3(signature = (name, serializer = None, /))]
    fn __new__(name: String, serializer: Option<PySerializer>) -> pyo3::PyClassInitializer<Self> {
        let name = native::data_type::DataTypeName::from(name);

        let serializer = match serializer {
            Some(x) => x,
            None => {
                let infer = native::data_type::DataTypeSerializer::infer_from_name(&name);
                PySerializer(infer)
            }
        };

        pyo3::PyClassInitializer::from(Self { name, serializer })
    }

    #[getter]
    fn get_name(&self) -> String {
        self.name.clone().into()
    }

    #[getter]
    fn get_serializer<'py>(
        &self,
        py: pyo3::prelude::Python<'py>,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        self.serializer.clone().into_subclasses(py)
    }

    fn serialize(
        &self,
        dialect: super::dialect::PySqlDialect,
        object: alias::BoundObject,
    ) -> pyo3::PyResult<String> {
        self.serializer.serialize(dialect, object)
    }
}

#[pyo3::pymodule(name = "data_type", gil_used = false)]
pub mod data_type_mod {
    #[pymodule_export]
    use super::{
        PyArraySerializer, PyBitSerializer, PyBooleanSerializer, PyBytesSerializer,
        PyCustomSerializer, PyDataType, PyDateSerializer, PyDatetimeSerializer, PyFloatSerializer,
        PyInetSerializer, PyIntegerSerializer, PyJSONSerializer, PyMacAddressSerializer,
        PySerializer, PyStringSerializer, PyTimeSerializer, PyUnspecifiedSerializer,
        PyUuidSerializer,
    };
}
