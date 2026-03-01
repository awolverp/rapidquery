use sea_query::IntoIden;

use crate::expression::PyExpr;
use crate::sqltypes::NativeSQLType;
use crate::sqltypes::TypeEngine;
use crate::utils::OptionalParam;
use crate::utils::ToSeaQuery;

implement_state_pyclass! {
    /// Defines a table column with its properties and constraints.
    ///
    /// Represents a complete column definition including:
    /// - Column name and data type
    /// - Constraints (primary key, unique, nullable)
    /// - Auto-increment behavior
    /// - Default values and generated columns
    /// - Comments and extra specifications
    ///
    /// This class is used within Table to specify the structure
    /// of table columns. It encapsulates all the properties that define how
    /// a column behaves and what data it can store.
    ///
    /// @extends typing.Generic[I,O]
    /// @signature (
    ///     name: str,
    ///     type: SQLTypeAbstract[I,O],
    ///     options: int = ..., *,
    ///     extra: str | None = ...,
    ///     comment: str | None = ...,
    ///     default: typing.Any = ...,
    ///     generated: typing.Any = ...,
    /// )
    #[derive(Debug)]
    pub struct [] PyColumn(ColumnState) as "Column" {
        pub name: String,
        pub r#type: TypeEngine,
        pub options: u8,
        pub default: Option<PyExpr>,
        pub generated: Option<PyExpr>,
        pub extra: Option<String>,
        pub comment: Option<String>,
    }
}

impl Clone for ColumnState {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            r#type: self.r#type.clone(),
            options: self.options,
            default: self.default.clone(),
            generated: self.generated.clone(),
            extra: self.extra.clone(),
            comment: self.comment.clone(),
        }
    }
}

impl ToSeaQuery<sea_query::ColumnRef> for ColumnState {
    #[inline]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::ColumnRef {
        sea_query::ColumnRef::Column(sea_query::Alias::new(&self.name).into_iden())
    }
}

impl ToSeaQuery<sea_query::ColumnDef> for ColumnState {
    #[inline]
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::ColumnDef {
        let mut column_def = sea_query::ColumnDef::new_with_type(
            sea_query::Alias::new(self.name.clone()),
            self.r#type.to_sea_query_column_type(),
        );

        if self.options & PyColumn::OPT_PRIMARY_KEY > 0 {
            column_def.primary_key();
        }
        if self.options & PyColumn::OPT_AUTO_INCREMENT > 0 {
            column_def.auto_increment();
        }
        if self.options & PyColumn::OPT_NULLABLE > 0 {
            column_def.null();
        } else {
            column_def.not_null();
        }
        if self.options & PyColumn::OPT_UNIQUE_KEY > 0 {
            column_def.unique_key();
        }

        if let Some(x) = &self.default {
            column_def.default(x.0.clone());
        }
        if let Some(x) = &self.generated {
            column_def.generated(
                x.0.clone(),
                self.options & PyColumn::OPT_STORED_GENERATED > 0,
            );
        }

        if let Some(x) = &self.extra {
            column_def.extra(x);
        }
        if let Some(x) = &self.comment {
            column_def.comment(x);
        }

        column_def
    }
}

#[pyo3::pymethods]
impl PyColumn {
    #[classattr]
    pub const OPT_PRIMARY_KEY: u8 = 1 << 0;
    #[classattr]
    pub const OPT_UNIQUE_KEY: u8 = 1 << 1;
    #[classattr]
    pub const OPT_NULLABLE: u8 = 1 << 2;
    #[classattr]
    pub const OPT_AUTO_INCREMENT: u8 = 1 << 3;
    #[classattr]
    pub const OPT_STORED_GENERATED: u8 = 1 << 4;

