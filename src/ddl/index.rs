use std::str::FromStr;

use crate::dialect;
use crate::error;
use crate::expr::enum_::ExprInner;
use crate::expr::enum_::Ident;
use crate::expr::extractors::IdentExtractor;
use crate::expr::extractors::LiteralExprExtractor;
use crate::expr::extractors::NameExprExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::sql;

/// Represents `ASC` and `DESC` keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum IndexOrder {
    /// Emits `ASC`.
    #[strum(serialize = "ASC")]
    Asc,
    /// Emits `DESC`.
    #[strum(serialize = "DESC")]
    Desc,
}

/// One key of a `CREATE INDEX` column list.
#[rapidquery_macros::pyclass("rapidquery.ddl.IndexColumn")]
#[derive(Debug, Clone)]
pub struct PyIndexColumn {
    /// Index column expression.
    pub column: ExprInner,

    /// Index column order.
    pub order: IndexOrder,
}

impl PyIndexColumn {
    #[inline]
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> error::Result<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("IndexColumn");

        fmt.positional(self.column.python_repr(py)?);
        if matches!(self.order, IndexOrder::Desc) {
            fmt.keyword("order", "DESC");
        }

        Ok(fmt)
    }
}

#[pyo3::pymethods]
impl PyIndexColumn {
    #[new]
    #[pyo3(signature = (column, *, order = "ASC"))]
    fn new(column: NameExprExtractor, order: &str) -> error::Result<Self> {
        let order = IndexOrder::from_str(order).map_err(|_| error::Error::InvalidIndexOrder {
            value: order.to_owned(),
        })?;
        Ok(Self {
            column: column.0,
            order,
        })
    }

    #[getter]
    fn column<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.column.clone())
    }

    #[getter]
    fn order(&self) -> &'static str {
        self.order.into()
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> error::Result<String> {
        self.python_repr(py).map(repr::ReprFormatter::into_string)
    }
}

/// Represents a `CREATE INDEX` statement.
///
/// ```ignore
/// CREATE [UNIQUE] INDEX [IF NOT EXISTS] <name>
/// ON <table>
/// [USING <index_type>]
/// (<column> [ASC | DESC], ...)
/// [USING <index_type>]
/// [INCLUDE (<include>, ...)]
/// [WHERE <predicate>]
/// ```
///
/// `table_name` may be omitted so a future `CREATE TABLE` can hold this index.
/// [`Self::to_sql`] rejects that shape: a standalone `CREATE INDEX` needs a table.
#[rapidquery_macros::pyclass("rapidquery.ddl.CreateIndex", extends = super::PyDataDefinitionLanguage)]
#[derive(Debug, Clone)]
pub struct PyCreateIndex {
    /// Index name, a single identifier such as `"idx_users_user_id"`.
    pub name: arcstr::ArcStr,

    /// `CREATE UNIQUE INDEX` when set.
    pub unique: bool,

    /// `IF NOT EXISTS`. Omitted when the dialect does not support it.
    pub if_not_exists: bool,

    /// Table name.
    pub table_name: Ident,

    /// Index columns. At least one.
    pub columns: Vec<pyo3::Py<PyIndexColumn>>,

    /// Partial index predicate (`WHERE`).
    pub predicate: Option<ExprInner>,

    /// Included columns (`INCLUDE`). Each entry is one identifier.
    pub include: Vec<arcstr::ArcStr>,

    /// Index access method, written unchanged after `USING`.
    ///
    /// Placement depends on the dialect: before the column list, or after it.
    pub index_type: Option<arcstr::ArcStr>,
}

impl sql::WriteSql for PyCreateIndex {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> error::Result<()> {
        let features = dialect.index.features;

        if !features.contains(dialect::IndexFeatures::SUPPORTED) {
            return Err(error::Error::Unsupported { feature: "CreateIndex" });
        }

