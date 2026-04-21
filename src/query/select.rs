use super::base::PyQueryStatement;
use super::ordering::PyOrdering;
use crate::common::column_ref::PyColumnRef;
use crate::common::expression::{PyExpr, PyFunc};
use crate::common::table_ref::PyTableName;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject, RefBoundObject, ToSeaQuery};
use crate::query::window::PyWindowStatement;

use pyo3::types::{PyAnyMethods, PyTupleMethods};
use sea_query::{IntoColumnRef, IntoIden};

/// Window type in [`PySelectLabel`]
pub enum SelectLabelWindow {
    Name(sea_query::DynIden),
    Query(
        /// Always is `PyWindowStatement`
        PyObject,
    ),
}

/// Select references in [`PySelectStatement`]
///
/// It exactly does what [`sea_query::TableRef`] does
pub enum SelectReference {
    SubQuery(
        /// Always is `PySelectStatement`
        PyObject,
        sea_query::DynIden,
    ),
    Func(PyFunc, sea_query::DynIden),
    TableName(PyTableName),
}

/// Distinct mode in [`PySelectStatement`]
#[derive(Debug, Clone, Default)]
pub enum DistinctMode {
    #[default]
    None,
    Distinct,
    DistinctOn(Vec<PyColumnRef>),
}

/// Lock mode and options in [`PySelectStatement`]
#[derive(Debug, Clone)]
pub struct LockMode {
    pub r#type: sea_query::LockType,
    pub behavior: Option<sea_query::LockBehavior>,
    pub tables: Vec<PyTableName>,
}

/// Join mode and options in [`PySelectStatement`]
pub struct JoinMode {
    pub r#type: sea_query::JoinType,
    pub reference: SelectReference,
    pub on: Option<PyExpr>,
    pub lateral: bool,
}

crate::implement_pyclass! {
    /// Represents a column expression with an optional alias in a SELECT statement.
    ///
    /// Used to specify both the expression to select and an optional alias name
    /// for the result column.
    immutable [subclass] PySelectLabel(SelectLabelState) as "SelectLabel" {
        pub expr: PyExpr,
        pub alias: Option<String>,
        pub window: Option<SelectLabelWindow>,
    }
}
crate::implement_pyclass! {
    /// Builds SELECT SQL statements with a fluent interface.
    ///
    /// Provides a chainable API for constructing SELECT queries with support for:
    /// - Column selection with expressions and aliases
    /// - Table and subquery sources
    /// - Filtering with WHERE and HAVING
    /// - Joins (inner, left, right, full, cross, lateral)
    /// - Grouping and aggregation
    /// - Ordering and pagination
    /// - Set operations (UNION, EXCEPT, INTERSECT)
    /// - Row locking for transactions
    /// - DISTINCT queries
    mutable [subclass, extends=PyQueryStatement] PySelectStatement(SelectStatementState) as "SelectStatement" {
        pub references: Vec<SelectReference>,

        /// Always is `Vec<PySelectLabel>`
        pub exprs: Vec<PyObject>,

        pub r#where: Option<PyExpr>,
        pub groups: Vec<PyExpr>,

        pub having: Option<PyExpr>,
        pub orders: Vec<PyOrdering>,
        pub distinct: DistinctMode,
        pub joins: Vec<JoinMode>,
        pub lock: Option<LockMode>,
        pub limit: Option<u64>,
        pub offset: Option<u64>,

        /// Always is `Option<(_, PyWindowStatement)>`
        pub window: Option<(sea_query::DynIden, PyObject)>,

        /// Always is `Option<(_, PySelectStatement)>`
        pub unions: Vec<(sea_query::UnionType, PyObject)>,

        // TODO
        // pub table_sample: Option<PyTableSample>,
        // pub index_hint: Option<PyIndexHint>,
    }
}

#[allow(clippy::derivable_impls)]
impl Default for SelectStatementState {
    fn default() -> Self {
        Self {
            references: Default::default(),
            exprs: Default::default(),
            r#where: Default::default(),
            groups: Default::default(),
            unions: Default::default(),
            having: Default::default(),
            orders: Default::default(),
            distinct: Default::default(),
            joins: Default::default(),
            lock: Default::default(),
            limit: Default::default(),
            offset: Default::default(),
            window: Default::default(),
        }
    }
}

