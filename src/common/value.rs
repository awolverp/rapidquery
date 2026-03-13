use crate::internal::repr::ReprFormatter;
use crate::internal::type_engine::TypeEngine;
use crate::internal::{BoundArgs, BoundKwargs, BoundObject, PyObject};
use crate::sqltypes::SQLTypeTrait;

crate::implement_pyclass! {
    /// Bridges Python types, Rust types, and SQL types for seamless data conversion.
    ///
    /// This class handles validation, adaptation, and conversion between different
    /// type systems used in the application stack.
    ///
    /// It can automatically detects the type of your value and selects appropriate Rust and SQL types.
    /// For example:
    /// - Python `int` becomes `BIGINT` SQL type (`BigIntegerType`)
    /// - Python `dict` or `list` becomes `JSON` SQL type (`JsonType`)
    /// - Python `float` becomes `DOUBLE` SQL type (`DoubleType`)
    ///
    /// However, for more accurate type selection, it's recommended to use the `sql_type` parameter.
    ///
    /// NOTE: this class is immutable and frozen.
    ///
    /// @extends typing.Generic[T]
    /// @signature (self, value: T | None, sql_type: SQLTypeAbstract[T] | None = ...)
    mutable [subclass] PyValue(ValueState) as "Value" {
        sql_type: TypeEngine,
        serialized: Option<sea_query::Value>,
        deserialized: Option<PyObject>,
    }
}

impl ValueState {
    #[inline(always)]
    pub unsafe fn new_unchecked(
        sql_type: TypeEngine,
        serialized: Option<sea_query::Value>,
        deserialized: Option<PyObject>,
    ) -> Self {
        Self {
            sql_type,
            serialized,
            deserialized,
        }
    }

    pub fn from_pyobject(sql_type: TypeEngine, object: BoundObject<'_>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_IsNone(object.as_ptr()) == 0 {
                sql_type.validate(object.py(), object.as_ptr())?;
            } else {
                // There's no need to do validation.
                // The `ValueState` will handle `NoneType`s itself
            }

            Ok(Self::new_unchecked(sql_type, None, Some(object.unbind())))
        }
    }

    pub fn from_sea_query_value(py: pyo3::Python, value: sea_query::Value) -> Self {
        let sql_type = TypeEngine::infer_value(py, &value);

        unsafe { Self::new_unchecked(sql_type, Some(value), None) }
    }

    pub fn simple_expr(&mut self, py: pyo3::Python) -> pyo3::PyResult<sea_query::SimpleExpr> {
        if let Some(x) = &self.serialized {
            return Ok(sea_query::SimpleExpr::Value(x.clone()));
        }

        let result = unsafe {
            let deserialized = self.deserialized.as_ref().unwrap_unchecked();

            if pyo3::ffi::Py_IsNone(deserialized.as_ptr()) == 1 {
                sea_query::Value::Bool(None)
            } else {
                self.sql_type.serialize(py, deserialized.as_ptr())?
            }
        };

        self.serialized = Some(result.clone());
        Ok(sea_query::SimpleExpr::Value(result))
    }

    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            sql_type: self.sql_type.clone(),
            serialized: self.serialized.clone(),
            deserialized: self.deserialized.as_ref().map(|x| x.clone_ref(py)),
        }
    }
}

#[pyo3::pymethods]
impl PyValue {
    #[new]
    #[allow(unused_variables)]
    #[pyo3(signature=(*args, **kwds))]
    fn __new__(args: BoundArgs<'_>, kwds: Option<BoundKwargs<'_>>) -> Self {
        Self::uninit()
    }

    #[pyo3(signature=(value, sql_type=None))]
    fn __init__(
        &self,
        value: BoundObject<'_>,
        sql_type: Option<BoundObject<'_>>,
    ) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::PyObject_TypeCheck(value.as_ptr(), crate::typeref::VALUE_TYPE) == 1 {
                let py = value.py();

                let casted_value = value.cast_into_unchecked::<Self>();
                let state = casted_value.get().0.lock().clone_ref(py);

                self.0.set(state);
                return Ok(());
            }
        }

        let type_engine = {
            if let Some(sql_type) = sql_type {
                TypeEngine::new(&sql_type)?
            } else {
                TypeEngine::infer_pyobject(&value)?
            }
        };

        let result = ValueState::from_pyobject(type_engine, value)?;
        self.0.set(result);
        Ok(())
    }

    /// @signature (self) -> SQLTypeAbstract[T]
    #[getter]
    fn sql_type<'a>(&self, py: pyo3::Python<'a>) -> BoundObject<'a> {
        let lock = self.0.lock();
        lock.sql_type.as_pyobject(py)
    }

    /// Converts the adapted value back to a Python type.
    ///
    /// @signature (self) -> T | None
    #[getter]
    fn value<'py>(&self, py: pyo3::Python<'py>) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::PyAny>> {
        let mut lock = self.0.lock();

        if let Some(x) = &lock.deserialized {
            return Ok(x.bind(py).clone());
        }

        let deserialized = unsafe {
            lock.sql_type
                .deserialize(py, lock.serialized.as_ref().unwrap())
                .map(|x| pyo3::Bound::from_owned_ptr(py, x))?
        };
        lock.deserialized = Some(deserialized.clone().unbind());

        Ok(deserialized)
    }

    fn __hash__(&self, py: pyo3::Python) -> pyo3::PyResult<isize> {
        self.value(py)
            .map(|x| unsafe { pyo3::ffi::PyObject_Hash(x.as_ptr()) })
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<String> {
        let value = slf.value(slf.py())?;

        Ok(ReprFormatter::new_with_pyref(&slf)
            .debug("", &value)
            .finish())
    }
}
