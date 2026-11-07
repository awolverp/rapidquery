use crate::dialect;
use crate::exc;
use crate::expr::enum_::Ident;
use crate::expr::extractors::IdentExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::sql;

/// Represent a `CREATE SCHEMA` statement.
///
/// ```ignore
/// CREATE {SCHEMA | DATABASE} [IF NOT EXISTS] <name>
/// [AUTHORIZATION <authorization>]
/// [*LOCATION <location>]
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.CreateSchema", extends=super::PyDataDefinitionLanguage)]
#[derive(Debug, Clone)]
pub struct PyCreateSchema {
    /// Schema name e.g. `"datasets"`
    pub name: Ident,

    /// Emits `IF NOT EXISTS`.
    pub if_not_exists: bool,

    /// Authorization name.
    pub authorization: Option<arcstr::ArcStr>,

    /// Location
    pub location: Option<arcstr::ArcStr>,
    // TODO: support embedded DDL statements
}

impl sql::WriteSql for PyCreateSchema {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        if !dialect.schema.features.contains(dialect::SchemaFeatures::SUPPORTED) {
            return Err(exc::Error::dialect("CREATE SCHEMA"));
        }

        if dialect.schema.features.contains(dialect::SchemaFeatures::IS_DATABASE) {
            out.write_str("CREATE DATABASE ");
        } else {
            out.write_str("CREATE SCHEMA ");
        }

        out.write_str_if(
            self.if_not_exists && dialect.schema.features.contains(dialect::SchemaFeatures::IF_NOT_EXISTS),
            "IF NOT EXISTS ",
        );

        match dialect.schema.authorization {
            dialect::SchemaAuthorization::Unsupported => {
                if self.authorization.is_some() {
                    return Err(exc::Error::dialect("CREATE SCHEMA ... AUTHORIZATION"));
                }

                self.name.write_sql(dialect, out)?;
            }
            dialect::SchemaAuthorization::Optional => {
                self.name.write_sql(dialect, out)?;

                if let Some(x) = &self.authorization {
                    out.write_str(" AUTHORIZATION ");
                    dialect.write_identifier(out, x);
                }
            }
        }

        if let Some(x) = &self.location {
            let Some(tmpl) = &dialect.schema.location else {
                return Err(exc::Error::dialect("CREATE SCHEMA ... LOCATION"));
            };

            out.write(' ');
            tmpl.write_to_result(out, |_, out| dialect.string.write_string_literal(out, x.as_str()))?;
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyCreateSchema {
    #[new]
    #[pyo3(signature=(name, *, if_not_exists=false, authorization=None, location=None))]
    fn new(
        name: IdentExtractor,
        if_not_exists: bool,
        authorization: Option<&str>,
        location: Option<&str>,
    ) -> pyo3::PyClassInitializer<Self> {
        let result = Self {
            name: name.0,
            if_not_exists,
            authorization: authorization.map(Into::into),
            location: location.map(Into::into),
        };
        pyo3::PyClassInitializer::from(super::PyDataDefinitionLanguage).add_subclass(result)
    }

    #[getter]
    fn name<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.name.clone().into())
    }

    #[getter]
    fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    #[getter]
    fn authorization(&self) -> Option<String> {
        self.authorization.as_ref().map(|x| x.to_string())
    }

    #[getter]
    fn location(&self) -> Option<String> {
        self.location.as_ref().map(|x| x.to_string())
    }

    /// Render this expression as raw SQL (values inlined as literals).
    fn to_sql(&self, dialect: pyo3::Bound<'_, crate::dialect::PyDialect>) -> exc::Result<String> {
        sql::write_sql(dialect.py(), self, dialect.get())
    }

    fn __repr__(&self) -> String {
        let mut fmt = repr::ReprFormatter::new_call("CreateSchema");

        fmt.positional(self.name.python_repr())
            .keyword_true("if_not_exists", self.if_not_exists)
            .keyword("authorization", self.authorization.clone())
            .keyword("location", self.location.clone());

        fmt.into_string()
    }
}

