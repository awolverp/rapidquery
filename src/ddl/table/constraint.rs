use std::str::FromStr;

use crate::dialect;
use crate::exc;
use crate::expr::enum_::ExprInner;
use crate::expr::enum_::Ident;
use crate::expr::extractors::IdentExtractor;
use crate::expr::extractors::LiteralExprExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::internal::utils;
use crate::sql;

/// Iterates over `columns`, extracts [`arcstr::ArcStr`] from items, and collect them into
/// a [`Vec<arcstr::ArcStr>`].
fn collect_columns_argument(columns: alias::ArgsType) -> exc::Result<Vec<arcstr::ArcStr>> {
    use pyo3::types::PyAnyMethods;
    use pyo3::types::PyTupleMethods;

    if columns.is_empty() {
        return Err(exc::Error::invalid("columns", "at least one column is required"));
    }

    let mut converted: Vec<arcstr::ArcStr> = Vec::with_capacity(columns.len());
    for c in columns.iter() {
        converted.push(c.extract::<&str>()?.into());
    }

    Ok(converted)
}

/// Base class for table-level constraints.
///
/// ```ignore
/// [CONSTRAINT <name>] ...
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.Constraint", subclass)]
#[derive(Debug, Clone)]
pub struct PyConstraint {
    /// Optional constraint name.
    pub name: Option<arcstr::ArcStr>,
}

#[pyo3::pymethods]
impl PyConstraint {
    #[getter]
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// `UNIQUE` constraint.
///
/// ```ignore
/// [CONSTRAINT <name>] UNIQUE (<column>, ...)
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.UniqueConstraint", extends = PyConstraint)]
#[derive(Debug, Clone)]
pub struct PyUniqueConstraint {
    /// Constrained columns. at least one.
    pub columns: Vec<arcstr::ArcStr>,
}

impl PyUniqueConstraint {
    pub fn python_repr(&self, name: Option<&str>) -> repr::ReprFormatter {
        let mut fmt = repr::ReprFormatter::new_call("UniqueConstraint");
        for col in &self.columns {
            fmt.positional(col.as_str());
        }
        fmt.keyword("name", name);
        fmt
    }
}

impl sql::WriteSql for PyUniqueConstraint {
    #[inline]
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        if !dialect.table.features.contains(dialect::TableFeatures::UNIQUE) {
            return Err(exc::Error::dialect("Table-level UNIQUE constraint"));
        }

        out.write_str("UNIQUE (");
        sql::write_identifiers(dialect, out, self.columns.iter().map(|x| x.as_str()), ", ");
        out.write(')');

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyUniqueConstraint {
    #[new]
    #[pyo3(signature = (*columns, name = None))]
    fn new(columns: alias::ArgsType, name: Option<&str>) -> exc::Result<pyo3::PyClassInitializer<Self>> {
        let base = PyConstraint {
            name: name.map(Into::into),
        };
        let slf = Self {
            columns: collect_columns_argument(columns)?,
        };

        Ok(pyo3::PyClassInitializer::from(base).add_subclass(slf))
    }

    #[getter]
    fn columns<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        unsafe {
            utils::iterator_to_tuple(
                py,
                self.columns.len() as isize,
                self.columns.iter().map(|col| {
                    let pystr = pyo3::types::PyString::new(py, col.as_str());
                    Ok(pystr.into_ptr())
                }),
            )
        }
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let name = slf.as_super().name.clone();
        slf.python_repr(name.as_deref()).into_string()
    }
}

/// `PRIMARY KEY` constraint.
///
/// ```ignore
/// [CONSTRAINT <name>] PRIMARY KEY (<column>, ...)
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.PrimaryKeyConstraint", extends = PyConstraint)]
#[derive(Debug, Clone)]
pub struct PyPrimaryKeyConstraint {
    /// Constrained columns. at least one.
    pub columns: Vec<arcstr::ArcStr>,
}

impl PyPrimaryKeyConstraint {
    pub fn python_repr(&self, name: Option<&str>) -> repr::ReprFormatter {
        let mut fmt = repr::ReprFormatter::new_call("PrimaryKeyConstraint");
        for col in &self.columns {
            fmt.positional(col.as_str());
        }
        fmt.keyword("name", name);
        fmt
    }
}

impl sql::WriteSql for PyPrimaryKeyConstraint {
    #[inline]
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        if !dialect.table.features.contains(dialect::TableFeatures::PRIMARY_KEY) {
            return Err(exc::Error::dialect("Table-level PRIMARY KEY constraint"));
        }