impl SelectLabelState {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            expr: self.expr.clone(),
            alias: self.alias.clone(),
            window: self.window.as_ref().map(|x| match x {
                SelectLabelWindow::Name(x) => SelectLabelWindow::Name(x.clone()),
                SelectLabelWindow::Query(x) => SelectLabelWindow::Query(x.clone_ref(py)),
            }),
        }
    }
}

impl SelectReference {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        match self {
            Self::Func(x, y) => Self::Func(x.clone(), y.clone()),
            Self::TableName(x) => Self::TableName(x.clone()),
            Self::SubQuery(x, y) => Self::SubQuery(x.clone_ref(py), y.clone()),
        }
    }
}

impl JoinMode {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            r#type: self.r#type,
            reference: self.reference.clone_ref(py),
            on: self.on.clone(),
            lateral: self.lateral,
        }
    }
}

impl SelectStatementState {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            references: self.references.iter().map(|x| x.clone_ref(py)).collect(),
            exprs: self.exprs.iter().map(|x| x.clone_ref(py)).collect(),
            r#where: self.r#where.clone(),
            groups: self.groups.clone(),
            having: self.having.clone(),
            orders: self.orders.clone(),
            distinct: self.distinct.clone(),
            joins: self.joins.iter().map(|x| x.clone_ref(py)).collect(),
            lock: self.lock.clone(),
            limit: self.limit,
            offset: self.offset,
            window: self
                .window
                .as_ref()
                .map(|(x, y)| (x.clone(), y.clone_ref(py))),
            unions: self
                .unions
                .iter()
                .map(|(x, y)| (*x, y.clone_ref(py)))
                .collect(),
        }
    }
}

impl TryFrom<RefBoundObject<'_>> for SelectLabelWindow {
    type Error = pyo3::PyErr;

    fn try_from(value: RefBoundObject<'_>) -> Result<Self, Self::Error> {
        unsafe {
            // Window statement
            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::WINDOW_STATEMENT_TYPE)
                == 1
            {
                Ok(Self::Query(value.clone().unbind()))
            }
            // String
            else if pyo3::ffi::PyUnicode_CheckExact(value.as_ptr()) == 1 {
                let window_name = value.extract::<String>().unwrap_unchecked();
                Ok(Self::Name(sea_query::Alias::new(window_name).into_iden()))
            }
            // Other types
            else {
                crate::new_error!(
                    PyTypeError,
                    "expected WindowStatement or str for SelectLabel window, got {}",
                    crate::internal::get_type_name(value.py(), value.as_ptr())
                )
            }
        }
    }
}

#[inline]
fn cast_into_select_expr<'a>(value: BoundObject<'a>) -> pyo3::PyResult<BoundObject<'a>> {
    unsafe {
        // SelectLabel itself
        if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::SELECT_LABEL_TYPE) == 1 {
            return Ok(value);
        }

        let state = SelectLabelState {
            expr: PyExpr::try_from(&value)?,
            alias: None,
            window: None,
        };
        let result: PySelectLabel = state.into();
        pyo3::Bound::new(value.py(), result).map(|x| x.into_any())
    }
}

#[inline]
fn map_str_to_join_type(value: Option<String>) -> pyo3::PyResult<sea_query::JoinType> {
    match value.map(|x| x.to_ascii_uppercase()) {
        Some(x) => match x.as_str() {
            "CROSS" => Ok(sea_query::JoinType::CrossJoin),
            "FULL" => Ok(sea_query::JoinType::FullOuterJoin),
            "INNER" => Ok(sea_query::JoinType::InnerJoin),
            "LEFT" => Ok(sea_query::JoinType::LeftJoin),
            "RIGHT" => Ok(sea_query::JoinType::RightJoin),
            _ => {
                crate::new_error!(
                    PyValueError,
                    "acceptable join types are 'CROSS', 'FULL', 'INNER', 'RIGHT', and 'LEFT'. got \
                     {}",
                    x
                )
            }
        },
        None => Ok(sea_query::JoinType::Join),
    }
}

impl SelectReference {
    fn from_table(value: RefBoundObject<'_>) -> pyo3::PyResult<Self> {
        let table = PyTableName::try_from(value)?;
        Ok(Self::TableName(table))
    }

    fn from_subquery(value: RefBoundObject<'_>, alias: String) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::SELECT_STATEMENT_TYPE)
                == 0
            {
                return crate::new_error!(
                    PyValueError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(value.py(), value.as_ptr())
                );
            }
        }
        let alias = sea_query::Alias::new(alias).into_iden();

