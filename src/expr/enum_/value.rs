//! A Python object plus the [`Serializer`] used to emit it as SQL.

use pyo3::types::PyAnyMethods;
use pyo3::types::PyStringMethods;

use crate::data_type::SerializerType;
use crate::data_type::serializer_repr;
use crate::dialect::PySqlDialect;
use crate::internal::alias;
use crate::sql::PrepareResult;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;

/// A Python object plus the [`Serializer`] used to emit it as SQL.
///
/// Parameterized SQL binds [`Self::object`] as a placeholder and ignores the
/// serializer. Raw SQL (`BufferedSql`) calls [`SerializerType::serialize`];
/// Python `None` becomes `NULL`.
///
/// Clone requires the GIL (`Py<T>::clone`).
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
    pub fn infer_bound(bound: alias::BoundObject) -> Self {
        let serializer = crate::data_type::SerializerType::infer_bound(&bound);
        Self::new(bound.unbind(), serializer)
    }

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let obj_repr = self.object.bind(py).repr()?;
        Ok(format!(
            "Value({}, {})",
            obj_repr.to_cow()?,
            serializer_repr(&self.serializer)
        ))
    }
}

impl WriteSql for Value {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        out.push_value(dialect, &self.serializer, &self.object)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.serializer == other.serializer && self.object.as_ptr() == other.object.as_ptr()
    }
}
