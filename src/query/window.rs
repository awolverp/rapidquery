use pyo3::types::PyTupleMethods;
use sea_query::IntoColumnRef;
use sea_query::OverStatement;

use super::ordering::PyOrdering;
use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::internal::statements::ToSeaQuery;

/// Frame clause
#[derive(Debug, Clone, PartialEq)]
pub struct FrameClause {
    pub r#type: sea_query::FrameType,
    pub start: sea_query::Frame,
    pub end: Option<sea_query::Frame>,
}

crate::implement_pyclass! {
    /// Window frame start and frame end clause. Use its classmethods.
    #[derive(Debug, Clone)]
    [] PyFrame as "Frame" (pub sea_query::Frame);
}
crate::implement_pyclass! {
    /// Window expression.
    ///
    /// # References:
    ///
    /// 1. <https://dev.mysql.com/doc/refman/8.0/en/window-function-descriptions.html>
    /// 2. <https://www.sqlite.org/windowfunctions.html>
    /// 3. <https://www.postgresql.org/docs/current/tutorial-window.html>
    ///
    /// @signature (self, *partition_by: Expr | Column | ColumnRef | str)
    #[derive(Clone)]
    mutable [subclass] PyWindowStatement(WindowStatementState) as "WindowStatement" {
        pub partition_by: Vec<PyExpr>,
        pub orders: Vec<PyOrdering>,
        pub frame: Option<FrameClause>
    }
}

#[pyo3::pymethods]
impl PyFrame {
    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn unbounded_preceding(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::UnboundedPreceding)
    }

    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn current_row(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::CurrentRow)
    }

    /// @signature (cls) -> typing.Self
    #[classmethod]
    fn unbounded_following(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::UnboundedFollowing)
    }

    /// @signature (cls, val: int) -> typing.Self
    #[classmethod]
    fn following(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, val: u32) -> Self {
        Self(sea_query::Frame::Following(val))
    }

    /// @signature (cls, val: int) -> typing.Self
    #[classmethod]
    fn preceding(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, val: u32) -> Self {
        Self(sea_query::Frame::Preceding(val))
    }
}

impl ToSeaQuery<sea_query::WindowStatement> for WindowStatementState {
    fn to_sea_query<'a>(&self, _py: pyo3::Python<'a>) -> sea_query::WindowStatement {
        let mut stmt = sea_query::WindowStatement::new();

        for partition in &self.partition_by {
            stmt.add_partition_by(partition.0.clone());
        }

        if let Some(x) = &self.frame {
            stmt.frame(x.r#type.clone(), x.start.clone(), x.end.clone());
        }

        for order in self.orders.iter() {
            if let Some(x) = order.null_order {
                stmt.order_by_expr_with_nulls(order.target.0.clone(), order.order.clone(), x);
            } else {
                stmt.order_by_expr(order.target.0.clone(), order.order.clone());
            }
        }

        stmt
    }
}

#[pyo3::pymethods]
impl PyWindowStatement {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(*partition_by))]
    pub fn __init__(
        &self,
        partition_by: &pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<()> {
        let mut partitions = Vec::with_capacity(partition_by.len());

        for partition in partition_by.iter() {
            let partition = unsafe {
                if pyo3::ffi::Py_TYPE(partition.as_ptr()) == crate::typeref::EXPR_TYPE {
                    partition.cast_unchecked::<PyExpr>().get().clone()
                } else {
                    let column_ref = PyColumnRef::try_from(&partition)?;

                    PyExpr(sea_query::SimpleExpr::Column(column_ref.into_column_ref()))
                }
            };

            partitions.push(partition);
        }

        let state = WindowStatementState {
            partition_by: partitions,
            orders: Vec::new(),
            frame: None,
        };
        self.0.set(state);
        Ok(())
    }

    /// Partition by column or custom expression.
    ///
    /// @signature (self, partition_by: Expr | Column | ColumnRef | str) -> typing.Self
    fn partition<'a>(
        slf: pyo3::PyRef<'a, Self>,
        partition_by: &'a pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let partition_by = unsafe {
            if pyo3::ffi::Py_TYPE(partition_by.as_ptr()) == crate::typeref::EXPR_TYPE {
                partition_by.cast_unchecked::<PyExpr>().get().clone()
            } else {
                let column_ref = PyColumnRef::try_from(partition_by)?;

                PyExpr(sea_query::SimpleExpr::Column(column_ref.into_column_ref()))
            }
        };

        {
            let mut lock = slf.0.lock();
            lock.partition_by.push(partition_by);
        }
        Ok(slf)
    }

    /// @signature (self, clause: Ordering) -> typing.Self
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

    /// @signature (
    ///     self,
    ///     frame_type: typing.Literal["ROWS", "RANGE"],
    ///     frame_start: Frame,
    ///     frame_end: Frame | None = None,
    /// ) -> typing.Self
    #[pyo3(signature=(frame_type, frame_start, frame_end=None))]
    fn frame<'a>(
        slf: pyo3::PyRef<'a, Self>,
        frame_type: String,
        frame_start: pyo3::Bound<'_, PyFrame>,
        frame_end: Option<pyo3::Bound<'_, PyFrame>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let frame_type = match frame_type.to_ascii_lowercase().as_str() {
            "rows" => sea_query::FrameType::Rows,
            "range" => sea_query::FrameType::Range,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "invalid frame type, expected 'rows' or 'range'; got {:?}",
                    frame_type
                )));
            }
        };

        {
            let mut lock = slf.0.lock();
            lock.frame = Some(FrameClause {
                r#type: frame_type,
                start: frame_start.get().0.clone(),
                end: frame_end.map(|x| x.get().0.clone()),
            })
        }
        Ok(slf)
    }

    pub fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.0.lock();
        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<Window partition_by=[").unwrap();
        let n = lock.partition_by.len();
        for (index, expr) in lock.partition_by.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}]", expr.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", expr.__repr__()).unwrap();
            }
        }

        write!(s, " orders=[").unwrap();
        let n = lock.orders.len();
        for (index, expr) in lock.orders.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{}]", expr.__repr__()).unwrap();
            } else {
                write!(s, "{}, ", expr.__repr__()).unwrap();
            }
        }

        if let Some(x) = &lock.frame {
            write!(s, " frame_type={:?} frame_start={:?}", x.r#type, x.start).unwrap();

            if let Some(y) = &x.end {
                write!(s, " frame_end={:?}", y).unwrap();
            }
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
