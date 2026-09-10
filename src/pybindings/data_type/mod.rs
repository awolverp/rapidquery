//! Python wrappers for SQL data types and value serializers.
//!
//! Native [`DataTypeName`](crate::native::data_type::DataTypeName) is a
//! spelling plus group. Native [`Serializer`](crate::native::data_type::Serializer)
//! writes Python objects as SQL literals. Native [`DataType`](crate::native::DataType)
//! is the name plus serializer; this module owns the pyclasses.
//!
//! Python sees:
//!
//! * [`PySerializer`] — `rapidquery._core.data_type.Serializer`, the ABC
//! * frozen subclasses per behaviour (`IntegerSerializer`, `JSONSerializer`, …)
//! * [`PyArraySerializer`] / [`PyCustomSerializer`]
//! * [`PyDataType`] — `rapidquery._core.data_type.DataType`
//!
//! `serialize(dialect, obj)` accepts a dialect **class or instance**. Serializer
//! extract is instance-only (Array/Custom carry state).

mod serializer;

use crate::internal::alias;
use crate::native;

pub(crate) use serializer::serializer_repr;
pub use serializer::{
    PyArraySerializer, PyBitSerializer, PyBooleanSerializer, PyBytesSerializer, PyCustomSerializer,
    PyDateSerializer, PyDatetimeSerializer, PyFloatSerializer, PyInetSerializer,
    PyIntegerSerializer, PyJSONSerializer, PyMacAddressSerializer, PySerializer,
    PyStringSerializer, PyTimeSerializer, PyUnspecifiedSerializer, PyUuidSerializer,
};

/// `Serializer` instance, or a `DataType` (its serializer only).
pub struct SerializerExtractor(pub crate::native::data_type::Serializer);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for SerializerExtractor {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        let ptr = obj.as_ptr();

        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
            // interned in `initialize_typeref` before any extractor runs.
            if crate::typeref::check(ptr, crate::typeref::SERIALIZER_TYPE) {
                let serializer = obj.cast_unchecked::<PySerializer>();
                return Ok(Self(serializer.get().0.clone()));
            }

            if crate::typeref::check_exact(ptr, crate::typeref::DATATYPE_TYPE) {
                let data_type = obj.cast_unchecked::<PyDataType>();
                return Ok(Self(data_type.get().0.serializer.clone()));
            }
        }

        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected Serializer or DataType, got {}",
            unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
        )
    }
}

/// Represents a SQL data type.
///
/// A data type is a SQL name plus the serializer used to turn Python objects
/// into SQL literals. The name can be any string; it is not limited to a set
/// of predefined values. The serializer is fully customizable.
///
/// Known names are interned and canonicalized (`"int"` becomes `"INT"`).
/// Unknown names and parameterized spellings (`VARCHAR(255)`) are kept as
/// given. Interning applies to the **name string**, not to `DataType`
/// instances: `DataType("int")` is not a singleton.
///
/// If `serializer` is omitted at construction, one is inferred from the
/// name; unrecognized names get `UnspecifiedSerializer`.
#[rapidquery_macros::pyclass(
    "DataType",
    eq,
    hash,
    from_py_object,
    module = "rapidquery._core.data_type"
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyDataType(pub native::DataType);

#[pyo3::pymethods]
impl PyDataType {
    /// Creates a new SQL data type.
    ///
    /// Args:
    ///     name: The SQL type spelling. Not limited to a predefined set.
    ///             Known names are interned and canonicalized (`"int"`
    ///             becomes `"INT"`). Unknown names and parameterized
    ///             spellings (`VARCHAR(255)`) are kept as given.
    ///     serializer: Used when a Python object is rendered as a SQL
    ///             literal (defaults, inserts, raw SQL). Built-in
    ///             serializers cover common types; `CustomSerializer`
    ///             covers the rest. If omitted, a serializer is inferred
    ///             from `name`; unrecognized names get
    ///             `UnspecifiedSerializer`.
    #[new]
    #[pyo3(signature = (name, serializer = None, /))]
    fn __new__(name: String, serializer: Option<PySerializer>) -> Self {
        let inner = match serializer {
            Some(PySerializer(s)) => {
                native::DataType::new(native::data_type::DataTypeName::from(name), s)
            }
            None => native::DataType::from_name(name),
        };
        Self(inner)
    }

    /// Returns the type name.
    ///
    /// Returns:
    ///     The canonical type name. For interned names this may differ in
    ///     casing or spacing from the string passed to the constructor.
    #[getter]
    fn get_name(&self) -> &str {
        self.0.name.as_str()
    }

    /// Returns the serializer used to render Python values as SQL literals.
    ///
    /// Returns:
    ///     The `Serializer` associated with this data type.
    #[getter]
    fn get_serializer(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        PySerializer(slf.0.serializer.clone()).into_subclasses(slf.py())
    }

    /// Renders `object` as a SQL literal for `dialect`.
    ///
    /// Shorthand for `self.serializer.serialize(dialect, object)`.
    ///
    /// Args:
    ///     dialect: A dialect class (`PostgreSQL`) or instance (`PostgreSQL()`).
    ///     object: The Python value to convert.
    ///
    /// Returns:
    ///     A SQL fragment produced by this type's serializer.
    fn serialize(
        &self,
        dialect: super::dialect::PySqlDialect,
        object: alias::BoundObject<'_>,
    ) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.0.serializer.write_sql(&dialect.0, &object, &mut buf)?;
        Ok(buf)
    }

    fn __repr__(&self) -> String {
        format!(
            "DataType({:?}, {})",
            self.0.name.as_str(),
            serializer_repr(&self.0.serializer)
        )
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
