use crate::internal::alias;
use crate::serializers::PySerializer;
use crate::serializers::SerializerExtractor;

use super::enum_::Value;

/// Python object kept until SQL is rendered. `prepare` binds it; `to_sql` serializes it.
#[rapidquery_macros::pyclass("rapidquery.expr.Value", skip_from_py_object, extends = super::base::PyExpr, generic)]
#[derive(Debug, Default)]
pub struct PyValue;

impl PyValue {
    #[inline(always)]
    fn inner<'a>(slf: &'a pyo3::PyRef<'a, PyValue>) -> &'a Value {
        debug_assert!(
            matches!(&slf.as_super().0, super::enum_::ExprInner::Value(..)),
            "PyValue instance always holds ExprInner::Value"
        );

        match &slf.as_super().0 {
            super::enum_::ExprInner::Value(v) => v,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

#[pyo3::pymethods]
impl PyValue {
    /// Wrap `object`. Omit `serializer` to infer one from its type.
    #[new]
    #[pyo3(signature = (object, serializer = None, /))]
    fn __new__(
        object: alias::BoundObject,
        serializer: Option<SerializerExtractor>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        // handle Value(Value(...), new_serializer)
        if unsafe { crate::typeref::check_exact(object.as_ptr(), crate::typeref::VALUE_TYPE) } {
            let expr = unsafe { object.cast_unchecked::<super::base::PyExpr>() };
            let expr_inner = expr.get().0.clone();

            debug_assert!(
                matches!(expr_inner, super::enum_::ExprInner::Value(..)),
                "PyValue instance always holds ExprInner::Value"
            );

            let inner = match expr_inner {
                super::enum_::ExprInner::Value(v) => v,
                _ => unsafe { std::hint::unreachable_unchecked() },
            };

            let value = match serializer {
                Some(SerializerExtractor(s)) => Value::new(inner.object, s),
                None => inner,
            };

            let new_expr = super::base::PyExpr(super::enum_::ExprInner::Value(value));
            return Ok(pyo3::PyClassInitializer::from(new_expr).add_subclass(PyValue));
        }

        let value = match serializer {
            Some(SerializerExtractor(s)) => Value::new(object.unbind(), s),
            None => Value::infer_bound(object)?,
        };

        let expr = super::base::PyExpr(super::enum_::ExprInner::Value(value));
        Ok(pyo3::PyClassInitializer::from(expr).add_subclass(PyValue))
    }

    /// The wrapped Python object.
    fn get_object(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        Self::inner(&slf).object.clone_ref(slf.py())
    }

    /// Serializer used by `to_sql`.
    fn get_serializer(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        PySerializer(Self::inner(&slf).serializer.clone()).into_subclasses(slf.py())
    }
}