        // `Some((before_columns, text))`
        let using = match (&self.index_type, dialect.index.index_type) {
            (None, _) => None,
            (Some(_), dialect::IndexType::Unsupported) => {
                return Err(error::Error::Unsupported {
                    feature: "CreateIndex.index_type",
                });
            }
            (Some(text), dialect::IndexType::BeforeColumns) => Some((true, text.as_str())),
            (Some(text), dialect::IndexType::AfterColumns) => Some((false, text.as_str())),
        };

        if self.unique {
            if !features.contains(dialect::IndexFeatures::UNIQUE) {
                return Err(error::Error::Unsupported {
                    feature: "CreateIndex.unique",
                });
            }
            out.write_str("CREATE UNIQUE INDEX ");
        } else {
            out.write_str("CREATE INDEX ");
        }
        out.write_str_if(
            self.if_not_exists && features.contains(dialect::IndexFeatures::IF_NOT_EXISTS),
            "IF NOT EXISTS ",
        );
        dialect.write_identifier(out, self.name.as_str());

        out.write_str(" ON ");
        self.table_name.write_sql(dialect, out)?;

        if let Some((true, text)) = using {
            out.write_str(" USING ");
            out.write_str(text);
        }

        let column_order = features.contains(dialect::IndexFeatures::COLUMN_ORDER);
        let expressions = features.contains(dialect::IndexFeatures::EXPRESSIONS);

        // SAFETY: `Python` is a zero-sized marker and the GIL is held for the whole
        // call, so extending its lifetime only detaches it from the borrow of `out`.
        // It must not escape this function.
        let py: pyo3::Python<'static> = unsafe { std::mem::transmute(out.py()) };

        out.write_str(" (");
        for (index, column) in self.columns.iter().enumerate() {
            let column = column.bind(py).get();

            if !expressions && !matches!(column.column, ExprInner::Ident(_)) {
                return Err(error::Error::Unsupported {
                    feature: "CreateIndex.expression",
                });
            }

            if !column_order && matches!(column.order, IndexOrder::Desc) {
                return Err(error::Error::Unsupported {
                    feature: "CreateIndex.order",
                });
            }

            if index > 0 {
                out.write_str(", ");
            }
            match &column.column {
                ExprInner::Ident(ident) => ident.write_sql(dialect, out)?,
                other => {
                    out.write('(');
                    other.write_sql(dialect, out)?;
                    out.write(')');
                }
            }

            if column_order {
                out.write(' ');
                out.write_str(column.order.into());
            }
        }
        out.write(')');

        if let Some((false, text)) = using {
            out.write_str(" USING ");
            out.write_str(text);
        }

        if !self.include.is_empty() {
            if !features.contains(dialect::IndexFeatures::INCLUDE) {
                return Err(error::Error::Unsupported {
                    feature: "CreateIndex.include",
                });
            }

            out.write_str(" INCLUDE (");
            for (index, name) in self.include.iter().enumerate() {
                if index > 0 {
                    out.write_str(", ");
                }
                dialect.write_identifier(out, name.as_str());
            }
            out.write(')');
        }

        if let Some(predicate) = &self.predicate {
            if !features.contains(dialect::IndexFeatures::PARTIAL) {
                return Err(error::Error::Unsupported {
                    feature: "CreateIndex.predicate",
                });
            }

            out.write_str(" WHERE ");
            predicate.write_sql(dialect, out)?;
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyCreateIndex {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        name,
        table_name,
        columns,
        *,
        unique = false,
        if_not_exists = false,
        include = None,
        predicate = None,
        index_type = None
    ))]
    fn new(
        name: &str,
        table_name: IdentExtractor,
        columns: Vec<pyo3::Py<PyIndexColumn>>,
        unique: bool,
        if_not_exists: bool,
        include: Option<Vec<String>>,
        predicate: Option<LiteralExprExtractor>,
        index_type: Option<&str>,
    ) -> error::Result<pyo3::PyClassInitializer<Self>> {
        if name.is_empty() {
            return Err(error::Error::EmptyIndexName);
        }
        if columns.is_empty() {
            return Err(error::Error::EmptyIndexColumns);
        }

        let include = include.unwrap_or_default();
        if include.iter().any(String::is_empty) {
            return Err(error::Error::EmptyIndexInclude);
        }

        let index_type = match index_type {
            None => None,
            Some("") => return Err(error::Error::EmptyIndexType),
            Some(text) => Some(arcstr::ArcStr::from(text)),
        };

        let result = Self {
            name: arcstr::ArcStr::from(name),
            unique,
            if_not_exists,
            table_name: table_name.0,
            columns,
            predicate: predicate.map(|predicate| predicate.0),
            include: include.into_iter().map(arcstr::ArcStr::from).collect(),
            index_type,
        };
        Ok(pyo3::PyClassInitializer::from(super::PyDataDefinitionLanguage).add_subclass(result))
    }

    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    #[getter]
    fn unique(&self) -> bool {
        self.unique
    }

