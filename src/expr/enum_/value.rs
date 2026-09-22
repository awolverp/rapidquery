//! A Python object plus the [`Serializer`] used to emit it as SQL.

use pyo3::types::{PyAnyMethods, PyStringMethods};

use crate::data_type::{SerializerEnum, serializer_repr};
use crate::dialect::PySqlDialect;
use crate::internal::alias;
use crate::sql::{PrepareResult, SqlWriter, WriteSql};

/// A Python object plus the [`Serializer`] used to emit it as SQL.
///
/// Parameterized SQL binds [`Self::object`] as a placeholder and ignores the
/// serializer. Raw SQL (`BufferedSql`) calls [`Serializer::write_sql`];
/// Python `None` becomes `NULL`.
///
/// Clone requires the GIL (`Py<T>::clone`).
#[derive(Debug, Clone)]
pub struct Value {
    pub object: alias::PyObject,
    pub serializer: SerializerEnum,
}

impl Value {
    #[inline]
    pub fn new(object: alias::PyObject, serializer: SerializerEnum) -> Self {
        Self { object, serializer }
    }

    #[inline]
    pub fn infer_bound(bound: alias::BoundObject) -> Self {
        let serializer = SerializerEnum::infer_bound(&bound);
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