    #[new]
    #[
        pyo3(
            signature=(
                name,
                r#type,
                options=0u8,
                *,
                extra=None,
                comment=None,
                default=OptionalParam::Undefined,
                generated=OptionalParam::Undefined,
            )
        )
    ]
    fn __new__(
        name: String,
        r#type: &pyo3::Bound<'_, pyo3::PyAny>,
        options: u8,
        extra: Option<String>,
        comment: Option<String>,
        default: OptionalParam,
        generated: OptionalParam,
    ) -> pyo3::PyResult<Self> {
        let sql_type = TypeEngine::new(r#type)?;

        let default_expr = match default {
            OptionalParam::Defined(x) => Some(crate::expression::PyExpr::try_from_specific_type(
                &x,
                Some(sql_type.clone()),
            )?),
            OptionalParam::Undefined => None,
        };
        let generated_expr = match generated {
            OptionalParam::Defined(x) => Some(crate::expression::PyExpr::try_from(&x)?),
            OptionalParam::Undefined => None,
        };

        let state = ColumnState {
            name,
            r#type: sql_type,
            options,
            default: default_expr,
            generated: generated_expr,
            extra,
            comment,
        };
        Ok(state.into())
    }

    /// Column name.
    ///
    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.0.lock().name.clone()
    }

    /// Column type.
    ///
    /// @signature (self) -> SQLTypeAbstract[I, O]
    #[getter]
    fn r#type(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.0.lock();
        lock.r#type.as_pyobject(py).unbind()
    }

    /// Column specified options.
    ///
    /// @signature (self) -> int
    /// @setter int
    #[getter]
    fn options(&self) -> u8 {
        self.0.lock().options
    }

    #[setter]
    fn set_options(&self, value: u8) {
        self.0.lock().options = value;
    }

    /// Shorthand for `self.options & OPT_PRIMARY_KEY > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn is_primary_key(&self) -> bool {
        self.0.lock().options & Self::OPT_PRIMARY_KEY > 0
    }

    /// Shorthand for `self.options & OPT_UNIQUE_KEY > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn is_unique_key(&self) -> bool {
        self.0.lock().options & Self::OPT_UNIQUE_KEY > 0
    }

    /// Shorthand for `self.options & OPT_AUTO_INCREMENT > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn is_auto_increment(&self) -> bool {
        self.0.lock().options & Self::OPT_AUTO_INCREMENT > 0
    }

    /// Shorthand for `self.options & OPT_NULLABLE > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn is_nullable(&self) -> bool {
        self.0.lock().options & Self::OPT_NULLABLE > 0
    }

    /// Shorthand for `self.options & OPT_STORED_GENERATED > 0`.
    ///
    /// @signature (self) -> bool
    #[getter]
    fn is_stored_generated(&self) -> bool {
        self.0.lock().options & Self::OPT_STORED_GENERATED > 0
    }

    /// Extra SQL specifications for this column.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn extra(&self) -> Option<String> {
        self.0.lock().extra.clone()
    }

    #[setter]
    fn set_extra(&self, val: Option<String>) {
        let mut lock = self.0.lock();
        lock.extra = val;
    }

    /// Comment describing this column.
    ///
    /// @signature (self) -> str | None
    /// @setter str | None
    #[getter]
    fn comment(&self) -> Option<String> {
        self.0.lock().comment.clone()
    }

    #[setter]
    fn set_comment(&self, val: Option<String>) {
        let mut lock = self.0.lock();
        lock.comment = val;
    }

    /// Default value for this column.
    ///
    /// @signature (self) -> Expr | None
    /// @setter Expr | None
    #[getter]
    fn default(&self) -> Option<PyExpr> {
        let lock = self.0.lock();
        lock.default.clone()
    }

    #[setter]
    fn set_default(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        let sql_type = lock.r#type.clone();

        let default = crate::expression::PyExpr::try_from_specific_type(val, Some(sql_type))?;
        lock.default = Some(default);
        Ok(())
    }

    /// Expression for generated column values.
    ///
    /// @signature (self) -> Expr | None
    /// @setter Expr | None
    #[getter]
    fn generated(&self) -> Option<PyExpr> {
        let lock = self.0.lock();
        lock.generated.clone()
    }

    #[setter]
    fn set_generated(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();

        let generated = crate::expression::PyExpr::try_from_specific_type(val, None)?;
        lock.generated = Some(generated);
        Ok(())
    }

    /// Shorthand for `Value(object, self.type)`.
    ///
    /// @signature (self, object: I) -> O
    fn adapt(&self, object: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<crate::value::PyValue> {
        let lock = self.0.lock();
        let sql_type = lock.r#type.clone();

        let result = crate::value::ValueState::from_pyobject(sql_type, object)?;
        Ok(result.into())
    }

    fn __copy__(&self) -> Self {
        let lock = self.0.lock();
        lock.clone().into()
    }

    pub fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s: Vec<u8> = Vec::with_capacity(20);

        write!(s, "<Column[{}] {:?}", lock.r#type, lock.name).unwrap();

        if lock.options & Self::OPT_PRIMARY_KEY > 0 {
            write!(s, " OPT_PRIMARY_KEY").unwrap();
        }
        if lock.options & Self::OPT_UNIQUE_KEY > 0 {
            write!(s, " OPT_UNIQUE_KEY").unwrap();
        }
        if lock.options & Self::OPT_AUTO_INCREMENT > 0 {
            write!(s, " OPT_AUTO_INCREMENT").unwrap();
        }
        if lock.options & Self::OPT_NULLABLE > 0 {
            write!(s, " OPT_NULLABLE").unwrap();
        }
        if lock.options & Self::OPT_STORED_GENERATED > 0 {
            write!(s, " OPT_STORED_GENERATED").unwrap();
        }
        if let Some(x) = &lock.extra {
            write!(s, " extra={x:?}").unwrap();
        }
        if let Some(x) = &lock.comment {
            write!(s, " comment={x:?}").unwrap();
        }
        if let Some(x) = &lock.default {
            write!(s, " default={}", x.__repr__()).unwrap();
        }
        if let Some(x) = &lock.generated {
            write!(s, " generated={}", x.__repr__()).unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