        Ok(Self::SubQuery(value.clone().unbind(), alias))
    }

    fn from_function(value: RefBoundObject<'_>, alias: String) -> pyo3::PyResult<Self> {
        let function = unsafe {
            let type_ptr = pyo3::ffi::Py_TYPE(value.as_ptr());

            // Func type
            if type_ptr == crate::typeref::FUNC_TYPE {
                value.cast_unchecked::<PyFunc>().get().clone()
            }
            // Expr type
            else if type_ptr == crate::typeref::EXPR_TYPE {
                let casted_expr = value.cast_unchecked::<PyExpr>();
                let get_casted_expr = casted_expr.get();

                if let sea_query::SimpleExpr::FunctionCall(x) = &get_casted_expr.0 {
                    PyFunc(x.clone())
                } else {
                    return crate::new_error!(PyValueError, "given Expr is not a function call");
                }
            }
            // Other types
            else {
                return crate::new_error!(
                    PyValueError,
                    "expected Func or Expr, got {}",
                    crate::internal::get_type_name(value.py(), value.as_ptr())
                );
            }
        };
        let alias = sea_query::Alias::new(alias).into_iden();

        Ok(Self::Func(function, alias))
    }
}

impl std::fmt::Display for SelectReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TableName(x) => write!(f, "{}", x.__repr__()),
            Self::Func(x, alias) => write!(f, "({}, {})", x.__repr__(), alias.to_string()),
            Self::SubQuery(x, alias) => write!(f, "({}, {})", x, alias.to_string()),
        }
    }
}

impl std::fmt::Display for JoinMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;

        match self.r#type {
            sea_query::JoinType::Join => (),
            sea_query::JoinType::CrossJoin => write!(f, "CROSS, ")?,
            sea_query::JoinType::FullOuterJoin => write!(f, "FULL, ")?,
            sea_query::JoinType::InnerJoin => write!(f, "INNER, ")?,
            sea_query::JoinType::LeftJoin => write!(f, "LEFT, ")?,
            sea_query::JoinType::RightJoin => write!(f, "RIGHT, ")?,
        }

        if let Some(on) = &self.on {
            write!(f, "{}, {}", self.reference, on.__repr__())?;
        } else {
            write!(f, "{}", self.reference)?;
        }

        if self.lateral {
            write!(f, ", LATERAL")?;
        }

        f.write_str(")")
    }
}

impl std::fmt::Display for LockMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        match self.r#type {
            sea_query::LockType::KeyShare => write!(f, "KEY SHARE")?,
            sea_query::LockType::NoKeyUpdate => write!(f, "NO KEY UPDATE")?,
            sea_query::LockType::Share => write!(f, "SHARE")?,
            sea_query::LockType::Update => write!(f, "UPDATE")?,
        }

        if let Some(x) = self.behavior {
            match x {
                sea_query::LockBehavior::Nowait => write!(f, ", NOWAIT")?,
                sea_query::LockBehavior::SkipLocked => write!(f, ", SKIP")?,
            }
        }

        if self.tables.is_empty() {
            return write!(f, ")");
        }

        write!(f, ", [")?;
        for (index, item) in self.tables.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", item.__repr__())?;
            } else {
                write!(f, ", {}", item.__repr__())?;
            }
        }
        write!(f, "])")
    }
}

impl ToSeaQuery<sea_query::WindowSelectType> for SelectLabelWindow {
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::WindowSelectType {
        match self {
            Self::Name(x) => sea_query::WindowSelectType::Name(x.clone()),
            SelectLabelWindow::Query(x) => {
                let statement = unsafe { x.cast_bound_unchecked::<PyWindowStatement>(py) };
                let lock = statement.get().0.lock();

                sea_query::WindowSelectType::Query(lock.to_sea_query(py))
            }
        }
    }
}

impl ToSeaQuery<sea_query::SelectExpr> for SelectLabelState {
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::SelectExpr {
        if self.alias.is_none() {
            // If expr is column, setting alias can optimize the SQL query.
            let default_alias = {
                match &self.expr.0 {
                    sea_query::SimpleExpr::Column(sea_query::ColumnRef::Column(name)) => {
                        Some(name.clone())
                    }
                    sea_query::SimpleExpr::Column(sea_query::ColumnRef::TableColumn(_, name)) => {
                        Some(name.clone())
                    }
                    sea_query::SimpleExpr::Column(sea_query::ColumnRef::SchemaTableColumn(
                        _,
                        _,
                        name,
                    )) => Some(name.clone()),
                    _ => None,
                }
            };

            sea_query::SelectExpr {
                expr: self.expr.0.clone(),
                alias: default_alias,
                window: self.window.as_ref().map(|x| x.to_sea_query(py)),
            }
        } else {
            sea_query::SelectExpr {
                expr: self.expr.0.clone(),
                alias: self
                    .alias
                    .as_ref()
                    .map(|x| sea_query::Alias::new(x).into_iden()),

                window: self.window.as_ref().map(|x| x.to_sea_query(py)),
            }
        }
    }
}