/// Represents `CASCADE` and `RESTRICT` keywords in `DROP SCHEMA` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum DropSchemaBehavior {
    /// Emits `CASCADE`.
    #[strum(serialize = "CASCADE")]
    Cascade,
    /// Emits `RESTRICT`.
    #[strum(serialize = "RESTRICT")]
    Restrict,
}

/// Represent a `DROP SCHEMA` statement.
///
/// ```ignore
/// DROP {SCHEMA | DATABASE} [IF EXISTS] <name> [CASCADE | RESTRICT]
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.DropSchema", extends=super::PyDataDefinitionLanguage)]
#[derive(Debug, Clone)]
pub struct PyDropSchema {
    /// Schema name e.g. `"datasets"`
    pub name: Ident,

    /// Emits `IF EXISTS`.
    pub if_exists: bool,

    /// `CASCADE` and `RESTRICT` keywords
    pub behavior: Option<DropSchemaBehavior>,
}

impl sql::WriteSql for PyDropSchema {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &crate::dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        if !dialect.schema.features.contains(dialect::SchemaFeatures::SUPPORTED) {
            return Err(exc::Error::dialect("DROP SCHEMA"));
        }

        if dialect.schema.features.contains(dialect::SchemaFeatures::IS_DATABASE) {
            out.write_str("DROP DATABASE ");
        } else {
            out.write_str("DROP SCHEMA ");
        }

        // ... [IF EXISTS]
        out.write_str_if(
            self.if_exists && dialect.schema.features.contains(dialect::SchemaFeatures::IF_NOT_EXISTS),
            "IF EXISTS ",
        );

        self.name.write_sql(dialect, out)?;

        match self.behavior {
            None => {}
            Some(DropSchemaBehavior::Cascade) => {
                if !dialect.schema.features.contains(dialect::SchemaFeatures::DROP_CASCADE) {
                    return Err(exc::Error::dialect("DROP SCHEMA ... CASCADE"));
                }

                out.write_str(" CASCADE");
            }
            Some(DropSchemaBehavior::Restrict) => {
                if !dialect.schema.features.contains(dialect::SchemaFeatures::DROP_RESTRICT) {
                    return Err(exc::Error::dialect("DROP SCHEMA ... RESTRICT"));
                }

                out.write_str(" RESTRICT");
            }
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyDropSchema {
    #[new]
    #[pyo3(signature=(name, *, if_exists=false, behavior=None))]
    fn new(
        name: IdentExtractor,
        if_exists: bool,
        behavior: Option<&str>,
    ) -> exc::Result<pyo3::PyClassInitializer<Self>> {
        use std::str::FromStr;

        let result = Self {
            name: name.0,
            if_exists,
            behavior: match behavior {
                None => None,
                Some(x) => Some(DropSchemaBehavior::from_str(x).map_err(|_| {
                    exc::Error::invalid("behavior", format!("expected 'CASCADE' or 'RESTRICT', received {x}"))
                })?),
            },
        };
        Ok(pyo3::PyClassInitializer::from(super::PyDataDefinitionLanguage).add_subclass(result))
    }

    #[getter]
    fn name<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        crate::expr::base::into_subclasses(py, self.name.clone().into())
    }

    #[getter]
    fn if_exists(&self) -> bool {
        self.if_exists
    }

    #[getter]
    fn behavior(&self) -> Option<&'static str> {
        self.behavior.map(Into::into)
    }

    /// Render this expression as raw SQL (values inlined as literals).
    fn to_sql(&self, dialect: pyo3::Bound<'_, crate::dialect::PyDialect>) -> exc::Result<String> {
        sql::write_sql(dialect.py(), self, dialect.get())
    }

    fn __repr__(&self) -> String {
        let mut fmt = repr::ReprFormatter::new_call("CreateSchema");

        fmt.positional(self.name.python_repr())
            .keyword_true("if_exists", self.if_exists)
            .keyword("behavior", self.behavior.map(Into::<&'static str>::into));

        fmt.into_string()
    }
}