        out.write_str("PRIMARY KEY (");
        sql::write_identifiers(dialect, out, self.columns.iter().map(|x| x.as_str()), ", ");
        out.write(')');

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyPrimaryKeyConstraint {
    #[new]
    #[pyo3(signature = (*columns, name = None))]
    fn new(columns: alias::ArgsType, name: Option<&str>) -> exc::Result<pyo3::PyClassInitializer<Self>> {
        let base = PyConstraint {
            name: name.map(Into::into),
        };
        let slf = Self {
            columns: collect_columns_argument(columns)?,
        };

        Ok(pyo3::PyClassInitializer::from(base).add_subclass(slf))
    }

    #[getter]
    fn columns<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        unsafe {
            utils::iterator_to_tuple(
                py,
                self.columns.len() as isize,
                self.columns.iter().map(|col| {
                    let pystr = pyo3::types::PyString::new(py, col.as_str());
                    Ok(pystr.into_ptr())
                }),
            )
        }
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let name = slf.as_super().name.clone();
        slf.python_repr(name.as_deref()).into_string()
    }
}

/// Action written after `ON DELETE` or `ON UPDATE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum ReferentialAction {
    /// Emits `RESTRICT`.
    #[strum(serialize = "RESTRICT")]
    Restrict,
    /// Emits `CASCADE`.
    #[strum(serialize = "CASCADE")]
    Cascade,
    /// Emits `SET NULL`.
    #[strum(serialize = "SET NULL")]
    SetNull,
    /// Emits `NO ACTION`.
    #[strum(serialize = "NO ACTION")]
    NoAction,
    /// Emits `SET DEFAULT`.
    #[strum(serialize = "SET DEFAULT")]
    SetDefault,
}

/// `FOREIGN KEY` constraint.
///
/// ```ignore
/// [CONSTRAINT <name>] FOREIGN KEY (<column>, ...) REFERENCES <table> [(<column>, ...)]
/// [ON DELETE <action>] [ON UPDATE <action>]
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.ForeignKeyConstraint", extends = PyConstraint)]
#[derive(Debug, Clone)]
pub struct PyForeignKeyConstraint {
    /// Constrained columns. At least one.
    pub columns: Vec<arcstr::ArcStr>,

    /// Referenced table.
    pub references: Ident,

    /// Referenced columns. At least one.
    pub ref_columns: Vec<arcstr::ArcStr>,

    /// `ON DELETE` action.
    pub on_delete: Option<ReferentialAction>,

    /// `ON UPDATE` action.
    pub on_update: Option<ReferentialAction>,
}

