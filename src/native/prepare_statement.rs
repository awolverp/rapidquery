use std::fmt;

use hashbrown::HashMap;

use crate::{
    internal::alias,
    native::dialect::{PlaceholderSyntax, SqlDialect},
};

/// Error produced while building a prepared statement.
#[derive(Debug)]
pub enum PrepareError {
    /// A [`fmt::Write`] implementation returned [`fmt::Error`].
    Write,
    /// An error that originated in Python (e.g. from `.extract()`).
    Python(pyo3::PyErr),
}

impl fmt::Display for PrepareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write => f.write_str("failed to write SQL"),
            Self::Python(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for PrepareError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write => None,
            Self::Python(err) => Some(err),
        }
    }
}

impl From<fmt::Error> for PrepareError {
    fn from(_: fmt::Error) -> Self {
        Self::Write
    }
}

impl From<pyo3::PyErr> for PrepareError {
    fn from(err: pyo3::PyErr) -> Self {
        Self::Python(err)
    }
}

pub type PrepareResult<T> = Result<T, PrepareError>;
pub type PrepareParams = HashMap<String, alias::PyObject>;

/// Sigil written before a named placeholder (`@param_0`, `:param_0`, `$param_0`).
#[inline]
fn placeholder_sigil(syntax: PlaceholderSyntax) -> char {
    match syntax {
        PlaceholderSyntax::AtSign => '@',
        PlaceholderSyntax::Colon => ':',
        PlaceholderSyntax::DollarNamed => '$',
    }
}

/// Accumulates rendered SQL and the bound-parameter map.
///
/// Implements [`fmt::Write`], so `write!` / `writeln!` work directly:
/// `write!(ctx, "{name} = ")?;`
pub struct PrepareContext<'py> {
    py: pyo3::Python<'py>,
    sql: String,
    params: PrepareParams,
    next_id: usize,
}

impl<'py> PrepareContext<'py> {
    #[inline]
    pub fn new(py: pyo3::Python<'py>) -> Self {
        Self {
            py,
            sql: String::with_capacity(256),
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
        &self.sql
    }

    #[inline]
    pub fn params(&self) -> &PrepareParams {
        &self.params
    }

    #[inline]
    pub fn into_parts(self) -> (String, PrepareParams) {
        (self.sql, self.params)
    }

    #[inline]
    fn alloc_param_name(&mut self) -> String {
        let name = format!("param_{}", self.next_id);
        self.next_id += 1;
        name
    }
}

impl PrepareContext<'_> {
    /// Register `param` and return its placeholder (`@param_N` / `:param_N` / `$param_N`).
    ///
    /// Prefer [`push_param`](Self::push_param) when the placeholder is only going
    /// to be appended to the SQL — that path does not allocate the placeholder
    /// `String`.
    #[inline]
    #[must_use = "write the placeholder into the SQL, or the parameter is never referenced"]
    pub fn add_param(&mut self, dialect: &SqlDialect, param: pyo3::Py<pyo3::PyAny>) -> String {
        let name = self.alloc_param_name();
        let mut placeholder = String::with_capacity(1 + name.len());
        placeholder.push(placeholder_sigil(dialect.placeholder_syntax()));
        placeholder.push_str(&name);
        self.params.insert(name, param);
        placeholder
    }

    /// Register `param` and append its placeholder to the SQL in one step.
    ///
    /// Returns `&mut Self` so calls can be chained:
    ///
    /// ```ignore
    /// context
    ///     .push_str("SELECT * FROM t WHERE id = ")
    ///     .push_param(dialect, id)
    ///     .push_str(" AND name = ")
    ///     .push_param(dialect, name);
    /// ```
    #[inline]
    pub fn push_param(&mut self, dialect: &SqlDialect, param: pyo3::Py<pyo3::PyAny>) -> &mut Self {
        let name = self.alloc_param_name();
        self.sql
            .push(placeholder_sigil(dialect.placeholder_syntax()));
        self.sql.push_str(&name);
        self.params.insert(name, param);
        self
    }

    #[inline]
    pub fn push_char(&mut self, c: char) -> &mut Self {
        self.sql.push(c);
        self
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) -> &mut Self {
        self.sql.push_str(s);
        self
    }
}

impl fmt::Write for PrepareContext<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.sql.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.sql.push(c);
        Ok(())
    }
}

pub trait PrepareStatement {
    fn prepare(&self, dialect: &SqlDialect, context: &mut PrepareContext<'_>) -> PrepareResult<()>;
}

/// Rust-only entry point. Safe to call from non-pyo3 code as long as a
/// `Python<'_>` token and dialect are available.
pub fn prepare<T: PrepareStatement>(
    py: pyo3::Python<'_>,
    stmt: &T,
    dialect: &SqlDialect,
) -> PrepareResult<(String, PrepareParams)> {
    let mut context = PrepareContext::new(py);
    stmt.prepare(dialect, &mut context)?;
    Ok(context.into_parts())
}

/// Prepare each item in order, writing `separator` between them
/// (not before the first or after the last).
///
/// Use `","` / `", "` for lists and `" AND "` / `" OR "` for predicates.
pub fn prepare_separated<'a, T: PrepareStatement + 'a>(
    dialect: &SqlDialect,
    context: &mut PrepareContext<'_>,
    items: impl IntoIterator<Item = &'a T>,
    separator: &str,
) -> PrepareResult<()> {
    let mut first = true;
    for item in items {
        if !first {
            context.push_str(separator);
        }
        first = false;
        item.prepare(dialect, context)?;
    }
    Ok(())
}
