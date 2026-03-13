pub mod macro_rules;
pub mod parameters;
pub mod repr;
pub mod type_engine;
pub mod uninitialized;

/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
#[inline]
pub fn get_type_name<'a>(py: pyo3::Python<'a>, obj: *mut pyo3::ffi::PyObject) -> String {
    use pyo3::types::{PyStringMethods, PyTypeMethods};

    unsafe {
        let type_ = pyo3::ffi::Py_TYPE(obj);

        if type_.is_null() {
            String::from("<unknown>")
        } else {
            let obj = pyo3::types::PyType::from_borrowed_type_ptr(py, type_);

            obj.fully_qualified_name()
                .map(|x| x.to_string_lossy().into_owned())
                .unwrap_or_else(|_| String::from("<unknown>"))
        }
    }
}

pub type PyObject = pyo3::Py<pyo3::PyAny>;
pub type BoundArgs<'a> = &'a pyo3::Bound<'a, pyo3::types::PyTuple>;
pub type BoundKwargs<'a> = &'a pyo3::Bound<'a, pyo3::types::PyDict>;
pub type BoundObject<'a> = pyo3::Bound<'a, pyo3::PyAny>;
pub type RefBoundObject<'a> = &'a pyo3::Bound<'a, pyo3::PyAny>;

pub trait ToSeaQuery<Output> {
    /// Convert to sea_query structures.
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> Output;
}

#[inline(always)]
pub fn get_schema_builder(
    name: impl AsRef<str>,
) -> pyo3::PyResult<Box<dyn sea_query::SchemaBuilder>> {
    let name = name.as_ref();

    if name == "sqlite" {
        Ok(Box::new(sea_query::SqliteQueryBuilder))
    } else if name == "mysql" {
        Ok(Box::new(sea_query::MysqlQueryBuilder))
    } else if name == "postgresql" || name == "postgres" {
        Ok(Box::new(sea_query::PostgresQueryBuilder))
    } else {
        Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid backend value, got {name}"
        )))
    }
}

#[inline(always)]
pub fn get_query_builder(
    name: impl AsRef<str>,
) -> pyo3::PyResult<Box<dyn sea_query::QueryBuilder>> {
    let name = name.as_ref();

    if name == "sqlite" {
        Ok(Box::new(sea_query::SqliteQueryBuilder))
    } else if name == "mysql" {
        Ok(Box::new(sea_query::MysqlQueryBuilder))
    } else if name == "postgresql" || name == "postgres" {
        Ok(Box::new(sea_query::PostgresQueryBuilder))
    } else {
        Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid backend value, got {name}"
        )))
    }
}
