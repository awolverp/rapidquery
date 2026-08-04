use std::fmt;

use hashbrown::HashMap;

use crate::{
    internal::alias,
    native::dialect::{PlaceholderSyntax, SqlDialect},
};

/// Error produced while building a prepared statement.
#[derive(Debug)]
pub enum PrepareStatementError {
    FmtError,
    /// An error that originated in Python (e.g. from `.extract()` ).
    Python(pyo3::PyErr),
}

impl std::fmt::Display for PrepareStatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FmtError => write!(f, "fmt error occurred"),
            Self::Python(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for PrepareStatementError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FmtError => None,
            Self::Python(err) => Some(err),
        }
    }
}

impl From<std::fmt::Error> for PrepareStatementError {
    fn from(_: std::fmt::Error) -> Self {
        Self::FmtError
    }
}

impl From<pyo3::PyErr> for PrepareStatementError {
    fn from(err: pyo3::PyErr) -> Self {
        Self::Python(err)
    }
}

pub type PrepareResult<T> = Result<T, PrepareStatementError>;
pub type PrepareStatementParams = HashMap<String, alias::PyObject>;

/// Mutable state threaded through [`PrepareStatement::prepare`] calls.
///
/// Accumulates the rendered SQL string and the parameter map as
/// statement fragments are built up. Because it implements
/// [`std::fmt::Write`] (see below), you can use the standard `write!`
/// / `writeln!` macros instead of the lower-level `write_str` /
/// `write_char` methods when that's more convenient.
// Assisted-by: Claude
pub struct PrepareStatementContext<'py> {
    py: pyo3::Python<'py>,
    string: String,
    params: PrepareStatementParams,
}

impl<'py> PrepareStatementContext<'py> {
    /// Creates a new [`PrepareStatementContext`].
    #[inline]
    pub fn new(py: pyo3::Python<'py>) -> Self {
        Self {
            py,
            string: String::with_capacity(256),
            params: HashMap::new(),
        }
    }

    /// Returns the Python interpreter.
    #[inline]
    pub fn py(&self) -> pyo3::Python<'py> {
        self.py
    }

    /// Consumes `self` and returns SQL and parameters.
    #[inline]
    pub fn into_parts(self) -> (String, PrepareStatementParams) {
        (self.string, self.params)
    }
}

impl PrepareStatementContext<'_> {
    /// Inserts a parameter into the context and returns a placeholder
    /// you can add to the SQL string.
    ///
    /// If you don't need the placeholder for anything other than
    /// writing it straight into the SQL (the common case), prefer
    /// [`write_parameter`](Self::write_parameter), which does both
    /// steps in one call and avoids the temporary `String` allocation
    /// this method's return value needs at the call site.
    // Assisted-by: Claude
    #[inline]
    #[must_use = "the placeholder must be written into the SQL string, e.g. with `write_str`, or the parameter won't be referenced"]
    pub fn add_parameter(&mut self, dialect: &SqlDialect, param: pyo3::Py<pyo3::PyAny>) -> String {
        let name = format!("param_{}", self.params.len());
        let placeholder = match dialect.placeholder_syntax() {
            PlaceholderSyntax::AtSign => format!("@{name}"),
            PlaceholderSyntax::Colon => format!(":{name}"),
            PlaceholderSyntax::DollarNamed => format!("${name}"),
        };
        self.params.insert(name, param);
        placeholder
    }

    /// Inserts a parameter into the context and immediately writes its
    /// placeholder into the SQL string.
    ///
    /// This is the ergonomic, one-step alternative to calling
    /// [`add_parameter`](Self::add_parameter) followed by `write_str`,
    /// which is by far the most common pattern at call sites.
    ///
    /// Returns `&mut Self` so calls can be chained, e.g.:
    ///
    /// ```ignore
    /// context
    ///     .write_str("SELECT * FROM t WHERE id = ")
    ///     .write_parameter(id)
    ///     .write_str(" AND name = ")
    ///     .write_parameter(name);
    /// ```
    // Assisted-by: Claude
    #[inline]
    pub fn write_parameter(
        &mut self,
        dialect: &SqlDialect,
        param: pyo3::Py<pyo3::PyAny>,
    ) -> &mut Self {
        let placeholder = self.add_parameter(dialect, param);
        self.string.push_str(&placeholder);
        self
    }

    /// Writes a character into SQL. Returns `&mut Self` for chaining
    /// (see [`write_parameter`](Self::write_parameter) for an example).
    // Assisted-by: Claude
    #[inline]
    pub fn write_char(&mut self, c: char) -> &mut Self {
        self.string.push(c);
        self
    }

    /// Writes a string into SQL. Returns `&mut Self` for chaining
    /// (see [`write_parameter`](Self::write_parameter) for an example).
    // Assisted-by: Claude
    #[inline]
    pub fn write_str(&mut self, s: &str) -> &mut Self {
        self.string.push_str(s);
        self
    }
}

/// Lets [`PrepareStatementContext`] be used with the standard `write!`
/// / `writeln!` macros, e.g. `write!(context, "{name} = ")?;`, which
/// reads more naturally than chained `write_str` calls when building
/// up a fragment with several interpolated pieces.
// Assisted-by: Claude
impl fmt::Write for PrepareStatementContext<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.string.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.string.push(c);
        Ok(())
    }
}

pub trait PrepareStatement {
    fn prepare(
        &self,
        dialect: &SqlDialect,
        context: &mut PrepareStatementContext,
    ) -> PrepareResult<()>;
}

/// Rust-only entry point. Returns your own error type — no `PyErr`
/// involved on the happy *or* error path, so this can be called from
/// non-pyo3 code (tests, internal tooling, etc.) as long as a `Python<'_>`
/// token and dialect are available.
pub fn prepare<T: PrepareStatement>(
    py: pyo3::Python,
    stmt: &T,
    dialect: &SqlDialect,
) -> PrepareResult<(String, PrepareStatementParams)> {
    let mut context = PrepareStatementContext::new(py);
    stmt.prepare(dialect, &mut context)?;
    Ok(context.into_parts())
}

/// Prepares each of `args` in turn, writing `separator` between
/// consecutive items (but not before the first or after the last).
///
/// Handy for comma-separated column lists, `AND`/`OR`-joined
/// predicates, etc.
// Assisted-by: Claude
pub fn prepare_separated<T: PrepareStatement>(
    dialect: &SqlDialect,
    context: &mut PrepareStatementContext,
    args: &[T],
    separator: char,
) -> PrepareResult<()> {
    if let [single] = args {
        return single.prepare(dialect, context);
    }
    for (ix, ident) in args.iter().enumerate() {
        ident.prepare(dialect, context)?;
        if ix + 1 < args.len() {
            context.write_char(separator);
        }
    }
    Ok(())
}
