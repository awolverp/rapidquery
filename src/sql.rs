//! SQL writers: raw (`SqlBuf`) and parameterized (`PrepareContext`).
//!
//! One AST trait, [`WriteSql`], walks a node into any [`SqlWriter`]. Identifiers
//! and type names only push text. Values (when they land) call
//! [`SqlWriter::push_value`]: raw inlines a literal, prepared emits a
//! placeholder.

use hashbrown::HashMap;
use pyo3::types::PyAnyMethods;

use crate::data_type::SerializerEnum;
use crate::dialect::PySqlDialect;
use crate::internal::alias;

/// Error produced while building SQL.
#[derive(Debug)]
pub enum PrepareError {
    /// A [`std::fmt::Write`] implementation returned [`std::fmt::Error`].
    Write,
    /// An error that originated in Python (e.g. from `.extract()`).
    Python(pyo3::PyErr),
    /// A `Custom` template had a different number of `{}` slots than args.
    CustomPlaceholders { expected: usize, got: usize },
}

impl std::fmt::Display for PrepareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Write => f.write_str("failed to write SQL"),
            Self::Python(err) => write!(f, "{err}"),
            Self::CustomPlaceholders { expected, got } => write!(
                f,
                "custom expression has {expected} placeholders but {got} arguments"
            ),
        }
    }
}

impl std::error::Error for PrepareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write | Self::CustomPlaceholders { .. } => None,
            Self::Python(err) => Some(err),
        }
    }
}

impl From<std::fmt::Error> for PrepareError {
    fn from(_: std::fmt::Error) -> Self {
        Self::Write
    }
}

impl From<pyo3::PyErr> for PrepareError {
    fn from(err: pyo3::PyErr) -> Self {
        Self::Python(err)
    }
}

impl From<PrepareError> for pyo3::PyErr {
    fn from(err: PrepareError) -> Self {
        match err {
            PrepareError::Python(err) => err,
            PrepareError::Write => {
                rapidquery_macros::pyerr!(PyRuntimeError, "failed to write SQL")
            }
            PrepareError::CustomPlaceholders { expected, got } => {
                rapidquery_macros::pyerr!(
                    PyValueError,
                    "custom expression has {} placeholders but {} arguments",
                    expected,
                    got
                )
            }
        }
    }
}

pub type PrepareResult<T> = Result<T, PrepareError>;
pub type PrepareParams = HashMap<String, alias::PyObject>;

/// Output target for [`WriteSql`].
///
/// [`SqlBuf`] inlines values as SQL literals. [`PrepareContext`] binds them as
/// named placeholders. Everything else is `push_str` / `push_char`.
pub trait SqlWriter: std::fmt::Write {
    fn push_str(&mut self, s: &str) -> &mut Self;
    fn push_char(&mut self, c: char) -> &mut Self;

    /// Raw (`SqlBuf`): `None` → `NULL`, else `serializer.write_sql` into the
    /// SQL buffer. Prepared: bind `object` as a placeholder; `serializer` is
    /// unused.
    fn push_value(
        &mut self,
        dialect: &PySqlDialect,
        serializer: &SerializerEnum,
        object: &alias::PyObject,
    ) -> PrepareResult<()>;
}

/// Raw SQL buffer. No parameter map.
pub struct BufferedSql<'py> {
    py: pyo3::Python<'py>,
    sql: String,
}

impl<'py> BufferedSql<'py> {
    #[inline]
    pub fn new(py: pyo3::Python<'py>) -> Self {
        Self {
            py,
            sql: String::new(),
        }
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
    pub fn as_str(&self) -> &str {
        &self.sql
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.sql
    }
}

impl<'py> SqlWriter for BufferedSql<'py> {
    #[inline]
    fn push_str(&mut self, s: &str) -> &mut Self {
        self.sql.push_str(s);
        self
    }

    #[inline]
    fn push_char(&mut self, c: char) -> &mut Self {
        self.sql.push(c);
        self
    }

