use std::str::FromStr;

use crate::dialect;
use crate::exc;
use crate::expr::enum_::Ident;
use crate::expr::extractors::IdentExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::internal::utils;
use crate::sql;

use super::column;
use super::constraint;

/// `ON COMMIT` action for a session-private temporary table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum OnCommit {
    /// Emits `PRESERVE ROWS`.
    #[strum(serialize = "PRESERVE ROWS")]
    PreserveRows,
    /// Emits `DELETE ROWS`.
    #[strum(serialize = "DELETE ROWS")]
    DeleteRows,
    /// Emits `DROP`.
    #[strum(serialize = "DROP")]
    Drop,
}

/// One table-level constraint stored on [`PyCreateTable`].
#[derive(Debug, Clone)]
pub enum TableConstraint {
    /// `UNIQUE`.
    Unique(pyo3::Py<constraint::PyUniqueConstraint>),
    /// `PRIMARY KEY`.
    PrimaryKey(pyo3::Py<constraint::PyPrimaryKeyConstraint>),
    /// `FOREIGN KEY`.
    ForeignKey(pyo3::Py<constraint::PyForeignKeyConstraint>),
    /// `CHECK`.
    Check(pyo3::Py<constraint::PyCheckConstraint>),
}

impl TableConstraint {
    /// Attaches inner `pyo3::Py<...>`s to the given Python context.
    #[inline(always)]
    pub fn bind<'a, 'py>(&'a self, py: pyo3::Python<'py>) -> &'a pyo3::Bound<'py, pyo3::PyAny> {
        match self {
            Self::Unique(constraint) => constraint.bind(py).as_any(),
            Self::PrimaryKey(constraint) => constraint.bind(py).as_any(),
            Self::ForeignKey(constraint) => constraint.bind(py).as_any(),
            Self::Check(constraint) => constraint.bind(py).as_any(),
        }
    }

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> exc::Result<repr::ReprFormatter> {
        match self {
            Self::Unique(x) => {
                let bound = x.bind(py);
                let base = bound.as_super().get();

                Ok(bound.get().python_repr(base.name.as_ref().map(arcstr::ArcStr::as_str)))
            }
            Self::PrimaryKey(x) => {
                let bound = x.bind(py);
                let base = bound.as_super().get();

                Ok(bound.get().python_repr(base.name.as_ref().map(arcstr::ArcStr::as_str)))
            }
            Self::ForeignKey(x) => {
                let bound = x.bind(py);
                let base = bound.as_super().get();

                Ok(bound.get().python_repr(base.name.as_ref().map(arcstr::ArcStr::as_str)))
            }
            Self::Check(x) => {
                let bound = x.bind(py);
                let base = bound.as_super().get();

                bound
                    .get()
                    .python_repr(py, base.name.as_ref().map(arcstr::ArcStr::as_str))
            }
        }
    }
}

impl sql::WriteSql for TableConstraint {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        let py: pyo3::Python<'static> = unsafe { std::mem::transmute(out.py()) };

        match self {
            Self::Unique(constraint) => {
                let bound = constraint.bind(py);

                if let Some(ref name) = bound.as_super().get().name {
                    out.write_str("CONSTRAINT ");
                    dialect.write_identifier(out, name.as_str());
                    out.write(' ');
                }

                bound.get().write_sql(dialect, out)
            }
            Self::PrimaryKey(constraint) => {
                let bound = constraint.bind(py);

                if let Some(ref name) = bound.as_super().get().name {
                    out.write_str("CONSTRAINT ");
                    dialect.write_identifier(out, name.as_str());
                    out.write(' ');
                }

                bound.get().write_sql(dialect, out)
            }
            Self::ForeignKey(constraint) => {
                let bound = constraint.bind(py);

                if let Some(ref name) = bound.as_super().get().name {
                    out.write_str("CONSTRAINT ");
                    dialect.write_identifier(out, name.as_str());
                    out.write(' ');
                }

                bound.get().write_sql(dialect, out)
            }
            Self::Check(constraint) => {
                let bound = constraint.bind(py);

                if let Some(ref name) = bound.as_super().get().name {
                    out.write_str("CONSTRAINT ");
                    dialect.write_identifier(out, name.as_str());
                    out.write(' ');
                }

                bound.get().write_sql(dialect, out)
            }
        }
    }
}

