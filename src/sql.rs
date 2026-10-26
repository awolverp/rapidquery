//! SQL text buffers.
//!
//! [`WriteSql`] walks a node into a [`SqlWriter`]. [`Buffer`] inlines values.
//! [`Parameterized`] binds them as placeholders.

use crate::dialect;
use crate::internal::alias;
use crate::internal::writer::FastWrite;
use crate::serializers::SerializerType;

pub type PrepareParams = Vec<alias::PyObject>;

/// Destination for [`WriteSql`].
///
/// [`Buffered`] inlines values. [`Parameterized`] binds them. Other methods
/// only append text.
pub trait SqlWriter: FastWrite {
    /// Raw (`SqlBuf`): `None` → `NULL`, else `serializer.serialize` into the
    /// SQL buffer. Prepared: bind `object` as a placeholder; `serializer` is
    /// unused.
    fn write_value(
        &mut self,
        dialect: &dialect::PyDialect,
        serializer: &SerializerType,
        object: &alias::PyObject,
    ) -> pyo3::PyResult<()>;
}

/// SQL buffer that inlines values. It does not keep a parameter list.
///
/// Build one with a Python token, append text through [`SqlWriter`], then take
/// the string with [`Self::into_string`].
pub struct Buffer<'py> {
    py: pyo3::Python<'py>,
    sql: String,
}

impl<'py> Buffer<'py> {
    #[inline]
    pub fn new(py: pyo3::Python<'py>) -> Self {
        Self { py, sql: String::new() }
    }

    #[inline]
    pub fn with_capacity(py: pyo3::Python<'py>, capacity: usize) -> Self {
        Self {
            py,
            sql: String::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn py(&self) -> pyo3::Python<'py> {
        self.py
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.sql
    }
}

impl<'py> FastWrite for Buffer<'py> {
    #[inline]
    fn write(&mut self, val: char) {
        self.sql.write(val);
    }

    #[inline]
    fn write_str(&mut self, val: &str) {
        self.sql.write_str(val);
    }
}

impl<'py> SqlWriter for Buffer<'py> {
    #[inline]
    fn write_value(
        &mut self,
        dialect: &dialect::PyDialect,
        serializer: &SerializerType,
        object: &alias::PyObject,
    ) -> pyo3::PyResult<()> {
        use pyo3::types::PyAnyMethods;

        let bound = object.bind(self.py);
        if bound.is_none() {
            self.sql.push_str("NULL");
            return Ok(());
        }
        serializer.serialize(dialect, bound, &mut self.sql)?;
        Ok(())
    }
}

/// SQL text plus the Python objects bound for each placeholder.
///
/// [`std::fmt::Write`] appends text. [`Self::push_param`] appends a placeholder
/// and stores the object. Numbering starts at 1.
pub struct Parameterized<'py> {
    py: pyo3::Python<'py>,
    sql: String,
    params: PrepareParams,
    next_id: usize,
}

impl<'py> Parameterized<'py> {
    #[inline]
    pub fn new(py: pyo3::Python<'py>) -> Self {
        Self {
            py,
            sql: String::new(),
            params: PrepareParams::new(),
            next_id: 1,
        }
    }

    #[inline]
    pub fn with_capacity(py: pyo3::Python<'py>, capacity: usize) -> Self {
        Self {
            py,
            sql: String::with_capacity(capacity),
            params: PrepareParams::new(),
            next_id: 1,
        }
    }

    #[inline]
    pub fn py(&self) -> pyo3::Python<'py> {
        self.py
    }

    #[inline]
    pub fn into_parts(self) -> (String, PrepareParams) {
        (self.sql, self.params)
    }
}

impl Parameterized<'_> {
    /// Register `param` and append its placeholder to the SQL in one step.
    #[inline]
    pub fn push_param(&mut self, dialect: &dialect::PyDialect, param: alias::PyObject) -> &mut Self {
        dialect
            .placeholder
            .write_to(&mut self.sql, |_, out| out.write_int(self.next_id));

        self.next_id += 1;
        self.params.push(param);
        self
    }
}

impl<'py> FastWrite for Parameterized<'py> {
    #[inline]
    fn write(&mut self, val: char) {
        self.sql.write(val);
    }

    #[inline]
    fn write_str(&mut self, val: &str) {
        self.sql.write_str(val);
    }
}

impl SqlWriter for Parameterized<'_> {
    #[inline]
    fn write_value(
        &mut self,
        dialect: &dialect::PyDialect,
        _serializer: &SerializerType,
        object: &alias::PyObject,
    ) -> pyo3::PyResult<()> {
        self.push_param(dialect, object.clone_ref(self.py));
        Ok(())
    }
}

impl std::fmt::Write for Parameterized<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.sql.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.sql.push(c);
        Ok(())
    }
}

/// Render `self` into `out`. The writer chooses literal vs placeholder for values.
pub trait WriteSql {
    fn write_sql<W: SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> pyo3::PyResult<()>;
}

/// Render `stmt` as raw SQL (values inlined as literals).
pub fn write_sql<'py, T: WriteSql>(
    py: pyo3::Python<'py>,
    stmt: &T,
    dialect: &dialect::PyDialect,
) -> pyo3::PyResult<String> {
    let mut buf = Buffer::with_capacity(py, 256);
    stmt.write_sql(dialect, &mut buf)?;
    Ok(buf.into_string())
}

/// Render `stmt` as a parameterized statement.
pub fn prepare<'py, T: WriteSql>(
    py: pyo3::Python<'py>,
    stmt: &T,
    dialect: &dialect::PyDialect,
) -> pyo3::PyResult<(String, PrepareParams)> {
    let mut context = Parameterized::with_capacity(py, 256);
    stmt.write_sql(dialect, &mut context)?;
    Ok(context.into_parts())
}

/// Write each item in order, writing `separator` between them
/// (not before the first or after the last).
///
/// Use `","` / `", "` for lists and `" AND "` / `" OR "` for predicates.
#[cfg_attr(feature = "nightly", optimize(speed))]
pub fn write_separated<'a, T, W>(
    dialect: &dialect::PyDialect,
    out: &mut W,
    items: impl IntoIterator<Item = &'a T>,
    separator: &str,
) -> pyo3::PyResult<()>
where
    T: WriteSql + 'a,
    W: SqlWriter,
{
    let mut first = true;
    for item in items {
        if !first {
            out.write_str(separator);
        }
        first = false;
        item.write_sql(dialect, out)?;
    }
    Ok(())
}
