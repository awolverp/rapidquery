use crate::{
    data_type::{PySerializer, SerializerExtractor},
    internal::alias,
};

use super::enum_::Value;

/// A Python object plus the [`Serializer`] used to emit it as SQL.
///
/// `Value` does not convert the object until SQL is built. Parameterized
/// statements bind the object as a placeholder. Raw SQL runs the serializer
/// (`None` becomes `NULL`).
///
/// If `serializer` is omitted, one is inferred from the object's type.
/// Passing an existing `Value` keeps its object (and its serializer, unless
/// a new one is given). A `DataType` as `serializer` contributes only its
/// serializer; the data type is not stored.
#[rapidquery_macros::pyclass(
    "Value",
    skip_from_py_object,
    extends = super::base::PyExpr,
    module = "rapidquery._core.expr",
    generic
)]
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
    /// Creates a SQL value.
    ///
    /// Args:
    ///     object: The Python object to bind or serialize. If this is already
    ///             a `Value`, its object is reused.
    ///     serializer: How to render `object` as a SQL literal. A `DataType`
    ///             is accepted and only its serializer is kept. If omitted,
    ///             a serializer is inferred from `object` (or kept, when
    ///             `object` is already a `Value`).
    ///
    /// Raises:
    ///     TypeError: If `serializer` is not a `Serializer` or `DataType`.
    #[new]
    #[pyo3(signature = (object, serializer = None, /))]
    fn __new__(
        object: alias::BoundObject,
        serializer: Option<SerializerExtractor>,
    ) -> pyo3::PyClassInitializer<Self> {
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
            return pyo3::PyClassInitializer::from(new_expr).add_subclass(PyValue);
        }

        let value = match serializer {
            Some(SerializerExtractor(s)) => Value::new(object.unbind(), s),
            None => Value::infer_bound(object),
        };

        let expr = super::base::PyExpr(super::enum_::ExprInner::Value(value));
        pyo3::PyClassInitializer::from(expr).add_subclass(PyValue)
    }

    /// Returns the wrapped Python object.
    ///
    /// Returns:
    ///     The object passed to the constructor (or taken from an existing `Value`).
    fn get_object(slf: pyo3::PyRef<'_, Self>) -> alias::PyObject {
        Self::inner(&slf).object.clone_ref(slf.py())
    }

    /// Returns the serializer used for raw SQL literals.
    ///
    /// Returns:
    ///     The `Serializer` stored on this value. Not used when the value
    ///     is bound as a parameter.
    fn get_serializer(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<alias::BoundObject<'_>> {
        PySerializer(Self::inner(&slf).serializer.clone()).into_subclasses(slf.py())
    }

    fn __hash__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<u64> {
        use pyo3::types::PyAnyMethods;
        use std::hash::{DefaultHasher, Hash, Hasher};

        let inner = Self::inner(&slf);
        let hash_object = inner.object.bind(slf.py()).hash()?;

        let mut state = DefaultHasher::default();
        state.write_isize(hash_object);
        inner.serializer.hash(&mut state);
        Ok(state.finish())
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return true;
        }

        Self::inner(&slf).eq(Self::inner(&other))
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return true;
        }

        Self::inner(&slf).ne(Self::inner(&other))
    }
}