/// `CREATE TABLE` statement.
///
/// ```ignore
/// CREATE [TEMPORARY | VOLATILE | PRIVATE TEMPORARY] [EXTERNAL] TABLE [IF NOT EXISTS] <name> (...)
/// [ON COMMIT PRESERVE ROWS | DELETE ROWS | DROP]
/// ```
///
/// Not supported: `OR REPLACE`, `CREATE TABLE ... AS`, `LIKE`, `CLONE`, and `OF`.
#[rapidquery_macros::pyclass("rapidquery.ddl.CreateTable", extends = super::super::PyDataDefinitionLanguage)]
#[derive(Debug, Clone)]
pub struct PyCreateTable {
    /// Table name.
    pub name: Ident,

    /// Column definitions. At least one.
    pub columns: Vec<pyo3::Py<column::PyColumn>>,

    /// Table-level constraints, written after the columns.
    pub constraints: Vec<TableConstraint>,

    /// Emits `IF NOT EXISTS`. Omitted when the dialect does not support it.
    pub if_not_exists: bool,

    /// Session-private temporary table. The keyword comes from the dialect.
    pub temporary: bool,

    /// `ON COMMIT` action. Requires [`Self::temporary`].
    pub on_commit: Option<OnCommit>,

    /// Emits `EXTERNAL` before `TABLE`.
    pub external: bool,

    /// Table comment. Omitted when the dialect has no comment template.
    pub comment: Option<arcstr::ArcStr>,

    /// External location. Rejected when the dialect has no location template.
    pub location: Option<arcstr::ArcStr>,

    /// Extra raw SQL appended after the other clauses.
    pub extra: Option<arcstr::ArcStr>,
}

impl sql::WriteSql for PyCreateTable {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        out.write_str("CREATE ");

        if self.temporary {
            let Some(tmpl) = &dialect.table.temporary else {
                return Err(exc::Error::dialect("temporary table"));
            };

            out.write_str(tmpl.as_str());
            out.write(' ');
        }

        if self.external {
            if !dialect.table.features.contains(dialect::TableFeatures::EXTERNAL) {
                return Err(exc::Error::dialect("external table"));
            }

            out.write_str("EXTERNAL ");
        }

        out.write_str("TABLE ");
        out.write_str_if(
            self.if_not_exists && dialect.table.features.contains(dialect::TableFeatures::IF_NOT_EXISTS),
            "IF NOT EXISTS ",
        );
        self.name.write_sql(dialect, out)?;

        let py: pyo3::Python<'static> = unsafe { std::mem::transmute(out.py()) };

        out.write_str(" (");
        for (index, column) in self.columns.iter().enumerate() {
            if index > 0 {
                out.write_str(", ");
            }
            column.bind(py).get().write_sql(dialect, out)?;
        }
        for constraint in &self.constraints {
            out.write_str(", ");
            constraint.write_sql(dialect, out)?;
        }
        out.write(')');

        if let Some(action) = self.on_commit {
            if !dialect.table.features.contains(dialect::TableFeatures::ON_COMMIT) {
                return Err(exc::Error::dialect("CREATE TABLE ... ON COMMIT"));
            }

            out.write_str(" ON COMMIT ");
            out.write_str(action.into());
        }

        if let Some(comment) = &self.comment
            && let Some(tmpl) = &dialect.table.comment
        {
            out.write(' ');
            tmpl.write_to(out, |_, out| dialect.string.write_string_literal(out, comment.as_str()));
        }

        if let Some(location) = &self.location {
            let Some(tmpl) = &dialect.table.location else {
                return Err(exc::Error::dialect("CREATE TABLE ... LOCATION"));
            };
            out.write(' ');
            tmpl.write_to(out, |_, out| {
                dialect.string.write_string_literal(out, location.as_str())
            });
        }

