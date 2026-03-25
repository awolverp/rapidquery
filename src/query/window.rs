use pyo3::types::PyTupleMethods;
use sea_query::{IntoColumnRef, OverStatement};

use super::ordering::PyOrdering;
use crate::common::column_ref::PyColumnRef;
use crate::common::expression::PyExpr;
use crate::internal::repr::ReprFormatter;
use crate::internal::{BoundArgs, BoundKwargs, RefBoundObject, ToSeaQuery};

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
    #[derive(Clone)]
    mutable [subclass] PyWindowStatement(WindowStatementState) as "WindowStatement" {
        pub partition_by: Vec<PyExpr>,
        pub orders: Vec<PyOrdering>,
        pub frame: Option<FrameClause>
    }
}

#[pyo3::pymethods]
impl PyFrame {
    #[classmethod]
    fn unbounded_preceding(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::UnboundedPreceding)
    }

    #[classmethod]
    fn current_row(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::CurrentRow)
    }

    #[classmethod]
    fn unbounded_following(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::UnboundedFollowing)
    }

    #[classmethod]
    fn following(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, val: u32) -> Self {
        Self(sea_query::Frame::Following(val))
    }

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
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(*partition_by))]
    pub fn __init__(&self, partition_by: BoundArgs<'_>) -> pyo3::PyResult<()> {
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
    fn partition<'a>(
        slf: pyo3::PyRef<'a, Self>,
        partition_by: RefBoundObject<'a>,
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

    fn __copy__<'a>(&self) -> Self {
        let lock = self.0.lock();
        lock.clone().into()
    }

    pub fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let lock = slf.0.lock();

        let mut fmt = ReprFormatter::new_with_pyref(&slf);

        fmt.vec("partition_by", true)
            .display_iter(lock.partition_by.iter().map(|x| x.__repr__()))
            .finish(&mut fmt);

        fmt.vec("orders", true)
            .display_iter(lock.orders.iter().map(|x| x.__repr__()))
            .finish(&mut fmt);

        fmt.optional_map("frame_type", lock.frame.as_ref(), |x| {
            format!("{:?}", x.r#type)
        })
        .optional_map("frame_start", lock.frame.as_ref(), |x| {
            format!("{:?}", x.start)
        })
        .optional_map("frame_end", lock.frame.as_ref(), |x| format!("{:?}", x.end))
        .finish()
    }
}
