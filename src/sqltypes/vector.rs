use pyo3::types::{PyListMethods, PyTupleMethods};
use pyo3::IntoPyObject;

use crate::internal::type_engine::TypeEngine;
use crate::internal::RefBoundObject;
use crate::sqltypes::abstracts::{PySQLTypeAbstract, SQLTypeTrait};

crate::implement_pyclass! {
    /// Vector column type for storing mathematical vectors.
    ///
    /// Specialized type for storing vector data, often used in machine learning,
    /// similarity search, or mathematical applications.
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyVectorType as "Vector" (pub Option<u32>);
}
crate::implement_pyclass! {
    /// Array column type for storing arrays of elements.
    ///
    /// Represents a column that stores arrays of a specified element type.
    /// Useful in databases that support native array types (like PostgreSQL)
    /// for storing lists of values in a single column.
    #[derive(Clone)]
    [extends=PySQLTypeAbstract] PyArrayType as "Array" (pub TypeEngine);
}

#[inline]
#[cfg_attr(feature = "optimize", optimize(speed))]
unsafe fn _validate_iterable_ptr<F>(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
    sql_type_name: &str,
    mut condition: F,
) -> pyo3::PyResult<()>
where
    F: FnMut(*mut pyo3::ffi::PyObject) -> bool,
{
    // We won't check all of elements, we only check some of them to improve performance `O(log(n))`.

    if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
        let mut size = pyo3::ffi::PyList_Size(ptr);
        if size == 0 {
            return Ok(());
        }

        while size > 0 {
            // item is a owned pointer
            let item = pyo3::ffi::PyList_GetItemRef(ptr, size - 1);
            if item.is_null() {
                return Err(pyo3::PyErr::fetch(py));
            }

            if !condition(item) {
                pyo3::ffi::Py_DECREF(item);
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "invalid type found in the list",
                ));
            }

            pyo3::ffi::Py_DECREF(item);
            size /= 2;
        }

        return Ok(());
    }

    if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
        let mut size = pyo3::ffi::PyTuple_Size(ptr);
        if size == 0 {
            return Ok(());
        }

        while size > 0 {
            // item is a borrowed pointer
            let item = pyo3::ffi::PyTuple_GetItem(ptr, 0);
            if item.is_null() {
                return Err(pyo3::PyErr::fetch(py));
            }

            if !condition(item) {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "invalid type found in the tuple",
                ));
            }

            size /= 2;
        }

        return Ok(());
    }

    crate::new_error!(
        PyTypeError,
        "expected list/tuple for {} serialization, got {}",
        sql_type_name,
        crate::internal::get_type_name(py, ptr)
    )
}

