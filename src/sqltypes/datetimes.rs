use crate::sqltypes::abstracts::NativeSQLType;
use crate::sqltypes::abstracts::PySQLTypeAbstract;

use chrono::TimeZone;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyTzInfoAccess;
use pyo3::IntoPyObject;

implement_pyclass! {
    (
        /// Date and time column type (DATETIME).
        ///
        /// Stores both date and time information without timezone awareness.
        /// Suitable for recording timestamps, event times, or scheduling information
        /// when timezone handling is managed at the application level.
        ///
        /// @extends SQLTypeAbstract[datetime.datetime,datetime.datetime]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyDateTimeType as "DateTimeType";
    )
    (
        /// Timestamp column type (TIMESTAMP).
        ///
        /// Stores timestamp values, often with automatic update capabilities.
        /// Behavior varies by database system.
        ///
        /// @extends SQLTypeAbstract[datetime.datetime | int | float,datetime.datetime]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyTimestampType as "TimestampType" (pub bool);
    )
    (
        /// Time-only column type (TIME).
        ///
        /// Stores time information without date component. Useful for storing
        /// daily schedules, opening hours, or any time-based data that repeats
        /// daily regardless of the specific date.
        ///
        /// @extends SQLTypeAbstract[datetime.time,datetime.time]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyTimeType as "TimeType";
    )
    (
        /// Date-only column type (DATE).
        ///
        /// Stores date information without time component. Ideal for birth dates,
        /// deadlines, or any date-based data where time precision is not needed.
        ///
        /// @extends SQLTypeAbstract[datetime.date,datetime.date]
        #[derive(Debug, Clone, Copy)]
        pub struct [extends=PySQLTypeAbstract] PyDateType as "DateType";
    )
}

#[inline(always)]
unsafe fn _serialize_datetime(
    py: pyo3::Python,
    ptr: *mut pyo3::ffi::PyObject,
) -> pyo3::PyResult<sea_query::Value> {
    let val: pyo3::Bound<'_, pyo3::types::PyDateTime> =
        pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

    let tzinfo = val.get_tzinfo();

    if tzinfo.is_none() {
        let result = Box::new(val.extract()?);
        Ok(sea_query::Value::ChronoDateTime(Some(result)))
    } else {
        let result = Box::new(val.extract()?);
        Ok(sea_query::Value::ChronoDateTimeWithTimeZone(Some(result)))
    }
}

impl NativeSQLType for PyDateTimeType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::DateTime
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::Py_TYPE(ptr) != crate::typeref::STD_DATETIME_TYPE {
            Err(typeerror!("expected datetime.datetime, got {:?}", py, ptr))
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        _serialize_datetime(py, ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::ChronoDateTime(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTimeWithTimeZone(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTimeUtc(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTimeLocal(Some(x)) => {
                let pyobject = x.to_utc().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTime(None)
            | sea_query::Value::ChronoDateTimeWithTimeZone(None)
            | sea_query::Value::ChronoDateTimeUtc(None)
            | sea_query::Value::ChronoDateTimeLocal(None) => Ok(pyo3::ffi::Py_None()),

            _ => invalid_value_for_deserialize!("datetime", value),
        }
    }
}

impl NativeSQLType for PyTimestampType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        if self.0 {
            sea_query::ColumnType::TimestampWithTimeZone
        } else {
            sea_query::ColumnType::Timestamp
        }
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::Py_TYPE(ptr) != crate::typeref::STD_DATETIME_TYPE
            && pyo3::ffi::PyLong_CheckExact(ptr) != 1
            && pyo3::ffi::PyFloat_CheckExact(ptr) != 1
        {
            Err(typeerror!(
                "expected datetime.datetime, int, or float, got {:?}",
                py,
                ptr
            ))
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
            let mut has_overflow: i32 = 0;
            let num = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);

            if has_overflow == 1 {
                return Err(pyo3::exceptions::PyOverflowError::new_err("out of range"));
            }

            match chrono::Utc.timestamp_opt(num, 0) {
                chrono::offset::LocalResult::Single(result) => {
                    return Ok(sea_query::Value::ChronoDateTimeUtc(Some(Box::new(result))));
                }
                _ => {
                    return Err(pyo3::exceptions::PyOverflowError::new_err(
                        "timestamp is invalid",
                    ));
                }
            }
        }

        if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
            let num = pyo3::ffi::PyFloat_AsDouble(ptr);

            match chrono::Utc
                .timestamp_opt(num.trunc() as i64, (num.fract() * 1_000_000_000.0) as u32)
            {
                chrono::offset::LocalResult::Single(result) => {
                    return Ok(sea_query::Value::ChronoDateTimeUtc(Some(Box::new(result))));
                }
                _ => {
                    return Err(pyo3::exceptions::PyOverflowError::new_err(
                        "timestamp is invalid",
                    ));
                }
            }
        }

        _serialize_datetime(py, ptr)
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::ChronoDateTime(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTimeWithTimeZone(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTimeUtc(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDateTime(None)
            | sea_query::Value::ChronoDateTimeWithTimeZone(None)
            | sea_query::Value::ChronoDateTimeUtc(None) => Ok(pyo3::ffi::Py_None()),

            _ => invalid_value_for_deserialize!("datetime", value),
        }
    }
}

impl NativeSQLType for PyDateType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Date
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::Py_TYPE(ptr) != crate::typeref::STD_DATE_TYPE {
            Err(typeerror!("expected datetime.date, got {:?}", py, ptr))
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        let val: pyo3::Bound<'_, pyo3::types::PyDate> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let result = Box::new(val.extract()?);
        Ok(sea_query::Value::ChronoDate(Some(result)))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::ChronoDate(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoDate(None) => Ok(pyo3::ffi::Py_None()),
            _ => invalid_value_for_deserialize!("date", value),
        }
    }
}

impl NativeSQLType for PyTimeType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Time
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::Py_TYPE(ptr) != crate::typeref::STD_TIME_TYPE {
            Err(typeerror!("expected datetime.time, got {:?}", py, ptr))
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        let val: pyo3::Bound<'_, pyo3::types::PyTime> =
            pyo3::Bound::from_borrowed_ptr(py, ptr).cast_into()?;

        let result = Box::new(val.extract()?);
        Ok(sea_query::Value::ChronoTime(Some(result)))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::ChronoTime(Some(x)) => {
                let pyobject = x.clone().into_pyobject(py)?;
                Ok(pyobject.into_ptr())
            }
            sea_query::Value::ChronoTime(None) => Ok(pyo3::ffi::Py_None()),
            _ => invalid_value_for_deserialize!("time", value),
        }
    }
}

super::abstracts::implement_native_pymethods!(PyDateTimeType);
super::abstracts::implement_native_pymethods!(PyDateType);
super::abstracts::implement_native_pymethods!(PyTimeType);
super::abstracts::implement_native_pymethods!(
    PyTimestampType,
    init(|timezone: bool| Self(timezone)),
    "bool",
    signature(timezone = false)
);
