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

#[macro_export]
macro_rules! build_schema_statement {
    ($backend:expr, $stmt:expr) => {{
        let builder = $crate::internal::statements::get_schema_builder($backend)?;
        let assert_unwind = std::panic::AssertUnwindSafe(|| $stmt.build_any(&*builder));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}

#[macro_export]
macro_rules! build_query_statement {
    ($backend:expr, $stmt:expr) => {{
        let builder = $crate::internal::statements::get_query_builder($backend)?;
        let mut sql = String::with_capacity(255);

        let assert_unwind =
            std::panic::AssertUnwindSafe(|| $stmt.build_collect_any_into(&*builder, &mut sql));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))?;

        Ok(sql)
    }};
}

#[macro_export]
macro_rules! build_query_parts {
    ($py:expr, $backend:expr, $stmt:expr) => {{
        let builder = $crate::internal::statements::get_query_builder($backend)?;

        let (placeholder, numbered) = builder.placeholder();
        let mut sql = sea_query::SqlWriterValues::new(placeholder, numbered);

        let assert_unwind =
            std::panic::AssertUnwindSafe(|| $stmt.build_collect_any_into(&*builder, &mut sql));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))?;

        let (sql, values) = sql.into_parts();

        let values = {
            values
                .into_iter()
                .map(|x| $crate::common::value::ValueState::from_sea_query_value($py, x))
                .map($crate::common::value::PyValue::from)
        };

        unsafe {
            let tuple_ptr = pyo3::ffi::PyTuple_New(values.len() as isize);
            if tuple_ptr.is_null() {
                return Err(pyo3::PyErr::fetch($py));
            }

            for (index, key) in values.enumerate() {
                let key = pyo3::Py::new($py, key).unwrap().into_ptr();

                if pyo3::ffi::PyTuple_SetItem(tuple_ptr, index as isize, key) == -1 {
                    pyo3::ffi::Py_XDECREF(tuple_ptr);
                    pyo3::ffi::Py_XDECREF(key);
                    return Err(pyo3::PyErr::fetch($py));
                }
            }

            Ok((sql, pyo3::Bound::from_owned_ptr($py, tuple_ptr)))
        }
    }};
}