impl SQLTypeTrait for PyVectorType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Vector(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        _validate_iterable_ptr(py, ptr, &self.to_sql_type_name(), |item| {
            pyo3::ffi::PyFloat_CheckExact(item) == 1 || pyo3::ffi::PyLong_CheckExact(item) == 1
        })
    }

    #[cfg_attr(feature = "optimize", optimize(speed))]
    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
            let bound = pyo3::Bound::from_borrowed_ptr(py, ptr);
            let list = bound.cast_unchecked::<pyo3::types::PyList>();

            let mut values: Vec<f32> = Vec::with_capacity(list.len());

            for item in list.iter() {
                if pyo3::ffi::PyFloat_CheckExact(item.as_ptr()) == 0
                    && pyo3::ffi::PyLong_CheckExact(item.as_ptr()) == 0
                {
                    return crate::new_error!(
                        PyTypeError,
                        "expected list of floats for {} serialization, found {} in the list",
                        self.to_sql_type_name(),
                        crate::internal::get_type_name(py, item.as_ptr())
                    );
                }

                values.push(pyo3::ffi::PyFloat_AsDouble(item.as_ptr()) as f32);
            }

            return Ok(sea_query::Value::Vector(Some(Box::new(values.into()))));
        }

        if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            let bound = pyo3::Bound::from_borrowed_ptr(py, ptr);
            let list = bound.cast_unchecked::<pyo3::types::PyTuple>();

            let mut values: Vec<f32> = Vec::with_capacity(list.len());

            for item in list.iter() {
                if pyo3::ffi::PyFloat_CheckExact(item.as_ptr()) == 0
                    && pyo3::ffi::PyLong_CheckExact(item.as_ptr()) == 0
                {
                    return crate::new_error!(
                        PyTypeError,
                        "expected tuple of floats for {} serialization, found {} in the tuple",
                        self.to_sql_type_name(),
                        crate::internal::get_type_name(py, item.as_ptr())
                    );
                }

                values.push(pyo3::ffi::PyFloat_AsDouble(item.as_ptr()) as f32);
            }

            return Ok(sea_query::Value::Vector(Some(Box::new(values.into()))));
        }

        crate::new_error!(
            PyTypeError,
            "expected list[float]/tuple[float] for {} serialization, got {}",
            self.to_sql_type_name(),
            crate::internal::get_type_name(py, ptr)
        )
    }

    #[cfg_attr(feature = "optimize", optimize(speed))]
    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Vector(Some(x)) => {
                let list = x.as_slice().into_pyobject(py)?;
                Ok(list.into_ptr())
            }
            sea_query::Value::Vector(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected vector for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyArrayType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Array(std::sync::Arc::new(self.0.to_sea_query_column_type()))
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        _validate_iterable_ptr(py, ptr, &self.to_sql_type_name(), |item| {
            self.0.validate(py, item).is_ok()
        })
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        if pyo3::ffi::PyList_CheckExact(ptr) == 1 {
            let bound = pyo3::Bound::from_borrowed_ptr(py, ptr);
            let list = bound.cast_unchecked::<pyo3::types::PyList>();

            let mut values: Vec<sea_query::Value> = Vec::with_capacity(list.len());

            for item in list.iter() {
                let val = self.0.serialize(py, item.as_ptr())?;
                values.push(val);
            }

            return Ok(sea_query::Value::Array(
                // array type is not important
                sea_query::ArrayType::BigInt,
                Some(Box::new(values)),
            ));
        }

        if pyo3::ffi::PyTuple_CheckExact(ptr) == 1 {
            let bound = pyo3::Bound::from_borrowed_ptr(py, ptr);
            let list = bound.cast_unchecked::<pyo3::types::PyTuple>();

            let mut values: Vec<sea_query::Value> = Vec::with_capacity(list.len());

            for item in list.iter() {
                let val = self.0.serialize(py, item.as_ptr())?;
                values.push(val);
            }

            return Ok(sea_query::Value::Array(
                // array type is not important
                sea_query::ArrayType::BigInt,
                Some(Box::new(values)),
            ));
        }

        crate::new_error!(
            PyTypeError,
            "expected list/tuple for {} serialization, got {}",
            self.to_sql_type_name(),
            crate::internal::get_type_name(py, ptr)
        )
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Array(_, Some(array)) => {
                let pylist = pyo3::ffi::PyList_New(array.len() as isize);

                if pylist.is_null() {
                    return Err(pyo3::PyErr::fetch(py));
                }

                for (index, item) in array.iter().enumerate() {
                    let result = self.0.deserialize(py, item);

                    match result {
                        Ok(x) => {
                            if pyo3::ffi::PyList_SetItem(pylist, index as isize, x) == 0 {
                                pyo3::ffi::Py_DECREF(x);
                                pyo3::ffi::Py_DECREF(pylist);
                                return Err(pyo3::PyErr::fetch(py));
                            }
                        }
                        Err(e) => {
                            pyo3::ffi::Py_DECREF(pylist);
                            return Err(e);
                        }
                    }
                }

                Ok(pylist)
            }
            sea_query::Value::Array(_, None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected array for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

super::abstracts::implement_sqltype_pymethods!(
    PyVectorType,
    init(|length: Option<u32>| Self(length)),
    "int | None",
    signature(length = None)
);

#[pyo3::pymethods]
impl PyArrayType {
    #[new]
    fn __new__(element: RefBoundObject<'_>) -> pyo3::PyResult<(Self, PySQLTypeAbstract)> {
        let type_engine = TypeEngine::new(element)?;

        Ok((Self(type_engine), PySQLTypeAbstract))
    }

    /// Type name. e.g. `'INTEGER'`, `'STRING'`
    ///
    /// It also may be a property. This function must NOT raise any error.
    #[getter]
    fn __type_name__(&self) -> String {
        self.to_sql_type_name()
    }

    #[getter]
    fn element<'py>(&self, py: pyo3::Python<'py>) -> pyo3::Bound<'py, pyo3::PyAny> {
        let ptr = self.0 .1.as_ptr();
        unsafe { pyo3::Bound::from_borrowed_ptr(py, ptr) }
    }

    fn __repr__(&self) -> String {
        format!("ArrayType(element={})", self.0)
    }
}