impl ToSeaQuery<sea_query::SelectStatement> for SelectStatementState {
    #[cfg_attr(feature = "optimize", optimize(speed))]
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> sea_query::SelectStatement {
        let mut stmt = sea_query::SelectStatement::new();

        // Distinct mode
        match &self.distinct {
            DistinctMode::None => (),
            DistinctMode::Distinct => {
                stmt.distinct();
            }
            DistinctMode::DistinctOn(cols) => {
                stmt.distinct_on(cols.clone());
            }
        }

        // References
        for table in self.references.iter() {
            match table {
                SelectReference::TableName(x) => unsafe {
                    stmt.from(x.clone());
                },
                SelectReference::Func(x, alias) => unsafe {
                    stmt.from_function(x.0.clone(), alias.clone());
                },
                SelectReference::SubQuery(x, alias) => unsafe {
                    let x = unsafe { x.cast_bound_unchecked::<PySelectStatement>(py) };
                    let lock = x.get().0.lock();

                    stmt.from_subquery(lock.to_sea_query(py), alias.clone());
                },
            }
        }

        // Columns
        stmt.exprs(self.exprs.iter().map(|x| unsafe {
            let expr = x.cast_bound_unchecked::<PySelectLabel>(py);
            expr.get().0.as_ref().to_sea_query(py)
        }));

        // Groups
        stmt.add_group_by(self.groups.iter().map(|x| x.0.clone()));

        // Condition
        stmt.and_where_option(self.r#where.as_ref().map(|x| x.0.clone()));

        // Having
        if let Some(x) = &self.having {
            stmt.and_having(x.0.clone());
        }

        // Limit & Offset
        if let Some(n) = self.limit {
            stmt.limit(n);
        }
        if let Some(n) = self.offset {
            stmt.offset(n);
        }

        // Orders
        for order in self.orders.iter() {
            if let Some(x) = order.null_order {
                stmt.order_by_expr_with_nulls(order.target.0.clone(), order.order.clone(), x);
            } else {
                stmt.order_by_expr(order.target.0.clone(), order.order.clone());
            }
        }

        // Lock mode
        if let Some(lock) = &self.lock {
            match (lock.behavior, lock.tables.is_empty()) {
                (Some(behavior), false) => {
                    stmt.lock_with_tables_behavior(
                        lock.r#type,
                        lock.tables.iter().cloned(),
                        behavior,
                    );
                }
                (Some(behavior), true) => {
                    stmt.lock_with_behavior(lock.r#type, behavior);
                }
                (None, false) => {
                    stmt.lock_with_tables(lock.r#type, lock.tables.iter().cloned());
                }
                (None, true) => {
                    stmt.lock(lock.r#type);
                }
            }
        }

        // Unions
        stmt.unions(self.unions.iter().map(|(union_type, union_stmt)| {
            let union_stmt = unsafe { union_stmt.cast_bound_unchecked::<PySelectStatement>(py) };
            let union_lock = union_stmt.get().0.lock();

            (*union_type, union_lock.to_sea_query(py))
        }));

        // Joins
        for join in self.joins.iter() {
            let condition =
                sea_query::Condition::all().add_option(join.on.as_ref().map(|x| x.0.clone()));

            match (&join.reference, join.lateral) {
                (SelectReference::TableName(x), _) => {
                    stmt.join(join.r#type, x.clone(), condition);
                }
                (SelectReference::Func(func, alias), _) => {
                    stmt.join(
                        join.r#type,
                        sea_query::TableRef::FunctionCall(func.0.clone(), alias.clone()),
                        condition,
                    );
                }
                (SelectReference::SubQuery(subquery, alias), true) => {
                    let subquery_stmt =
                        unsafe { subquery.cast_bound_unchecked::<PySelectStatement>(py) };
                    let subquery_lock = subquery_stmt.get().0.lock();

                    stmt.join_lateral(
                        join.r#type,
                        subquery_lock.to_sea_query(py),
                        alias.clone(),
                        condition,
                    );
                }
                (SelectReference::SubQuery(subquery, alias), false) => {
                    let subquery_stmt =
                        unsafe { subquery.cast_bound_unchecked::<PySelectStatement>(py) };
                    let subquery_lock = subquery_stmt.get().0.lock();

                    stmt.join_subquery(
                        join.r#type,
                        subquery_lock.to_sea_query(py),
                        alias.clone(),
                        condition,
                    );
                }
            }
        }

        // Window
        if let Some((window_name, window)) = &self.window {
            let window = unsafe { window.cast_bound_unchecked::<PyWindowStatement>(py) };
            let lock = window.get().0.lock();

            stmt.window(window_name.clone(), lock.to_sea_query(py));
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PySelectLabel {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(expr, alias=None, window=None))]
    fn __init__(
        &self,
        expr: RefBoundObject<'_>,
        alias: Option<String>,
        window: Option<RefBoundObject<'_>>,
    ) -> pyo3::PyResult<()> {
        let window = match window {
            Some(x) => Some(SelectLabelWindow::try_from(x)?),
            None => None,
        };
        let expr = PyExpr::try_from(expr)?;

        let state = SelectLabelState {
            expr,
            alias,
            window,
        };
        unsafe {
            self.0.set(state);
        }
        Ok(())
    }

    #[getter]
    fn expr(&self) -> PyExpr {
        self.0.as_ref().expr.clone()
    }

    #[getter]
    fn alias(&self) -> Option<String> {
        self.0.as_ref().alias.clone()
    }

    #[getter]
    fn window<'a>(&self, py: pyo3::Python<'a>) -> Option<BoundObject<'a>> {
        use pyo3::IntoPyObjectExt;

        let inner = self.0.as_ref();

        match &inner.window {
            Some(ref select_window) => match select_window {
                SelectLabelWindow::Name(name) => {
                    Some(name.to_string().into_bound_py_any(py).unwrap())
                }
                SelectLabelWindow::Query(w) => Some(w.bind(py).clone()),
            },
            None => None,
        }
    }

    fn __copy__(&self, py: pyo3::Python<'_>) -> Self {
        let lock = self.0.as_ref();
        lock.clone_ref(py).into()
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = slf.0.as_ref();

        let mut fmt = ReprFormatter::new_with_pyref(&slf)
            .map("expr", &inner.expr, |x| x.__repr__())
            .optional_quote("alias", inner.alias.as_ref())
            .take();

        match &inner.window {
            Some(SelectLabelWindow::Name(x)) => {
                fmt.iden("window", x);
            }
            Some(SelectLabelWindow::Query(x)) => {
                fmt.display("window", x);
            }
            None => {}
        }

        fmt.finish()
    }
}

#[pyo3::pymethods]
impl PySelectStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> (Self, PyQueryStatement) {
        (Self::uninit(), PyQueryStatement)
    }

    #[pyo3(signature=(*exprs))]
    pub fn __init__(&self, exprs: BoundArgs<'_>) -> pyo3::PyResult<()> {
        let mut casted = Vec::new();
        for item in exprs.iter() {
            casted.push(cast_into_select_expr(item)?.unbind());
        }

        let state = SelectStatementState {
            exprs: casted,
            ..Default::default()
        };
        self.0.set(state);
        Ok(())
    }

    #[pyo3(signature=(*on))]
    fn distinct<'a>(
        slf: pyo3::PyRef<'a, Self>,
        on: BoundArgs<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if on.is_empty() {
            // Distinct mode without specific column
            slf.0.lock().distinct = DistinctMode::Distinct;
        } else {
            let mut cols = Vec::new();

            for item in on.iter() {
                cols.push(PyColumnRef::try_from(&item)?);
            }

            slf.0.lock().distinct = DistinctMode::DistinctOn(cols);
        }

        Ok(slf)
    }

    #[pyo3(signature=(*args))]
    fn columns<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: BoundArgs<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut casted = Vec::new();
        for item in args.iter() {
            let column_ref = PyColumnRef::try_from(&item)?;

            let state = SelectLabelState {
                expr: PyExpr(sea_query::SimpleExpr::Column(column_ref.into_column_ref())),
                alias: None,
                window: None,
            };
            let result: PySelectLabel = state.into();

            casted.push(pyo3::Py::new(slf.py(), result)?.into_any());
        }

        slf.0.lock().exprs.append(&mut casted);
        Ok(slf)
    }

    #[pyo3(signature=(*args))]
    fn exprs<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: BoundArgs<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut casted = Vec::new();
        for item in args.iter() {
            casted.push(cast_into_select_expr(item)?.unbind());
        }

        slf.0.lock().exprs.append(&mut casted);
        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_table<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let reference = SelectReference::from_table(table)?;

        slf.0.lock().references.push(reference);
        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_subquery<'a>(
        slf: pyo3::PyRef<'a, Self>,
        subquery: RefBoundObject<'a>,
        alias: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if subquery.as_ptr() == slf.as_ptr() {
            return crate::new_error!(PyValueError, "select statement cannot select from itself!");
        }

        let reference = SelectReference::from_subquery(subquery, alias)?;

        slf.0.lock().references.push(reference);
        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_function<'a>(
        slf: pyo3::PyRef<'a, Self>,
        function: RefBoundObject<'a>,
        alias: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let reference = SelectReference::from_function(function, alias)?;

        slf.0.lock().references.push(reference);
        Ok(slf)
    }

