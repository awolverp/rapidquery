use std::str::FromStr;

use crate::data_type;
use crate::exc;
use crate::expr::enum_::ExprInner;
use crate::expr::extractors::LiteralExprExtractor;
use crate::internal::OptionalExtractor;
use crate::internal::alias;
use crate::internal::repr;
use crate::sql;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum ColumnGeneratedType {
    /// Emits `VIRTUAL`.
    #[strum(serialize = "VIRTUAL")]
    Virtual,
    /// Emits `PERSISTED`.
    #[strum(serialize = "PERSISTED")]
    Persisted,
    /// Emits `STORED`.
    #[strum(serialize = "STORED")]
    Stored,
}

/// Represents `GENERATED ALWAYS AS ...` clause
#[derive(Debug, Clone)]
pub struct ColumnGenerated {
    pub expression: ExprInner,
    pub generated_type: Option<ColumnGeneratedType>,
}

/// Column definition.
///
/// ```ignore
/// <name> <data_type> [GENERATED ALWAYS AS (<expression>) [VIRTUAL | STORED | PERSISTED]]
/// [PRIMARY KEY | UNIQUE] [AUTOINCREMENT] {NULL | NOT NULL} [DEFAULT <default>] [CHECK (<check>)]
/// [COMMENT <comment>] <extra>
/// ```
#[rapidquery_macros::pyclass("rapidquery.ddl.Column")]
#[derive(Debug, Clone)]
pub struct PyColumn {
    /// Column name.
    pub name: arcstr::ArcStr,

    /// Column data type name.
    pub data_type: data_type::DataTypeName,

    /// Emit `PRIMARY KEY` keyword
    pub primary_key: bool,

    /// Emit `UNIQUE` keyword
    pub unique: bool,

    /// Whether is nullable or not
    pub nullable: bool,

    /// Default value
    pub default: Option<ExprInner>,

    /// `CHECK` constraint
    pub check: Option<Box<ExprInner>>,

    /// Extra raw SQL
    pub extra: Option<arcstr::ArcStr>,

    // --- Dialect specific options ---
    /// Comment
    pub comment: Option<arcstr::ArcStr>,

    /// Whether is auto increment
    pub auto_increment: bool,

    /// `GENERATED ALWAYS AS` expression
    pub generated: Option<Box<ColumnGenerated>>,
}

impl PyColumn {
    pub fn python_repr(&self, py: pyo3::Python) -> exc::Result<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("Column");

        fmt.positional(self.name.as_str())
            .positional(self.data_type.as_str())
            .keyword_true("primary_key", self.primary_key)
            .keyword_true("unique", self.unique)
            .keyword_false("nullable", self.nullable);

        if let Some(x) = &self.default {
            fmt.keyword("default", x.python_repr(py)?);
        }
        if let Some(x) = &self.check {
            fmt.keyword("check", x.python_repr(py)?);
        }

        fmt.keyword("comment", self.comment.as_ref().map(|x| x.as_str()))
            .keyword_true("auto_increment", self.auto_increment);

        if let Some(generated) = &self.generated {
            fmt.keyword("generated", generated.expression.python_repr(py)?);
            fmt.keyword(
                "generated_type",
                generated.generated_type.map(Into::<&'static str>::into),
            );
        }

        fmt.keyword("extra", self.extra.as_ref().map(|x| x.as_str()));

        Ok(fmt)
    }
}

impl sql::WriteSql for PyColumn {
    fn write_sql<W: sql::SqlWriter>(&self, dialect: &crate::dialect::PyDialect, out: &mut W) -> exc::Result<()> {
        dialect.write_identifier(out, &self.name);
        out.write(' ');
        self.data_type.write_sql(dialect, out)?;

        if let Some(x) = &self.generated {
            if !dialect
                .table
                .features
                .contains(crate::dialect::TableFeatures::GENERATED_SUPPORTED)
            {
                return Err(exc::Error::dialect("Column.generated"));
            }

            out.write_str(" GENERATED ALWAYS AS (");
            x.expression.write_sql(dialect, out)?;
            out.write(')');

            if let Some(typ) = &x.generated_type {
                out.write(' ');
                out.write_str(typ.into());
            }
        }

        out.write_str_if(self.primary_key, " PRIMARY KEY");
        out.write_str_if(self.unique, " UNIQUE");

        if self.auto_increment {
            let Some(tmpl) = &dialect.table.column_auto_increment else {
                return Err(exc::Error::dialect("Column.auto_increment"));
            };

            out.write(' ');
            out.write_str(tmpl.as_str());
        }

        if self.nullable {
            out.write_str(" NULL");
        } else {
            out.write_str(" NOT NULL");
        }

        if let Some(x) = &self.default {
            out.write_str(" DEFAULT ");
            x.write_sql(dialect, out)?;
        }

        if let Some(x) = &self.check {
            out.write_str(" CHECK (");
            x.write_sql(dialect, out)?;
            out.write(')');
        }

        if let Some(x) = &self.comment
            && let Some(tmpl) = &dialect.table.column_comment
        {
            out.write(' ');
            tmpl.write_to(out, |_, out| dialect.string.write_string_literal(out, x));
        }

        if let Some(x) = &self.extra {
            out.write(' ');
            out.write_str(x.as_str());
        }

        Ok(())
    }
}

