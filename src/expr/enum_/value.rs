//! A Python object plus the [`Serializer`] used to emit it as SQL.

use pyo3::types::PyAnyMethods;
use pyo3::types::PyStringMethods;

use crate::dialect::PyDialect;
use crate::internal::alias;
use crate::internal::repr;
use crate::serializers::SerializerType;
use crate::serializers::serializer_repr;
use crate::sql::PrepareResult;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;

/// Python object plus the serializer that turns it into a SQL literal.
///
/// Parameterized SQL binds [`Self::object`] and ignores the serializer.
/// Raw SQL runs the serializer. Python `None` becomes `NULL`.
///
/// ```
/// pyo3::Python::attach(|py| {
///     let object = pyo3::types::PyString::new(py, "hi").into_any().unbind();
///     let value = rapidquery::expr::enum_::Value::new(object, rapidquery::serializers::SerializerType::String);
///     assert!(matches!(value.serializer, rapidquery::serializers::SerializerType::String));
/// });
/// ```
#[derive(Debug, Clone)]
pub struct Value {
    pub object: alias::PyObject,
    pub serializer: SerializerType,
}

impl Value {
    #[inline]
    pub fn new(object: alias::PyObject, serializer: SerializerType) -> Self {
        Self { object, serializer }
    }

    #[inline]
    pub fn infer_bound(bound: alias::BoundObject) -> pyo3::PyResult<Self> {
        SerializerType::infer_bound(&bound).map(|x| Self::new(bound.unbind(), x))
    }

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let obj_repr = self.object.bind(py).repr()?;
        let text = obj_repr.to_cow()?;

        let mut fmt = repr::ReprFormatter::new_call("Value");

        fmt.positional_raw(&text)
            .positional(serializer_repr(&self.serializer))
            .close();

        Ok(fmt)
    }
}

impl WriteSql for Value {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        out.push_value(dialect, &self.serializer, &self.object)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.serializer == other.serializer && self.object.as_ptr() == other.object.as_ptr()
    }
}