impl PyForeignKeyConstraint {
    pub fn python_repr(&self, name: Option<&str>) -> repr::ReprFormatter {
        let mut fmt = repr::ReprFormatter::new_call("ForeignKeyConstraint");

        {
            let mut lst = fmt.positional_list();
            for col in &self.columns {
                lst.positional(col.as_str());
            }
        }

        fmt.positional(self.references.python_repr());

        {
            let mut lst = fmt.positional_list();
            for col in &self.ref_columns {
                lst.positional(col.as_str());
            }
        }

        fmt.keyword("name", name);

        fmt.keyword("on_delete", self.on_delete.map(Into::<&'static str>::into))
            .keyword("on_update", self.on_update.map(Into::<&'static str>::into));

        fmt
    }
}

impl sql::WriteSql for PyForeignKeyConstraint {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        if !dialect.table.features.contains(dialect::TableFeatures::FOREIGN_KEY) {
            return Err(exc::Error::dialect("Table-level FOREIGN KEY constraint"));
        }

        out.write_str("FOREIGN KEY (");
        sql::write_identifiers(dialect, out, self.columns.iter().map(|x| x.as_str()), ", ");
        out.write_str(") REFERENCES ");

        self.references.write_sql(dialect, out)?;

        out.write_str(" (");
        sql::write_identifiers(dialect, out, self.ref_columns.iter().map(|x| x.as_str()), ", ");
        out.write(')');

        if let Some(action) = self.on_delete {
            if !dialect
                .table
                .features
                .contains(dialect::TableFeatures::REFERENTIAL_ACTION)
            {
                return Err(exc::Error::dialect("REFERENCES ... ON DELETE"));
            }

            out.write_str(" ON DELETE ");
            out.write_str(action.into());
        }
        if let Some(action) = self.on_update {
            if !dialect
                .table
                .features
                .contains(dialect::TableFeatures::REFERENTIAL_ACTION)
            {
                return Err(exc::Error::dialect("REFERENCES ... ON UPDATE"));
            }

            out.write_str(" ON UPDATE ");
            out.write_str(action.into());
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyForeignKeyConstraint {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (columns, references, ref_columns, *, name = None, on_delete = None, on_update = None))]
    fn new(
        columns: Vec<String>,
        references: IdentExtractor,
        ref_columns: Vec<String>,
        name: Option<&str>,
        on_delete: Option<&str>,
        on_update: Option<&str>,
    ) -> exc::Result<pyo3::PyClassInitializer<Self>> {
        let on_delete = match on_delete {
            Some(x) => Some(
                ReferentialAction::from_str(x)
                    .map_err(|_| exc::Error::invalid("on_delete", format!("invalid value: {x:?}")))?,
            ),
            None => None,
        };
        let on_update = match on_update {
            Some(x) => Some(
                ReferentialAction::from_str(x)
                    .map_err(|_| exc::Error::invalid("on_update", format!("invalid value: {x:?}")))?,
            ),
            None => None,
        };

        if columns.is_empty() {
            return Err(exc::Error::invalid("columns", "at least one column is required"));
        }
        if ref_columns.is_empty() {
            return Err(exc::Error::invalid(
                "ref_columns",
                "at least one ref_column is required",
            ));
        }

        let base = PyConstraint {
            name: name.map(Into::into),
        };
        let slf = Self {
            columns: columns.into_iter().map(Into::into).collect(),
            references: references.0,
            ref_columns: ref_columns.into_iter().map(Into::into).collect(),
            on_delete,
            on_update,
        };

        Ok(pyo3::PyClassInitializer::from(base).add_subclass(slf))
    }

    #[getter]
    fn columns<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        unsafe {
            utils::iterator_to_tuple(
                py,
                self.columns.len() as isize,
                self.columns.iter().map(|col| {
                    let pystr = pyo3::types::PyString::new(py, col.as_str());
                    Ok(pystr.into_ptr())
                }),
            )
        }
    }

    #[getter]
    fn references<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.references.clone().into())
    }

    #[getter]
    fn ref_columns<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<alias::BoundObject<'py>> {
        unsafe {
            utils::iterator_to_tuple(
                py,
                self.ref_columns.len() as isize,
                self.ref_columns.iter().map(|col| {
                    let pystr = pyo3::types::PyString::new(py, col.as_str());
                    Ok(pystr.into_ptr())
                }),
            )
        }
    }

    #[getter]
    fn on_delete(&self) -> Option<&'static str> {
        self.on_delete.map(Into::into)
    }

    #[getter]
    fn on_update(&self) -> Option<&'static str> {
        self.on_update.map(Into::into)
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let name = slf.as_super().name.clone();
        slf.python_repr(name.as_deref()).into_string()
    }
}

/// `CHECK` constraint.
///
/// ```ignore
/// [CONSTRAINT <name>] CHECK (<expression>)
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.CheckConstraint", extends = PyConstraint)]
#[derive(Debug, Clone)]
pub struct PyCheckConstraint {
    /// Predicate inside `CHECK (...)`.
    pub expression: ExprInner,
}

impl PyCheckConstraint {
    pub fn python_repr(&self, py: pyo3::Python<'_>, name: Option<&str>) -> exc::Result<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("CheckConstraint");
        fmt.positional(self.expression.python_repr(py)?).keyword("name", name);
        Ok(fmt)
    }
}

impl sql::WriteSql for PyCheckConstraint {
    #[inline]
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        if !dialect.table.features.contains(dialect::TableFeatures::CHECK) {
            return Err(exc::Error::dialect("Table-level CHECK constraint"));
        }

        out.write_str("CHECK (");
        self.expression.write_sql(dialect, out)?;
        out.write(')');

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyCheckConstraint {
    #[new]
    #[pyo3(signature = (expression, *, name = None))]
    fn new(expression: LiteralExprExtractor, name: Option<&str>) -> exc::Result<pyo3::PyClassInitializer<Self>> {
        let base = PyConstraint {
            name: name.map(Into::into),
        };
        let slf = Self {
            expression: expression.0,
        };

        Ok(pyo3::PyClassInitializer::from(base).add_subclass(slf))
    }

    #[getter]
    fn expression<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.expression.clone())
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>, py: pyo3::Python<'_>) -> exc::Result<String> {
        let name = slf.as_super().name.clone();
        Ok(slf.python_repr(py, name.as_deref())?.into_string())
    }
}