    fn push_value(
        &mut self,
        dialect: &PySqlDialect,
        serializer: &SerializerEnum,
        object: &alias::PyObject,
    ) -> PrepareResult<()> {
        let bound = object.bind(self.py);
        if bound.is_none() {
            self.sql.push_str("NULL");
            return Ok(());
        }
        serializer.write_sql(dialect, bound, &mut self.sql)?;
        Ok(())
    }
}

impl<'py> std::fmt::Write for BufferedSql<'py> {
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

/// Accumulates rendered SQL and the bound-parameter map.
///
/// Implements [`std::fmt::Write`], so `write!` / `writeln!` work directly:
/// `write!(ctx, "{name} = ")?;`
pub struct ParameterizedSql<'py> {
    py: pyo3::Python<'py>,
    sql: String,
    params: PrepareParams,
    next_id: usize,
}

impl<'py> ParameterizedSql<'py> {
    #[inline]
    pub fn new(py: pyo3::Python<'py>) -> Self {
        Self {
            py,
            sql: String::new(),
            params: HashMap::new(),
            next_id: 0,
        }
    }

    #[inline]
    pub fn with_capacity(py: pyo3::Python<'py>, capacity: usize) -> Self {
        Self {
            py,
            sql: String::with_capacity(capacity),
            params: HashMap::new(),
            next_id: 0,
        }
    }

    #[inline]
    pub fn py(&self) -> pyo3::Python<'py> {
        self.py
    }

    #[inline]
    pub fn sql(&self) -> &str {
        self.sql.as_str()
    }

    #[inline]
    pub fn params(&self) -> &PrepareParams {
        &self.params
    }

    #[inline]
    pub fn into_parts(self) -> (String, PrepareParams) {
        (self.sql, self.params)
    }
}

impl ParameterizedSql<'_> {
    /// Register `param` and append its placeholder to the SQL in one step.
    #[inline]
    pub fn push_param(&mut self, dialect: &PySqlDialect, param: alias::PyObject) -> &mut Self {
        // allocate param name
        let name = format!("param_{}", self.next_id);
        self.next_id += 1;

        self.sql.push(dialect.0.placeholder_sigil);
        self.sql.push_str(&name);
        self.params.insert(name, param);
        self
    }
}

impl SqlWriter for ParameterizedSql<'_> {
    #[inline]
    fn push_str(&mut self, s: &str) -> &mut Self {
        self.sql.push_str(s);
        self
    }

    #[inline]
    fn push_char(&mut self, c: char) -> &mut Self {
        self.sql.push(c);
        self
    }

    #[inline]
    fn push_value(
        &mut self,
        dialect: &PySqlDialect,
        _serializer: &SerializerEnum,
        object: &alias::PyObject,
    ) -> PrepareResult<()> {
        // TODO: use positional parameters instead of named parameters
        //       some of dialects and their native clients does not support
        //       named parameters.
        self.push_param(dialect, object.clone_ref(self.py));
        Ok(())
    }
}

impl std::fmt::Write for ParameterizedSql<'_> {
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
    fn write_sql<W: SqlWriter>(
        &self,
        // NOTE FOR ME:
        // Why `PySqlDialect` instead of Rust `Dialect`?
        // because we accept custom serializers and custom serializer functions
        // need to get dialect objects.
        dialect: &PySqlDialect,
        out: &mut W,
    ) -> PrepareResult<()>;
}

/// Render `stmt` as raw SQL (values inlined as literals).
pub fn write_sql<'py, T: WriteSql>(
    py: pyo3::Python<'py>,
    stmt: &T,
    dialect: &PySqlDialect,
) -> PrepareResult<String> {
    let mut buf = BufferedSql::new(py);
    stmt.write_sql(dialect, &mut buf)?;
    Ok(buf.into_string())
}

/// Render `stmt` as a parameterized statement.
pub fn prepare<'py, T: WriteSql>(
    py: pyo3::Python<'py>,
    stmt: &T,
    dialect: &PySqlDialect,
) -> PrepareResult<(String, PrepareParams)> {
    let mut context = ParameterizedSql::new(py);
    stmt.write_sql(dialect, &mut context)?;
    Ok(context.into_parts())
}

/// Write each item in order, writing `separator` between them
/// (not before the first or after the last).
///
/// Use `","` / `", "` for lists and `" AND "` / `" OR "` for predicates.
#[cfg_attr(feature = "nightly", optimize(speed))]
pub fn write_separated<'a, T, W>(
    dialect: &PySqlDialect,
    out: &mut W,
    items: impl IntoIterator<Item = &'a T>,
    separator: &str,
) -> PrepareResult<()>
where
    T: WriteSql + 'a,
    W: SqlWriter,
{
    let mut first = true;
    for item in items {
        if !first {
            out.push_str(separator);
        }
        first = false;
        item.write_sql(dialect, out)?;
    }
    Ok(())
}