    fn limit(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().limit = Some(n);
        slf
    }

    fn offset(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().offset = Some(n);
        slf
    }

    fn r#where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return crate::new_error!(
                    PyTypeError,
                    "expected Expr, got {}",
                    crate::internal::get_type_name(condition.py(), condition.as_ptr())
                );
            }

            let condition = condition.cast_unchecked::<PyExpr>().get().clone();
            let mut lock = slf.0.lock();

            match std::mem::take(&mut lock.r#where) {
                None => {
                    lock.r#where = Some(condition);
                }
                Some(x) => {
                    lock.r#where = Some(PyExpr(x.0.and(condition.0)));
                }
            }
        }

        Ok(slf)
    }

    /// Remove where conditions from statement.
    fn clear_where(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().r#where = None;
        slf
    }

    /// Specify the order in which to delete rows.
    #[pyo3(signature=(clause))]
    fn order_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        clause: pyo3::Bound<'_, PyOrdering>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let mut lock = slf.0.lock();
            lock.orders.push(clause.get().clone());
        }

        Ok(slf)
    }

    /// Remove orders from statement.
    fn clear_order_by(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf.0.lock().orders.clear();
        slf
    }

    fn having<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return crate::new_error!(
                    PyTypeError,
                    "expected Expr, got {}",
                    crate::internal::get_type_name(condition.py(), condition.as_ptr())
                );
            }

            let condition = condition.cast_unchecked::<PyExpr>().get().clone();
            let mut lock = slf.0.lock();

            match std::mem::take(&mut lock.having) {
                None => {
                    lock.having = Some(condition);
                }
                Some(x) => {
                    lock.having = Some(PyExpr(x.0.and(condition.0)));
                }
            }
        }

        Ok(slf)
    }

    #[pyo3(signature=(r#type=String::from("UPDATE"), behavior=None, tables=Vec::new()))]
    fn lock(
        slf: pyo3::PyRef<'_, Self>,
        r#type: String,
        behavior: Option<String>,
        tables: Vec<PyObject>,
    ) -> pyo3::PyResult<pyo3::PyRef<'_, Self>> {
        let lock_type = match r#type.to_ascii_uppercase().as_str() {
            "UPDATE" => sea_query::LockType::Update,
            "NO KEY UPDATE" => sea_query::LockType::NoKeyUpdate,
            "SHARE" => sea_query::LockType::Share,
            "KEY SHARE" => sea_query::LockType::KeyShare,
            _ => {
                return crate::new_error!(
                    PyValueError,
                    "acceptable lock types are 'UPDATE', 'NO KEY UPDATE', 'SHARE', and 'KEY \
                     SHARE'. got {}",
                    r#type
                )
            }
        };
        let lock_behavior = match behavior.map(|x| x.to_ascii_uppercase()) {
            Some(x) => match x.as_str() {
                "NOWAIT" => Some(sea_query::LockBehavior::Nowait),
                "SKIP" => Some(sea_query::LockBehavior::SkipLocked),
                _ => {
                    return crate::new_error!(
                        PyValueError,
                        "acceptable lock behaviors are 'NOWAIT', and 'SKIP'. got {}",
                        x
                    )
                }
            },
            None => None,
        };

        let mut tbs = Vec::with_capacity(tables.len());

        for table in tables.into_iter() {
            tbs.push(PyTableName::try_from(table.bind(slf.py()))?);
        }

        slf.0.lock().lock = Some(LockMode {
            r#type: lock_type,
            behavior: lock_behavior,
            tables: tbs,
        });
        Ok(slf)
    }

    #[pyo3(signature=(*groups))]
    fn group_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        groups: BoundArgs<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut exprs = Vec::with_capacity(groups.len());

        for expr in groups.iter() {
            let target = unsafe {
                if pyo3::ffi::Py_TYPE(expr.as_ptr()) == crate::typeref::EXPR_TYPE {
                    expr.cast_unchecked::<PyExpr>().get().clone()
                } else {
                    let column_ref = PyColumnRef::try_from(&expr)?;

                    PyExpr(sea_query::SimpleExpr::Column(column_ref.into_column_ref()))
                }
            };

            exprs.push(target);
        }

        slf.0.lock().groups.append(&mut exprs);
        Ok(slf)
    }

    #[pyo3(signature=(statement, r#type=String::from("DISTINCT")))]
    fn union<'a>(
        slf: pyo3::PyRef<'a, Self>,
        statement: BoundObject<'a>,
        r#type: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if statement.as_ptr() == slf.as_ptr() {
                return crate::new_error!(
                    PyValueError,
                    "select statement cannot select from itself!"
                );
            }

            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::SELECT_STATEMENT_TYPE,
            ) == 0
            {
                return crate::new_error!(
                    PyValueError,
                    "expected SelectStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }
        }

        let union_type = match r#type.to_ascii_uppercase().as_str() {
            "ALL" => sea_query::UnionType::All,
            "INTERSECT" => sea_query::UnionType::Intersect,
            "DISTINCT" => sea_query::UnionType::Distinct,
            "EXCEPT" => sea_query::UnionType::Except,
            _ => {
                return crate::new_error!(
                    PyValueError,
                    "acceptable union types are 'ALL', 'INTERSECT', 'DISTINCT', and 'EXCEPT'. got \
                     {}",
                    r#type
                )
            }
        };

        slf.0.lock().unions.push((union_type, statement.unbind()));
        Ok(slf)
    }

    #[pyo3(signature=(table, on=None, r#type=None))]
    fn join<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: RefBoundObject<'a>,
        on: Option<RefBoundObject<'a>>,
        r#type: Option<String>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let reference = SelectReference::from_table(table)?;
        let join_mode = JoinMode {
            r#type: map_str_to_join_type(r#type)?,
            reference,
            on: match on {
                Some(x) => Some(PyExpr::try_from(x)?),
                None => None,
            },
            lateral: false,
        };

        slf.0.lock().joins.push(join_mode);
        Ok(slf)
    }

    #[pyo3(signature=(function, alias, on=None, r#type=None))]
    fn join_function<'a>(
        slf: pyo3::PyRef<'a, Self>,
        function: RefBoundObject<'a>,
        alias: String,
        on: Option<RefBoundObject<'a>>,
        r#type: Option<String>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let reference = SelectReference::from_function(function, alias)?;

        let join_mode = JoinMode {
            r#type: map_str_to_join_type(r#type)?,
            reference,
            on: match on {
                Some(x) => Some(PyExpr::try_from(x)?),
                None => None,
            },
            lateral: false,
        };

        slf.0.lock().joins.push(join_mode);
        Ok(slf)
    }

    #[pyo3(signature=(subquery, alias, on=None, r#type=None, lateral=false))]
    fn join_subquery<'a>(
        slf: pyo3::PyRef<'a, Self>,
        subquery: RefBoundObject<'a>,
        alias: String,
        on: Option<RefBoundObject<'a>>,
        r#type: Option<String>,
        lateral: bool,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if subquery.as_ptr() == slf.as_ptr() {
            return crate::new_error!(PyValueError, "select statement cannot select from itself!");
        }

        let reference = SelectReference::from_subquery(subquery, alias)?;

        let join_mode = JoinMode {
            r#type: map_str_to_join_type(r#type)?,
            reference,
            on: match on {
                Some(x) => Some(PyExpr::try_from(x)?),
                None => None,
            },
            lateral,
        };
        slf.0.lock().joins.push(join_mode);
        Ok(slf)
    }

    #[pyo3(signature=(name, statement))]
    fn window<'a>(
        slf: pyo3::PyRef<'a, Self>,
        name: String,
        statement: RefBoundObject<'a>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(
                statement.as_ptr(),
                crate::typeref::WINDOW_STATEMENT_TYPE,
            ) == 0
            {
                return crate::new_error!(
                    PyTypeError,
                    "expected WindowStatement, got {}",
                    crate::internal::get_type_name(statement.py(), statement.as_ptr())
                );
            }
        }

        slf.0.lock().window = Some((
            sea_query::Alias::new(name).into_iden(),
            statement.clone().unbind(),
        ));
        Ok(slf)
    }

    /// Shorthand for `Expr(self)`
    fn to_expr(&self, py: pyo3::Python) -> PyExpr {
        let stmt = self.0.lock().to_sea_query(py);
        let subquery = sea_query::SubQueryStatement::SelectStatement(stmt);
        PyExpr(sea_query::SimpleExpr::SubQuery(None, Box::new(subquery)))
    }

    /// Shorthand for `SelectLabel(self, alias, window)`
    #[pyo3(signature=(alias, window=None))]
    fn label(
        &self,
        py: pyo3::Python,
        alias: String,
        window: Option<RefBoundObject<'_>>,
    ) -> pyo3::PyResult<crate::query::select::PySelectLabel> {
        let window = match window {
            Some(x) => Some(crate::query::select::SelectLabelWindow::try_from(x)?),
            None => None,
        };
        let expr = self.to_expr(py);

        let state = crate::query::select::SelectLabelState {
            expr,
            alias: Some(alias),
            window,
        };
        Ok(state.into())
    }

    #[pyo3(signature = (backend, /))]
    #[allow(clippy::wrong_self_convention)]
    fn to_sql(&self, py: pyo3::Python<'_>, backend: String) -> pyo3::PyResult<String> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_query_statement!(backend, stmt)
    }

    #[pyo3(signature = (backend, /))]
    fn build<'a>(
        &self,
        py: pyo3::Python<'a>,
        backend: String,
    ) -> pyo3::PyResult<(String, BoundObject<'a>)> {
        let lock = self.0.lock();
        let stmt = lock.to_sea_query(py);
        drop(lock);

        crate::build_query_parts!(py, backend, stmt)
    }

    fn __copy__<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<pyo3::Bound<'a, Self>> {
        let lock = self.0.lock();
        pyo3::Bound::new(py, (lock.clone_ref(py).into(), PyQueryStatement))
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf);

        match &lock.distinct {
            DistinctMode::None => (),
            DistinctMode::Distinct => {
                fmt.pair("distinct", "true");
            }
            DistinctMode::DistinctOn(x) => {
                fmt.vec("distinct", false)
                    .display_iter(x.iter().map(|x| x.__repr__()))
                    .finish(&mut fmt);
            }
        }

        fmt.vec("references", false)
            .display_iter(lock.references.iter())
            .finish(&mut fmt);

        fmt.vec("exprs", false)
            .display_iter(lock.exprs.iter())
            .finish(&mut fmt);

        fmt.optional_map("where", lock.r#where.as_ref(), |x| x.__repr__());

        fmt.vec("groups", true)
            .display_iter(lock.groups.iter().map(|x| x.__repr__()))
            .finish(&mut fmt);

        fmt.optional_map("having", lock.having.as_ref(), |x| x.__repr__());

        fmt.vec("orders", true)
            .display_iter(lock.orders.iter().map(|x| x.__repr__()))
            .finish(&mut fmt);

        fmt.vec("joins", true)
            .display_iter(lock.joins.iter())
            .finish(&mut fmt);

        fmt.optional_display("lock", lock.lock.as_ref())
            .optional_display("offset", lock.offset)
            .optional_display("limit", lock.limit)
            .optional_map("window", lock.window.as_ref(), |(name, stmt)| {
                format!("('{}', {})", name.to_string(), stmt)
            });

        fmt.vec("unions", true)
            .display_iter(
                lock.unions
                    .iter()
                    .map(|(union_type, stmt)| match union_type {
                        sea_query::UnionType::All => format!("(ALL, {})", stmt),
                        sea_query::UnionType::Distinct => format!("(DISTINCT, {})", stmt),
                        sea_query::UnionType::Except => format!("(EXCEPT, {})", stmt),
                        sea_query::UnionType::Intersect => format!("(INTERSECT, {})", stmt),
                    }),
            )
            .finish(&mut fmt);

        fmt.finish()
    }
}
