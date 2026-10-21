use crate::dialect::config;
use crate::expr::enum_::Ident;
use crate::expr::extractors::IdentExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::internal::sql;

/// Represent a `CREATE SCHEMA` statement.
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
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &crate::dialect::PyDialect, out: &mut W) -> sql::PrepareResult<()> {
        if dialect.0.schema_features.contains(config::SchemaFeatures::UNSUPPORTED) {
            return Err(rapidquery_macros::pyerr!(
                PyRuntimeError,
                "CREATE SCHEMA statement is not supported by this dialect"
            )
            .into());
        }

        // CREATE SCHEMA
        out.push_str("CREATE SCHEMA ");

        // ... [IF NOT EXISTS]
        if self.if_not_exists
            && dialect
                .0
                .schema_features
                .contains(config::SchemaFeatures::IF_NOT_EXISTS)
        {
            out.push_str("IF NOT EXISTS ");
        }

        // ... [name] AUTHORIZATION [owner_name]
        match dialect.0.schema_authorization_style {
            config::SchemaAuthorizationStyle::Unsupported => {
                if self.authorization.is_some() {
                    return Err(rapidquery_macros::pyerr!(
                        PyRuntimeError,
                        "authorization is not supported in CREATE SCHEMA by this dialect"
                    )
                    .into());
                }

                self.name.write_sql(dialect, out)?;
            }
            config::SchemaAuthorizationStyle::Optional => {
                self.name.write_sql(dialect, out)?;

                if let Some(x) = &self.authorization {
                    out.push_str(" AUTHORIZATION");
                    dialect.0.write_identifier(out, x)?;
                }
            }
            config::SchemaAuthorizationStyle::Forced => {
                // If we have authorization, it should be equal to name.
                if let Some(x) = &self.authorization {
                    let parts = self.name.parts();

                    if parts.len() > 1 || parts[0] != x.as_str() {
                        return Err(rapidquery_macros::pyerr!(
                            PyRuntimeError,
                            "AUTHORIZATION in CREATE SCHEMA is required by this dialect and schema \
                            name needs to be equal to authorization. omit authorization, or set same name and authorization."
                        )
                        .into());
                    }
                }

                out.push_str("AUTHORIZATION ");
                self.name.write_sql(dialect, out)?;
            }
        }

        if let Some(x) = &self.location {
            // ... (LOCATION | MANAGED LOCATION | EXTERNAL_VOLUME =) location
            match dialect.0.schema_location_style {
                config::SchemaLocationStyle::Unsupported => {
                    return Err(rapidquery_macros::pyerr!(
                        PyRuntimeError,
                        "location is not supported in CREATE SCHEMA by this dialect"
                    )
                    .into());
                }
                config::SchemaLocationStyle::LocationKeyword => {
                    out.push_str(" LOCATION ");
                    dialect.0.write_string_literal(out, x)?;
                }
                config::SchemaLocationStyle::ManagedLocationKeyword => {
                    out.push_str(" MANAGED LOCATION ");
                    dialect.0.write_string_literal(out, x)?;
                }
                config::SchemaLocationStyle::ExternalVolume => {
                    out.push_str(" EXTERNAL_VOLUME = ");
                    dialect.0.write_string_literal(out, x)?;
                }
            }
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyCreateSchema {
    #[new]
    #[pyo3(signature=(name, if_not_exists=false, authorization=None, location=None))]
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
    fn to_sql(&self, py: pyo3::Python, dialect: crate::dialect::PyDialect) -> pyo3::PyResult<String> {
        sql::write_sql(py, self, &dialect).map_err(Into::into)
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