    #[getter]
    fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    #[getter]
    fn table_name<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.table_name.clone().into())
    }

    #[getter]
    fn columns(&self) -> Vec<pyo3::Py<PyIndexColumn>> {
        self.columns.clone()
    }

    #[getter]
    fn predicate<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<Option<alias::BoundObject<'a>>> {
        self.predicate
            .as_ref()
            .map(|predicate| crate::expr::base::into_subclasses(py, predicate.clone()))
            .transpose()
    }

    #[getter]
    fn include(&self) -> Vec<String> {
        self.include.iter().map(|name| name.to_string()).collect()
    }

    #[getter]
    fn index_type(&self) -> Option<String> {
        self.index_type.as_ref().map(|text| text.to_string())
    }

    /// Render this statement as raw SQL (values inlined as literals).
    fn to_sql(&self, dialect: pyo3::Bound<'_, crate::dialect::PyDialect>) -> error::Result<String> {
        sql::write_sql(dialect.py(), self, dialect.get())
    }

    fn __repr__(&self, py: pyo3::Python<'_>) -> error::Result<String> {
        let mut fmt = repr::ReprFormatter::new_call("CreateIndex");

        fmt.positional(self.name.as_str())
            .positional(self.table_name.python_repr());

        {
            let mut list = fmt.positional_list();
            for column in &self.columns {
                let column = column.bind(py).get();
                list.positional(column.python_repr(py)?);
            }
        }

        fmt.keyword_true("unique", self.unique)
            .keyword_true("if_not_exists", self.if_not_exists);

        if !self.include.is_empty() {
            let mut include = repr::ReprFormatter::new_list();
            for name in &self.include {
                include.positional(name.as_str());
            }
            fmt.keyword("include", include);
        }

        if let Some(predicate) = &self.predicate {
            fmt.keyword("predicate", predicate.python_repr(py)?);
        }
        fmt.keyword("index_type", self.index_type.clone());

        Ok(fmt.into_string())
    }
}

/// `CASCADE` and `RESTRICT` in `DROP INDEX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum DropIndexBehavior {
    /// Emits `CASCADE`.
    #[strum(serialize = "CASCADE")]
    Cascade,
    /// Emits `RESTRICT`.
    #[strum(serialize = "RESTRICT")]
    Restrict,
}

/// Represents a `DROP INDEX` statement.
///
/// ```ignore
/// DROP INDEX [IF EXISTS] <name> [ON <table>] [CASCADE | RESTRICT]
/// DROP INDEX [IF EXISTS] <table>.<name> [CASCADE | RESTRICT]
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.DropIndex", extends = super::PyDataDefinitionLanguage)]
#[derive(Debug, Clone)]
pub struct PyDropIndex {
    /// Index name. One identifier.
    pub name: arcstr::ArcStr,

    /// Table name. Required for `ON` and `table.name` dialects, rejected for name-only dialects.
    pub table_name: Option<Ident>,

    /// `IF EXISTS`. Omitted when the dialect lacks `IF_NOT_EXISTS`.
    pub if_exists: bool,

    /// `CASCADE` or `RESTRICT`.
    pub behavior: Option<DropIndexBehavior>,
}

impl sql::WriteSql for PyDropIndex {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> error::Result<()> {
        let drop_features = dialect.index.drop_features;
        if !drop_features.contains(dialect::IndexDropFeatures::SUPPORTED) {
            return Err(error::Error::Unsupported { feature: "DropIndex" });
        }

