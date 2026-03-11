use sea_query::IntoIden;

use crate::common::expression::PyExpr;
use crate::internal::parameters::OptionalParam;
use crate::internal::statements::ToSeaQuery;
use crate::internal::type_engine::TypeEngine;
use crate::sqltypes::SQLTypeTrait;

pub const OPT_PRIMARY_KEY: u8 = 1 << 0;
pub const OPT_UNIQUE_KEY: u8 = 1 << 1;
pub const OPT_NULLABLE: u8 = 1 << 2;
pub const OPT_AUTO_INCREMENT: u8 = 1 << 3;
pub const OPT_STORED_GENERATED: u8 = 1 << 4;

crate::implement_pyclass! {
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
    /// @extends typing.Generic[T]
    /// @signature (
    ///     self,
    ///     name: str,
    ///     type: SQLTypeAbstract[T],
    ///     *,
    ///     primary_key: bool = False,
    ///     unique_key: bool = False,
    ///     nullable: bool = False,
    ///     auto_increment: bool = False,
    ///     stored_generated: bool = False,
    ///     extra: str | None = ...,
    ///     comment: str | None = ...,
    ///     default: typing.Any = ...,
    ///     generated: typing.Any = ...,
    /// )
    #[derive(Debug, Clone)]
    mutable [subclass, generic] PyColumn(ColumnState) as "Column" {
        pub name: String,
        pub r#type: TypeEngine,
        pub options: u8,
        pub default: Option<PyExpr>,
        pub generated: Option<PyExpr>,
        pub extra: Option<String>,
        pub comment: Option<String>,
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

        if self.options & OPT_PRIMARY_KEY > 0 {
            column_def.primary_key();
        }
        if self.options & OPT_AUTO_INCREMENT > 0 {
            column_def.auto_increment();
        }
        if self.options & OPT_NULLABLE > 0 {
            column_def.null();
        } else {
            column_def.not_null();
        }
        if self.options & OPT_UNIQUE_KEY > 0 {
            column_def.unique_key();
        }

        if let Some(x) = &self.default {
            column_def.default(x.0.clone());
        }
        if let Some(x) = &self.generated {
            column_def.generated(x.0.clone(), self.options & OPT_STORED_GENERATED > 0);
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
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> Self {
        Self::uninit()
    }

    #[
        pyo3(
            signature=(
                name,
                r#type,
                *,
                primary_key=false,
                unique_key=false,
                nullable=false,
                auto_increment=false,
                stored_generated=false,
                extra=None,
                comment=None,
                default=OptionalParam::Undefined,
                generated=OptionalParam::Undefined,
            )
        )
    ]
    fn __init__(
        &self,
        name: String,
        r#type: &pyo3::Bound<'_, pyo3::PyAny>,
        primary_key: bool,
        unique_key: bool,
        nullable: bool,
        auto_increment: bool,
        stored_generated: bool,
        extra: Option<String>,
        comment: Option<String>,
        default: OptionalParam,
        generated: OptionalParam,
    ) -> pyo3::PyResult<()> {
        let sql_type = TypeEngine::new(r#type)?;

        let default_expr = match default {
            OptionalParam::Defined(x) => Some(super::expression::PyExpr::try_from_specific_type(
                &x,
                Some(sql_type.clone()),
            )?),
            OptionalParam::Undefined => None,
        };
        let generated_expr = match generated {
            OptionalParam::Defined(x) => Some(super::expression::PyExpr::try_from(&x)?),
            OptionalParam::Undefined => None,
        };

        let mut options = 0u8;
        if primary_key {
            options |= OPT_PRIMARY_KEY;
        }
        if unique_key {
            options |= OPT_UNIQUE_KEY;
        }
        if nullable {
            options |= OPT_NULLABLE;
        }
        if auto_increment {
            options |= OPT_AUTO_INCREMENT;
        }
        if stored_generated {
            options |= OPT_STORED_GENERATED;
        }

        let state = ColumnState {
            name,
            r#type: sql_type,
            options,
            default: default_expr,
            generated: generated_expr,
            extra,
            comment,
        };
        self.0.set(state);

        Ok(())
    }

    /// Column name.
    ///
    /// @signature (self) -> str
    /// @setter str
    #[getter]
    fn name(&self) -> String {
        self.0.lock().name.clone()
    }

    #[setter]
    fn set_name(&self, value: String) {
        self.0.lock().name = value;
    }

    /// Column type.
    ///
    /// @signature (self) -> SQLTypeAbstract[T]
    #[getter]
    fn r#type(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.0.lock();
        lock.r#type.as_pyobject(py).unbind()
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn primary_key(&self) -> bool {
        self.0.lock().options & OPT_PRIMARY_KEY > 0
    }

    #[setter]
    fn set_primary_key(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_PRIMARY_KEY;
        } else {
            lock.options &= !OPT_PRIMARY_KEY;
        }
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn unique_key(&self) -> bool {
        self.0.lock().options & OPT_UNIQUE_KEY > 0
    }

    #[setter]
    fn set_unique_key(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_UNIQUE_KEY;
        } else {
            lock.options &= !OPT_UNIQUE_KEY;
        }
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn auto_increment(&self) -> bool {
        self.0.lock().options & OPT_AUTO_INCREMENT > 0
    }

    #[setter]
    fn set_auto_increment(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_AUTO_INCREMENT;
        } else {
            lock.options &= !OPT_AUTO_INCREMENT;
        }
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn nullable(&self) -> bool {
        self.0.lock().options & OPT_NULLABLE > 0
    }

    #[setter]
    fn set_nullable(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_NULLABLE;
        } else {
            lock.options &= !OPT_NULLABLE;
        }
    }

    /// @signature (self) -> bool
    /// @setter bool
    #[getter]
    fn stored_generated(&self) -> bool {
        self.0.lock().options & OPT_STORED_GENERATED > 0
    }

    #[setter]
    fn set_stored_generated(&self, value: bool) {
        let mut lock = self.0.lock();
        if value {
            lock.options |= OPT_STORED_GENERATED;
        } else {
            lock.options &= !OPT_STORED_GENERATED;
        }
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

        let default = super::expression::PyExpr::try_from_specific_type(val, Some(sql_type))?;
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

        let generated = super::expression::PyExpr::try_from_specific_type(val, None)?;
        lock.generated = Some(generated);
        Ok(())
    }

    /// Shorthand for `Value(object, self.type)`.
    ///
    /// @signature (self, object: T) -> Value[T]
    fn adapt(&self, object: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<super::value::PyValue> {
        let lock = self.0.lock();
        let sql_type = lock.r#type.clone();

        let result = super::value::ValueState::from_pyobject(sql_type, object)?;
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

        write!(s, "<Column {:?} {}", lock.name, lock.r#type,).unwrap();

        if lock.options & OPT_PRIMARY_KEY > 0 {
            write!(s, " primary_key=True").unwrap();
        }
        if lock.options & OPT_UNIQUE_KEY > 0 {
            write!(s, " unique_key=True").unwrap();
        }
        if lock.options & OPT_AUTO_INCREMENT > 0 {
            write!(s, " auto_increment=True").unwrap();
        }
        if lock.options & OPT_NULLABLE > 0 {
            write!(s, " nullable=True").unwrap();
        }
        if lock.options & OPT_STORED_GENERATED > 0 {
            write!(s, " stored_generated=True").unwrap();
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
