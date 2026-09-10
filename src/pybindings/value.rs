//! Python wrapper around [`crate::native::Value`].
//!
//! Native [`Value`](crate::native::Value) holds a Python object and a
//! [`Serializer`](crate::native::data_type::Serializer). This module owns
//! the pyclass:
//!
//! * [`PyValue`] — `rapidquery._core.Value`
//!
//! `FromPyObject` unwraps an existing `Value` or infers a serializer from
//! any other object. The constructor also accepts a `Serializer` or a
//! `DataType` (using only its serializer).

use std::hash::{Hash, Hasher};

use pyo3::types::{PyAnyMethods, PyStringMethods};

use crate::internal::alias;
use crate::native;
use crate::native::sql::write_sql;
use crate::pybindings::data_type::{PySerializer, SerializerExtractor, serializer_repr};

/// A Python object plus the serializer used to emit it as SQL.
///
/// `Value` does not convert the object until SQL is built. Parameterized
/// statements bind the object as a placeholder. Raw SQL runs the serializer
/// (`None` becomes `NULL`).
///
/// If `serializer` is omitted, one is inferred from the object's type.
/// Passing an existing `Value` keeps its object (and its serializer, unless
/// a new one is given). A `DataType` as `serializer` contributes only its
/// serializer; the data type is not stored.
#[rapidquery_macros::pyclass("Value", skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyValue(native::Value);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PyValue {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            if crate::typeref::check_exact(obj.as_ptr(), crate::typeref::VALUE_TYPE) {
                let value = obj.cast_unchecked::<Self>();
                return Ok(value.get().clone());
            }
        }

        Ok(Self(native::Value::infer_bound(obj.as_any().clone())))
    }
}

#[pyo3::pymethods]
impl PyValue {
    /// Creates a SQL value.
    ///
    /// Args:
    ///     object: The Python object to bind or serialize. If this is already
    ///             a `Value`, its object is reused.
    ///     serializer: How to render `object` as a SQL literal. A `DataType`
    ///             is accepted and only its serializer is kept. If omitted,
    ///             a serializer is inferred from `object` (or kept, when
    ///             `object` is already a `Value`).
    ///
    /// Raises:
    ///     TypeError: If `serializer` is not a `Serializer` or `DataType`.
    #[new]
    #[pyo3(signature = (object, serializer = None, /))]
    fn __new__(
        object: alias::BoundObject,
        serializer: Option<SerializerExtractor>,
    ) -> pyo3::PyClassInitializer<Self> {
        // handle Value(Value(...), new_serializer)
        if unsafe { crate::typeref::check_exact(object.as_ptr(), crate::typeref::VALUE_TYPE) } {
            let value = unsafe { object.cast_unchecked::<Self>() };
            let native_value = value.get().0.clone();

            let slf = match serializer {
                Some(SerializerExtractor(s)) => Self(native::Value::new(native_value.object, s)),
                None => Self(native_value),
            };
            return pyo3::PyClassInitializer::from(slf);
        }

        let slf = match serializer {
            Some(SerializerExtractor(s)) => Self(native::Value::new(object.unbind(), s)),
            None => Self(native::Value::infer_bound(object)),
        };
        pyo3::PyClassInitializer::from(slf)
    }

    /// Returns the wrapped Python object.
    ///
    /// Returns:
    ///     The object passed to the constructor (or taken from an existing
    ///     `Value`).
    #[getter]
    fn get_object(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        slf.0.object.clone_ref(slf.py())
    }

    /// Returns the serializer used for raw SQL literals.
    ///
    /// Returns:
    ///     The `Serializer` stored on this value. Not used when the value
    ///     is bound as a parameter.
    #[getter]
    fn get_serializer(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        PySerializer(slf.0.serializer.clone()).into_subclasses(slf.py())
    }

    /// Renders this value as a SQL literal for `dialect`.
    ///
    /// Raw path: `None` becomes `NULL`; otherwise the serializer writes a
    /// literal. This is not the parameterized form.
    ///
    /// Args:
    ///     dialect: A dialect class (`PostgreSQL`) or instance
    ///             (`PostgreSQL()`).
    ///
    /// Returns:
    ///     A SQL fragment suitable for embedding in a statement.
    ///
    /// Raises:
    ///     TypeError: If `object` is not an accepted type for the serializer.
    ///     ValueError: If `object` is the right type but cannot be represented.
    fn to_sql(
        &self,
        py: pyo3::Python<'_>,
        dialect: super::dialect::PySqlDialect,
    ) -> pyo3::PyResult<String> {
        write_sql(py, &self.0, &dialect.0).map_err(Into::into)
    }

    fn __eq__(&self, py: pyo3::Python<'_>, other: &pyo3::Bound<'_, Self>) -> pyo3::PyResult<bool> {
        if self.0.serializer != other.get().0.serializer {
            return Ok(false);
        }

        self.0.object.bind(py).eq(other.get().0.object.bind(py))
    }

    fn __hash__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<u64> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.0.object.bind(py).hash()?.hash(&mut hasher);
        self.0.serializer.hash(&mut hasher);
        Ok(hasher.finish())
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let obj_repr = self.0.object.bind(py).repr()?;
        Ok(format!(
            "Value({}, {})",
            obj_repr.to_cow()?,
            serializer_repr(&self.0.serializer)
        ))
    }
}
