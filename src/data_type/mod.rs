//! SQL data types: interned names, serializers, and the composed [`DataType`].
//!
//! A [`DataTypeName`] is a canonical spelling plus a [`DataTypeGroup`]. The
//! group is the inference key for which [`Serializer`] [`DataType::from_name`]
//! constructs when none is passed. Groups are serializer-oriented: `NUMERIC`
//! and `DECIMAL` sit in [`DataTypeGroup::Float`] because they share a
//! serializer with IEEE floats.

mod name;
mod serializer;

#[cfg(test)]
mod tests;

pub use name::{DataTypeGroup, DataTypeName};
pub use serializer::*;

use crate::internal::{alias, utils};

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
///
/// [`WriteSql`] emits the type name (CAST, column definitions). Literal
/// serialization of a Python object is [`Serializer::write_sql`], not this
/// type.
///
/// Clone of a [`Serializer::Custom`] value requires the GIL.
#[rapidquery_macros::pyclass(
    "DataType",
    from_py_object,
    module = "rapidquery._core.data_type",
    eq,
    hash,
    generic
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PyDataType {
    pub name: DataTypeName,
    pub serializer: SerializerEnum,
}

impl PyDataType {
    #[inline]
    pub fn new(name: DataTypeName, serializer: SerializerEnum) -> Self {
        Self { name, serializer }
    }

    /// Infer the serializer from [`DataTypeName::group`].
    ///
    /// Unrecognized and parameterized names use [`Serializer::Unspecified`].
    #[inline]
    pub fn from_name(name: impl Into<DataTypeName>) -> Self {
        let name = name.into();
        let serializer = SerializerEnum::from_group(name.group());
        Self { name, serializer }
    }
}

impl crate::sql::WriteSql for PyDataType {
    #[inline]
    fn write_sql<W: crate::sql::SqlWriter>(
        &self,
        dialect: &crate::dialect::SqlDialect,
        out: &mut W,
    ) -> crate::sql::PrepareResult<()> {
        self.name.write_sql(dialect, out)
    }
}

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
        match serializer {
            Some(PySerializer(s)) => PyDataType::new(DataTypeName::from(name), s),
            None => PyDataType::from_name(name),
        }
    }

    /// Returns the type name.
    ///
    /// Returns:
    ///     The canonical type name. For interned names this may differ in
    ///     casing or spacing from the string passed to the constructor.
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
        PySerializer(slf.serializer.clone()).into_subclasses(slf.py())
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
    ///
    /// Raises:
    ///     TypeError: If `object` is not an accepted type for this serializer.
    ///     ValueError: If `object` is the right type but cannot be represented.
    ///     NotImplementedError: If `dialect` has no native literal for this
    ///             type's serializer (bit strings, `ARRAY[…]`, inet, macaddr).
    fn serialize(
        &self,
        dialect: crate::dialect::py::PySqlDialect,
        object: alias::BoundObject<'_>,
    ) -> pyo3::PyResult<String> {
        let mut buf = String::new();
        self.serializer.write_sql(&dialect.0, &object, &mut buf)?;
        Ok(buf)
    }

    fn __repr__(&self) -> String {
        format!(
            "DataType({:?}, {})",
            self.name.as_str(),
            serializer_repr(&self.serializer)
        )
    }
}

/// An extractor for DataTypeName, which accepts `str` and `DataType`
pub struct DataTypeNameExtractor(pub DataTypeName);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for DataTypeNameExtractor {
    type Error = pyo3::PyErr;

    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        let ptr = obj.as_ptr();

        unsafe {
            // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
            // interned in `initialize_typeref` before any extractor runs.
            if crate::typeref::check(ptr, crate::typeref::DATATYPE_TYPE) {
                let dt = obj.cast_unchecked::<PyDataType>();
                return Ok(Self(dt.get().name.clone()));
            }

            if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
                let borrowed = utils::borrow_py_str_buffer(obj.py(), ptr)?;
                return Ok(Self(DataTypeName::from(borrowed)));
            }
        }

        rapidquery_macros::pyerr!(
            Err(PyTypeError),
            "expected Serializer or DataType, got {}",
            unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
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
