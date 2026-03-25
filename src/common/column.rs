use sea_query::IntoIden;

use crate::common::expression::PyExpr;
use crate::internal::parameters::OptionalParam;
use crate::internal::repr::ReprFormatter;
use crate::internal::type_engine::TypeEngine;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject, RefBoundObject, ToSeaQuery};
use crate::sqltypes::SQLTypeTrait;

pub const OPT_PRIMARY_KEY: u8 = 1 << 0;
pub const OPT_UNIQUE_KEY: u8 = 1 << 1;
pub const OPT_NOT_NULL: u8 = 1 << 2;
pub const OPT_NULL: u8 = 1 << 3;
pub const OPT_AUTO_INCREMENT: u8 = 1 << 4;
pub const OPT_STORED_GENERATED: u8 = 1 << 5;

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
        if self.options & OPT_NULL > 0 {
            column_def.null();
        }
        if self.options & OPT_NOT_NULL > 0 {
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
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
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
                nullable=None,
                auto_increment=false,
                extra=None,
                comment=None,
                default=OptionalParam::Undefined,
                generated=OptionalParam::Undefined,
                stored_generated=false,
            )
        )
    ]
    fn __init__(
        &self,
        name: String,
        r#type: RefBoundObject<'_>,
        primary_key: bool,
        unique_key: bool,
        nullable: Option<bool>,
        auto_increment: bool,
        extra: Option<String>,
        comment: Option<String>,
        default: OptionalParam,
        generated: OptionalParam,
        stored_generated: bool,
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

        match nullable {
            Some(true) => options |= OPT_NULL,
            Some(false) => options |= OPT_NOT_NULL,
            None => (),
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
    #[getter]
    fn name(&self) -> String {
        self.0.lock().name.clone()
    }

    #[setter]
    fn set_name(&self, value: String) {
        self.0.lock().name = value;
    }

    /// Column type.
    #[getter]
    fn r#type(&self, py: pyo3::Python) -> PyObject {
        let lock = self.0.lock();
        lock.r#type.as_pyobject(py).unbind()
    }

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

    #[getter]
    fn nullable(&self) -> Option<bool> {
        let options = self.0.lock().options;

        if options & OPT_NOT_NULL > 0 {
            Some(false)
        } else if options & OPT_NULL > 0 {
            Some(true)
        } else {
            None
        }
    }

    #[setter]
    fn set_nullable(&self, value: Option<bool>) {
        let mut lock = self.0.lock();

        match value {
            Some(true) => {
                lock.options |= OPT_NULL;
                lock.options &= !OPT_NOT_NULL;
            }
            Some(false) => {
                lock.options |= OPT_NOT_NULL;
                lock.options &= !OPT_NULL;
            }
            None => {
                lock.options &= !OPT_NOT_NULL;
                lock.options &= !OPT_NULL;
            }
        }
    }

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
    #[getter]
    fn default(&self) -> Option<PyExpr> {
        let lock = self.0.lock();
        lock.default.clone()
    }

    #[setter]
    fn set_default(&self, val: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();
        let sql_type = lock.r#type.clone();

        let default = super::expression::PyExpr::try_from_specific_type(val, Some(sql_type))?;
        lock.default = Some(default);
        Ok(())
    }

    /// Expression for generated column values.
    #[getter]
    fn generated(&self) -> Option<PyExpr> {
        let lock = self.0.lock();
        lock.generated.clone()
    }

    #[setter]
    fn set_generated(&self, val: RefBoundObject<'_>) -> pyo3::PyResult<()> {
        let mut lock = self.0.lock();

        let generated = super::expression::PyExpr::try_from_specific_type(val, None)?;
        lock.generated = Some(generated);
        Ok(())
    }

    /// Shorthand for `Value(object, self.type)`.
    fn adapt(&self, object: BoundObject<'_>) -> pyo3::PyResult<super::value::PyValue> {
        let lock = self.0.lock();
        let sql_type = lock.r#type.clone();

        let result = super::value::ValueState::from_pyobject(sql_type, object)?;
        Ok(result.into())
    }

    /// Shorthand for `Expr(self)`
    fn to_expr(&self) -> PyExpr {
        let col_ref = Self::__column_ref__(self);
        PyExpr(sea_query::Expr::column(col_ref))
    }

    #[getter]
    fn __column_ref__(&self) -> super::column_ref::PyColumnRef {
        let lock = self.0.lock();

        super::column_ref::PyColumnRef {
            name: Some(sea_query::Alias::new(&lock.name).into_iden()),
            table: None,
            schema: None,
        }
    }

    fn __copy__(&self) -> Self {
        let lock = self.0.lock();
        lock.clone().into()
    }

    pub fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        ReprFormatter::new_with_pyref(&slf)
            .quote("", &lock.name)
            .display("type", &lock.r#type)
            .optional_boolean("primary_key", lock.options & OPT_PRIMARY_KEY > 0)
            .optional_boolean("unique_key", lock.options & OPT_UNIQUE_KEY > 0)
            .optional_boolean("auto_increment", lock.options & OPT_AUTO_INCREMENT > 0)
            .optional_display(
                "nullable",
                if lock.options & OPT_NOT_NULL > 0 {
                    Some(false)
                } else if lock.options & OPT_NULL > 0 {
                    Some(true)
                } else {
                    None
                },
            )
            .optional_boolean("stored_generated", lock.options & OPT_STORED_GENERATED > 0)
            .optional_quote("extra", lock.extra.as_ref())
            .optional_quote("comment", lock.comment.as_ref())
            .optional_map("default", lock.default.as_ref(), |x| x.__repr__())
            .optional_map("generated", lock.generated.as_ref(), |x| x.__repr__())
            .finish()
    }
}