        if let Some(extra) = &self.extra {
            out.write(' ');
            out.write_str(extra.as_str());
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyCreateTable {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        name,
        *args,
        if_not_exists = false,
        temporary = false,
        on_commit = None,
        external = false,
        comment = None,
        location = None,
        extra = None
    ))]
    fn new(
        name: IdentExtractor,
        args: alias::ArgsType,
        if_not_exists: bool,
        temporary: bool,
        on_commit: Option<&str>,
        external: bool,
        comment: Option<&str>,
        location: Option<&str>,
        extra: Option<&str>,
    ) -> exc::Result<pyo3::PyClassInitializer<Self>> {
        use pyo3::types::PyTupleMethods;

        let mut columns = Vec::with_capacity(1);
        let mut constraints = Vec::new();

        for arg in args.iter() {
            unsafe {
                let type_ptr = pyo3::ffi::Py_TYPE(arg.as_ptr());

                // PyColumn
                if type_ptr == crate::typeref::DDL_COLUMN_TYPE {
                    let obj = arg.cast_unchecked::<column::PyColumn>().clone().unbind();
                    columns.push(obj);
                }
                // PyUniqueConstraint
                else if type_ptr == crate::typeref::DDL_UNIQUE_CONSTRAINT_TYPE {
                    let obj = arg.cast_unchecked::<constraint::PyUniqueConstraint>().clone().unbind();
                    constraints.push(TableConstraint::Unique(obj));
                }
                // PyPrimaryKeyConstraint
                else if type_ptr == crate::typeref::DDL_PRIMARY_KEY_CONSTRAINT_TYPE {
                    let obj = arg
                        .cast_unchecked::<constraint::PyPrimaryKeyConstraint>()
                        .clone()
                        .unbind();
                    constraints.push(TableConstraint::PrimaryKey(obj));
                }
                // PyForeignKeyConstraint
                else if type_ptr == crate::typeref::DDL_FOREIGN_KEY_CONSTRAINT_TYPE {
                    let obj = arg
                        .cast_unchecked::<constraint::PyForeignKeyConstraint>()
                        .clone()
                        .unbind();
                    constraints.push(TableConstraint::ForeignKey(obj));
                }
                // PyCheckConstraint
                else if type_ptr == crate::typeref::DDL_CHECK_CONSTRAINT_TYPE {
                    let obj = arg.cast_unchecked::<constraint::PyCheckConstraint>().clone().unbind();
                    constraints.push(TableConstraint::Check(obj));
                }
                // No match
                else {
                    return Err(exc::Error::type_mismatch("args", "Column or Constraint", &arg));
                }
            }
        }

        if columns.is_empty() {
            return Err(exc::Error::invalid("columns", "at least one Column is required"));
        }

        let on_commit = match on_commit {
            None => None,
            Some(action) => Some(OnCommit::from_str(action).map_err(|_| {
                exc::Error::invalid(
                    "on_commit",
                    format!("expected 'PRESERVE ROWS', 'DELETE ROWS', or 'DROP', received {action}"),
                )
            })?),
        };

        if on_commit.is_some() && !temporary {
            return Err(exc::Error::invalid("on_commit", "ON COMMIT requires a temporary table"));
        }

        let result = Self {
            name: name.0,
            columns,
            constraints,
            if_not_exists,
            temporary,
            on_commit,
            external,
            comment: comment.map(arcstr::ArcStr::from),
            location: location.map(arcstr::ArcStr::from),
            extra: extra.map(arcstr::ArcStr::from),
        };
        Ok(pyo3::PyClassInitializer::from(super::super::PyDataDefinitionLanguage).add_subclass(result))
    }

    #[getter]
    fn name<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.name.clone().into())
    }

    #[getter]
    fn columns<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        unsafe {
            utils::iterator_to_tuple(
                py,
                self.columns.len() as isize,
                self.columns.iter().map(|column| Ok(column.clone_ref(py).into_ptr())),
            )
        }
    }

    #[getter]
    fn constraints<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        unsafe {
            utils::iterator_to_tuple(
                py,
                self.constraints.len() as isize,
                self.constraints
                    .iter()
                    .map(|constraint| Ok(constraint.bind(py).clone().into_ptr())),
            )
        }
    }

    #[getter]
    fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    #[getter]
    fn temporary(&self) -> bool {
        self.temporary
    }

    #[getter]
    fn on_commit(&self) -> Option<&'static str> {
        self.on_commit.map(Into::into)
    }

    #[getter]
    fn external(&self) -> bool {
        self.external
    }

    #[getter]
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }

    #[getter]
    fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    #[getter]
    fn extra(&self) -> Option<&str> {
        self.extra.as_deref()
    }

    /// Render this statement as raw SQL (values inlined as literals).
    fn to_sql(&self, dialect: pyo3::Bound<'_, dialect::PyDialect>) -> exc::Result<String> {
        sql::write_sql(dialect.py(), self, dialect.get())
    }

    // TODO(awolverp): Support `__ident__` and `__expr__` protocols

    fn __repr__(&self, py: pyo3::Python<'_>) -> exc::Result<String> {
        let mut fmt = repr::ReprFormatter::new_call("CreateTable");
        fmt.positional(self.name.python_repr());

        {
            let mut args = fmt.positional_list();
            for column in &self.columns {
                args.positional(column.bind(py).get().python_repr(py)?);
            }

            for constraint in &self.constraints {
                args.positional(constraint.python_repr(py)?);
            }
        }

        fmt.keyword_true("if_not_exists", self.if_not_exists)
            .keyword_true("temporary", self.temporary)
            .keyword("on_commit", self.on_commit.map(Into::<&'static str>::into))
            .keyword_true("external", self.external)
            .keyword("comment", self.comment.clone())
            .keyword("location", self.location.clone())
            .keyword("extra", self.extra.clone());

        Ok(fmt.into_string())
    }
}
