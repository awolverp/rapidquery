//! Python wrappers for SQL data types and value serializers.
//!
//! Native [`DataTypeName`](crate::native::data_type::DataTypeName) is a
//! spelling plus group. Native [`Serializer`](crate::native::Serializer)
//! writes Python objects as SQL literals. This module owns the pyclasses.
//!
//! Python sees:
//!
//! * [`PySerializer`] — `rapidquery._core.data_type.Serializer`, the ABC
//! * frozen subclasses per behaviour (`IntegerSerializer`, `JSONSerializer`, …)
//! * [`PyArraySerializer`] / [`PyCustomSerializer`]
//! * [`PyDataType`] — a name plus a serializer
//!
//! `serialize(dialect, obj)` accepts a dialect **class or instance**. Serializer
//! extract is instance-only (Array/Custom carry state).

mod serializer;

use crate::internal::alias;
use crate::native::data_type::DataTypeName;

pub use serializer::{
    PyArraySerializer, PyBitSerializer, PyBooleanSerializer, PyBytesSerializer, PyCustomSerializer,
    PyDateSerializer, PyDatetimeSerializer, PyFloatSerializer, PyInetSerializer,
    PyIntegerSerializer, PyJSONSerializer, PyMacAddressSerializer, PySerializer,
    PyStringSerializer, PyTimeSerializer, PyUnspecifiedSerializer, PyUuidSerializer,
};

/// Represents a SQL data type.
///
/// A data type is a SQL name plus the serializer used to turn Python objects
/// into SQL literals. The name can be any string; it is not limited to a set
/// of predefined values. The serializer is fully customizable.
///
/// Known names are interned and canonicalized (`"int"` becomes `"INT"`).
/// Unknown names and parameterized spellings (`VARCHAR(255)`) are kept as
/// given. If `serializer` is omitted at construction, one is inferred from
/// the name; unrecognized names get `UnspecifiedSerializer`.
#[rapidquery_macros::pyclass("DataType", module = "rapidquery._core.data_type")]
#[derive(Debug)]
pub struct PyDataType {
    name: DataTypeName,
    serializer: PySerializer,
}

#[pyo3::pymethods]
impl PyDataType {
    /// Creates a new SQL data type.
    ///
    /// Args:
    ///     name: The name of a data type. *It is not limited to a set of predefined values.*
    ///
    ///             **Important memory optimization note**: At import time, the library pre-allocates
    ///             and interns (reserves in memory) several simple and commonly used data types
    ///             (e.g. `INTEGER`, `VARCHAR`, `TIMESTAMP`, `BOOLEAN`, etc.). If you instantiate
    ///             a `DataType` using one of these common names, the library will automatically
    ///             return a reference to the pre-allocated instance instead of creating a new one.
    ///             This reduces memory consumption and improves performance when working with
    ///             standard SQL types.
    ///
    ///             As a result of this interning mechanism, when you access the `.name` property
    ///             on a pre-allocated instance, the returned string **may be slightly different**
    ///             from the exact string you passed in (for example in casing, spacing, or canonical
    ///             form chosen by the library). This is expected behavior and does not affect
    ///             correctness.
    ///
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
    #[pyo3(signature = (name, serializer = None, /))]
    fn __new__(name: String, serializer: Option<PySerializer>) -> pyo3::PyClassInitializer<Self> {
        let name = DataTypeName::from(name);
        let serializer = match serializer {
            Some(s) => s,
            None => PySerializer::inferred(name.group()),
        };
        pyo3::PyClassInitializer::from(Self { name, serializer })
    }

    /// Returns the type name.
    ///
    /// Returns:
    ///     The canonical type name.
    ///
    /// Note:
    ///     For interned common data types (see `__new__`), the returned value
    ///     may be a canonicalized version of the original input string (for
    ///     example in casing or spacing).
    #[getter]
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the serializer used to render Python values as SQL literals.
    ///
    /// Returns:
    ///     The `Serializer` associated with this data type.
    #[getter]
    fn get_serializer(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        slf.serializer.clone().into_subclasses(slf.py())
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
        self.serializer.serialize_value(dialect, object)
    }

    fn __repr__(&self) -> String {
        format!("DataType({:?})", self.name.as_str())
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