#[pyo3::pymethods]
impl PyColumn {
    #[new]
    #[
        pyo3(
            signature = (
                name,
                data_type,
                /,
                primary_key=false,
                unique=false,
                nullable=None,
                default=OptionalExtractor::Undefined,
                check=OptionalExtractor::Undefined,
                comment=None,
                auto_increment=false,
                generated=OptionalExtractor::Undefined,
                generated_type=None,
                *,
                extra=None,
            )
        )
    ]
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: &str,
        data_type: &str,
        primary_key: bool,
        unique: bool,
        nullable: Option<bool>,
        default: OptionalExtractor<LiteralExprExtractor>,
        check: OptionalExtractor<LiteralExprExtractor>,
        comment: Option<&str>,
        auto_increment: bool,
        generated: OptionalExtractor<LiteralExprExtractor>,
        generated_type: Option<&str>,
        extra: Option<&str>,
    ) -> exc::Result<Self> {
        let generated_type = generated_type
            .map(ColumnGeneratedType::from_str)
            .transpose()
            .map_err(|_| {
                exc::Error::invalid(
                    "generated_type",
                    format!(
                        "expected 'VIRTUAL', 'PERSISTED' or 'STORED', received {}",
                        generated_type.unwrap()
                    ),
                )
            })?;

        let nullable = match nullable {
            Some(x) => x,
            None => !primary_key && !auto_increment,
        };

        Ok(Self {
            name: name.into(),
            data_type: data_type.into(),
            primary_key,
            // primary_key columns are unique by default, so `UNIQUE` keyword
            // is completely meaningless if primary_key=true
            unique: !primary_key && unique,
            nullable,
            default: default.ok().map(|x| x.0),
            check: check.ok().map(|x| Box::new(x.0)),
            extra: extra.map(Into::into),
            comment: comment.map(Into::into),
            auto_increment,
            generated: generated
                .ok()
                .map(|x| ColumnGenerated {
                    expression: x.0,
                    generated_type,
                })
                .map(Box::new),
        })
    }

    #[getter]
    fn get_name(&self) -> &str {
        &self.name
    }

    #[getter]
    fn get_data_type(&self) -> &str {
        self.data_type.as_str()
    }

    #[getter]
    fn get_primary_key(&self) -> bool {
        self.primary_key
    }

    #[getter]
    fn get_unique(&self) -> bool {
        self.unique
    }

    #[getter]
    fn get_nullable(&self) -> bool {
        self.nullable
    }

    #[getter]
    fn get_default<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<Option<alias::BoundObject<'a>>> {
        self.default
            .as_ref()
            .map(|x| crate::expr::base::into_subclasses(py, x.clone()))
            .transpose()
    }

    #[getter]
    fn get_check<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<Option<alias::BoundObject<'a>>> {
        self.check
            .as_ref()
            .map(|x| crate::expr::base::into_subclasses(py, *x.clone()))
            .transpose()
    }

    #[getter]
    fn get_extra(&self) -> Option<&str> {
        self.extra.as_ref().map(arcstr::ArcStr::as_str)
    }

    #[getter]
    fn get_comment(&self) -> Option<&str> {
        self.comment.as_ref().map(arcstr::ArcStr::as_str)
    }

    #[getter]
    fn get_auto_increment(&self) -> bool {
        self.auto_increment
    }

    #[getter]
    fn get_generated<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<Option<alias::BoundObject<'a>>> {
        self.generated
            .as_ref()
            .map(|x| crate::expr::base::into_subclasses(py, x.expression.clone()))
            .transpose()
    }

    #[getter]
    fn get_generated_type(&self) -> Option<&str> {
        self.generated
            .as_ref()
            .and_then(|x| x.generated_type.map(Into::<&'static str>::into))
    }

    #[getter]
    fn __expr__<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        let ident = crate::expr::enum_::Ident::from_one(crate::expr::enum_::IdentPart::Quote(self.name.clone()));
        crate::expr::base::into_subclasses(py, ident.into())
    }

    #[getter]
    #[inline(always)]
    fn __ident__<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        self.__expr__(py)
    }

    #[inline(always)]
    fn to_expr<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<alias::BoundObject<'a>> {
        self.__expr__(py)
    }

    fn __repr__(&self, py: pyo3::Python) -> exc::Result<String> {
        self.python_repr(py).map(repr::ReprFormatter::into_string)
    }
}