        match self.behavior {
            Some(DropIndexBehavior::Cascade) if !drop_features.contains(dialect::IndexDropFeatures::CASCADE) => {
                return Err(error::Error::Unsupported {
                    feature: "DropIndex.behavior = 'CASCADE'",
                });
            }
            Some(DropIndexBehavior::Restrict) if !drop_features.contains(dialect::IndexDropFeatures::RESTRICT) => {
                return Err(error::Error::Unsupported {
                    feature: "DropIndex.behavior = 'RESTRICT'",
                });
            }
            _ => {}
        }

        out.write_str("DROP INDEX ");
        out.write_str_if(
            self.if_exists && dialect.index.features.contains(dialect::IndexFeatures::IF_NOT_EXISTS),
            "IF EXISTS ",
        );

        match (dialect.index.drop_table, &self.table_name) {
            (dialect::IndexDropTable::NameOnly, None) => {
                dialect.write_identifier(out, self.name.as_str());
            }
            (dialect::IndexDropTable::On, Some(table)) => {
                dialect.write_identifier(out, self.name.as_str());
                out.write_str(" ON ");
                table.write_sql(dialect, out)?;
            }
            (dialect::IndexDropTable::Qualified, Some(table)) => {
                table.write_sql(dialect, out)?;
                out.write('.');
                dialect.write_identifier(out, self.name.as_str());
            }

            (dialect::IndexDropTable::NameOnly, Some(_)) => {
                // ignore table_name
            }
            (dialect::IndexDropTable::On | dialect::IndexDropTable::Qualified, None) => {
                return Err(error::Error::MissingDropIndexTable);
            }
        }

        match self.behavior {
            Some(DropIndexBehavior::Cascade) => out.write_str(" CASCADE"),
            Some(DropIndexBehavior::Restrict) => out.write_str(" RESTRICT"),
            None => {}
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyDropIndex {
    #[new]
    #[pyo3(signature = (name, *, table_name = None, if_exists = false, behavior = None))]
    fn new(
        name: &str,
        table_name: Option<IdentExtractor>,
        if_exists: bool,
        behavior: Option<&str>,
    ) -> error::Result<pyo3::PyClassInitializer<Self>> {
        if name.is_empty() {
            return Err(error::Error::EmptyIndexName);
        }

        let behavior = match behavior {
            None => None,
            Some(value) => {
                Some(
                    DropIndexBehavior::from_str(value).map_err(|_| error::Error::InvalidIndexBehavior {
                        value: value.to_owned(),
                    })?,
                )
            }
        };

        let result = Self {
            name: arcstr::ArcStr::from(name),
            table_name: table_name.map(|table| table.0),
            if_exists,
            behavior,
        };
        Ok(pyo3::PyClassInitializer::from(super::PyDataDefinitionLanguage).add_subclass(result))
    }

    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    #[getter]
    fn table_name<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<Option<alias::BoundObject<'a>>> {
        self.table_name
            .as_ref()
            .map(|name| crate::expr::base::into_subclasses(py, name.clone().into()))
            .transpose()
    }

    #[getter]
    fn if_exists(&self) -> bool {
        self.if_exists
    }

    #[getter]
    fn behavior(&self) -> Option<&'static str> {
        self.behavior.map(Into::into)
    }

    /// Render this statement as raw SQL.
    fn to_sql(&self, dialect: pyo3::Bound<'_, crate::dialect::PyDialect>) -> error::Result<String> {
        sql::write_sql(dialect.py(), self, dialect.get())
    }

    fn __repr__(&self) -> String {
        let mut fmt = repr::ReprFormatter::new_call("DropIndex");

        fmt.positional(self.name.as_str());
        if let Some(table_name) = &self.table_name {
            fmt.keyword("table_name", table_name.python_repr());
        }
        fmt.keyword_true("if_exists", self.if_exists)
            .keyword("behavior", self.behavior.map(Into::<&'static str>::into));

        fmt.into_string()
    }
}
