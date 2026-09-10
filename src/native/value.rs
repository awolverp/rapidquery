use crate::internal::alias;
use crate::native::data_type::Serializer;
use crate::native::sql::WriteSql;

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
    pub serializer: Serializer,
}

impl Value {
    #[inline]
    pub fn new(object: alias::PyObject, serializer: Serializer) -> Self {
        Self { object, serializer }
    }

    #[inline]
    pub fn infer_bound(bound: alias::BoundObject) -> Self {
        let serializer = Serializer::infer_bound(&bound);
        Self::new(bound.unbind(), serializer)
    }
}

impl WriteSql for Value {
    fn write_sql<W: super::sql::SqlWriter>(
        &self,
        dialect: &super::dialect::SqlDialect,
        out: &mut W,
    ) -> super::sql::PrepareResult<()> {
        out.push_value(dialect, &self.serializer, &self.object)
    }
}
