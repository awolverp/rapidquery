use pyo3::types::PyAnyMethods;
use pyo3::IntoPyObject;
use sea_query::IntoIden;
use std::str::FromStr;

use crate::sqltypes::abstracts::PySQLTypeAbstract;
use crate::sqltypes::abstracts::SQLTypeTrait;

crate::implement_pyclass! {
    /// Exact numeric decimal column type (DECIMAL/NUMERIC).
    ///
    /// Stores exact numeric values with fixed precision and scale. Essential for
    /// financial calculations, currency values, or any situation where exact
    /// decimal representation is required without floating-point approximation.
    ///
    /// @extends SQLTypeAbstract[decimal.Decimal | int | float | str]
    /// @signature (cls, context: tuple[int, int] | None = None)
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyDecimalType as "Decimal" (pub Option<(u32, u32)>);
}
crate::implement_pyclass! {
    /// UUID column type (UUID).
    ///
    /// Stores universally unique identifiers. Ideal for distributed systems,
    /// primary keys, or any situation where globally unique identifiers are
    /// needed without central coordination.
    ///
    /// @extends SQLTypeAbstract[uuid.UUID]
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyUUIDType as "UUID";
}
crate::implement_pyclass! {
    /// Internet address column type (INET).
    ///
    /// Stores IPv4 or IPv6 addresses, with or without subnet specification.
    /// More flexible than CIDR type, allowing both host addresses and network ranges.
    ///
    /// @extends SQLTypeAbstract[str]
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyINETType as "INET";
}
crate::implement_pyclass! {
    /// MAC address column type (MACADDR).
    ///
    /// Stores MAC (Media Access Control) addresses for network devices.
    /// Provides validation and formatting for 6-byte MAC addresses.
    ///
    /// @extends SQLTypeAbstract[str]
    #[derive(Debug, Clone, Copy)]
    [extends=PySQLTypeAbstract] PyMacAddressType as "MacAddress";
}
crate::implement_pyclass! {
    /// Enumeration column type (ENUM).
    ///
    /// Stores one value from a predefined set of allowed string values.
    /// Provides type safety and storage efficiency for categorical data
    /// with a fixed set of possible values.
    ///
    /// @extends SQLTypeAbstract[str | enum.Enum]
    /// @signature (name: str, variants: typing.Iterable[str])
    #[derive(Debug, Clone)]
    [extends=PySQLTypeAbstract] PyEnumType as "Enum" {
        pub name: sea_query::DynIden,
        pub variants: Vec<sea_query::DynIden>,
    }
}

impl SQLTypeTrait for PyDecimalType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Decimal(self.0)
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::PyLong_CheckExact(ptr) == 1 {
            let mut has_overflow: i32 = 0;
            let _ = pyo3::ffi::PyLong_AsLongLongAndOverflow(ptr, &mut has_overflow);

            if has_overflow == 1 {
                return Err(pyo3::exceptions::PyOverflowError::new_err("out of range"));
            }

            return Ok(());
        }

        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DECIMAL_TYPE
            || pyo3::ffi::PyFloat_CheckExact(ptr) == 1
        {
            return Ok(());
        }

        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let val = super::primitives::_serialize_string(py, ptr).map_err(|x| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string())
            })?;

            rust_decimal::Decimal::from_str_exact(&val).or_else(|_| {
                rust_decimal::Decimal::from_scientific(&val).map_err(|x| {
                    pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string())
                })
            })?;
            return Ok(());
        }

        crate::new_error!(
            PyTypeError,
            "expected decimal.Decimal/int/float/str for {} serialization, got {}",
            self.to_sql_type_name(),
            crate::internal::get_type_name(py, ptr)
        )
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

            return Ok(sea_query::Value::Decimal(Some(Box::new(
                rust_decimal::Decimal::new(num, 0),
            ))));
        }

        if pyo3::ffi::PyFloat_CheckExact(ptr) == 1 {
            use rust_decimal::prelude::FromPrimitive;
            let num = pyo3::ffi::PyFloat_AsDouble(ptr);

            return rust_decimal::Decimal::from_f64(num)
                .ok_or_else(|| {
                    pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "This value cannot be represented as a decimal",
                    )
                })
                .map(|x| sea_query::Value::Decimal(Some(Box::new(x))));
        }

        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let val = super::primitives::_serialize_string(py, ptr).map_err(|x| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string())
            })?;

            return rust_decimal::Decimal::from_str_exact(&val)
                .or_else(|_| {
                    rust_decimal::Decimal::from_scientific(&val).map_err(|x| {
                        pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string())
                    })
                })
                .map(|x| sea_query::Value::Decimal(Some(Box::new(x))));
        }

        if pyo3::ffi::Py_TYPE(ptr) == crate::typeref::STD_DECIMAL_TYPE {
            let borrowed = pyo3::Borrowed::from_ptr(py, ptr);
            let val = borrowed.to_string();

            return rust_decimal::Decimal::from_str_radix(&val, 10)
                .or_else(|_| {
                    rust_decimal::Decimal::from_scientific(&val).map_err(|x| {
                        pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string())
                    })
                })
                .map(|x| sea_query::Value::Decimal(Some(Box::new(x))));
        }

        crate::new_error!(
            PyTypeError,
            "expected decimal.Decimal/int/float/str for {} serialization, got {}",
            self.to_sql_type_name(),
            crate::internal::get_type_name(py, ptr)
        )
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        use pyo3::IntoPyObject;

        match value {
            sea_query::Value::Decimal(Some(x)) => x.into_pyobject(py).map(|x| x.into_ptr()),
            sea_query::Value::Decimal(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected decimal for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyUUIDType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Uuid
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        if pyo3::ffi::Py_TYPE(ptr) != crate::typeref::STD_UUID_TYPE {
            crate::new_error!(
                PyTypeError,
                "expected uuid.UUID for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            )
        } else {
            Ok(())
        }
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        let result = pyo3::Bound::from_borrowed_ptr(py, ptr)
            .clone()
            .extract::<::uuid::Uuid>()?;

        Ok(sea_query::Value::Uuid(Some(Box::new(result))))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::Uuid(Some(x)) => {
                let result = x.clone().into_pyobject(py)?;
                Ok(result.into_ptr())
            }
            sea_query::Value::Uuid(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected uuid for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyINETType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Inet
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        let string = super::primitives::_serialize_string(py, ptr)?;

        ipnetwork::Ipv4Network::from_str(&string)
            .map_err(|x| pyo3::exceptions::PyValueError::new_err(x.to_string()))?;

        Ok(())
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        let string = super::primitives::_serialize_string(py, ptr)?;

        ipnetwork::Ipv4Network::from_str(&string)
            .map(|x| sea_query::Value::IpNetwork(Some(Box::new(ipnetwork::IpNetwork::V4(x)))))
            .map_err(|x| pyo3::exceptions::PyValueError::new_err(x.to_string()))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::IpNetwork(Some(x)) => {
                super::primitives::_deserialize_string(py, &x.to_string())
            }
            sea_query::Value::IpNetwork(None) => Ok(pyo3::ffi::Py_None()),
            sea_query::Value::String(Some(x)) => super::primitives::_deserialize_string(py, x),
            sea_query::Value::String(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected ipnetwork/str for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

impl SQLTypeTrait for PyMacAddressType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::MacAddr
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        let string = super::primitives::_serialize_string(py, ptr)?;

        mac_address::MacAddress::from_str(&string)
            .map_err(|x| pyo3::exceptions::PyValueError::new_err(x.to_string()))?;

        Ok(())
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        let string = super::primitives::_serialize_string(py, ptr)?;

        mac_address::MacAddress::from_str(&string)
            .map(|x| sea_query::Value::MacAddress(Some(Box::new(x))))
            .map_err(|x| pyo3::exceptions::PyValueError::new_err(x.to_string()))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::MacAddress(Some(x)) => {
                super::primitives::_deserialize_string(py, &x.to_string())
            }
            sea_query::Value::MacAddress(None) => Ok(pyo3::ffi::Py_None()),
            sea_query::Value::String(Some(x)) => super::primitives::_deserialize_string(py, x),
            sea_query::Value::String(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected mac_address/str for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

const ENUM_TYPE_VALUE: &std::ffi::CStr = c"value";

impl SQLTypeTrait for PyEnumType {
    fn to_sea_query_column_type(&self) -> sea_query::ColumnType {
        sea_query::ColumnType::Enum {
            name: self.name.clone(),
            variants: self.variants.clone(),
        }
    }

    unsafe fn validate(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<()> {
        // https://github.com/ijl/orjson/blob/master/src/util.rs#L55
        if (*pyo3::ffi::Py_TYPE(ptr)).ob_base.ob_base.ob_type == crate::typeref::STD_ENUM_TYPE {
            let attribute = pyo3::ffi::PyObject_GetAttrString(ptr, ENUM_TYPE_VALUE.as_ptr());

            if attribute.is_null() {
                return Err(pyo3::PyErr::fetch(py));
            }

            if pyo3::ffi::PyUnicode_CheckExact(attribute) == 0 {
                let type_error = crate::new_error!(
                    PyTypeError,
                    "Enum value wasn't str for {} serialization, was {}",
                    self.to_sql_type_name(),
                    crate::internal::get_type_name(py, attribute)
                );

                pyo3::ffi::Py_DECREF(attribute);
                return type_error;
            }
        } else if pyo3::ffi::PyUnicode_CheckExact(ptr) == 0 {
            return crate::new_error!(
                PyTypeError,
                "expected Enum/str for {} serialization, got {}",
                self.to_sql_type_name(),
                crate::internal::get_type_name(py, ptr)
            );
        }

        Ok(())
    }

    unsafe fn serialize(
        &self,
        py: pyo3::Python,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> pyo3::PyResult<sea_query::Value> {
        let string = {
            if (*pyo3::ffi::Py_TYPE(ptr)).ob_base.ob_base.ob_type == crate::typeref::STD_ENUM_TYPE {
                let attribute = pyo3::ffi::PyObject_GetAttrString(ptr, ENUM_TYPE_VALUE.as_ptr());

                if attribute.is_null() {
                    return Err(pyo3::PyErr::fetch(py));
                }

                // SAFETY: Type of the attribute was checked in `validate` method
                attribute
            } else {
                pyo3::ffi::Py_INCREF(ptr);

                // SAFETY: Type of the attribute was checked in `validate` method
                ptr
            }
        };

        let serialized = super::primitives::_serialize_string(py, string);
        pyo3::ffi::Py_DECREF(string);

        serialized.map(|x| sea_query::Value::String(Some(Box::new(x))))
    }

    unsafe fn deserialize(
        &self,
        py: pyo3::Python,
        value: &sea_query::Value,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        match value {
            sea_query::Value::String(Some(x)) => super::primitives::_deserialize_string(py, x),
            sea_query::Value::String(None) => Ok(pyo3::ffi::Py_None()),
            _ => crate::new_error!(
                PyTypeError,
                "expected str for {} deserialization, got {:?}",
                self.to_sql_type_name(),
                value
            ),
        }
    }
}

super::abstracts::implement_sqltype_pymethods!(PyUUIDType);
super::abstracts::implement_sqltype_pymethods!(PyINETType);
super::abstracts::implement_sqltype_pymethods!(PyMacAddressType);
super::abstracts::implement_sqltype_pymethods!(
    PyDecimalType,
    init(|context: Option<(u32, u32)>| Self(context)),
    "tuple[int, int] | None",
    signature(context = None)
);

#[pyo3::pymethods]
impl PyEnumType {
    #[new]
    fn __new__(name: String, variants: Vec<String>) -> (Self, PySQLTypeAbstract) {
        let slf = Self {
            name: sea_query::Alias::new(name).into_iden(),
            variants: variants
                .into_iter()
                .map(|x| sea_query::Alias::new(x).into_iden())
                .collect(),
        };

        (slf, PySQLTypeAbstract)
    }

    /// Type name. e.g. `'INTEGER'`, `'STRING'`
    ///
    /// It also may be a property. This function must NOT raise any error.
    ///
    /// @signature (self) -> str
    #[getter]
    fn __type_name__(&self) -> String {
        self.to_sql_type_name()
    }

    /// @signature (self) -> str
    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    /// @signature (self) -> typing.Sequence[str]
    #[getter]
    fn variants(&self) -> Vec<String> {
        self.variants.iter().map(|x| x.to_string()).collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "EnumType(name={:?}, variants={:?})",
            self.name.to_string(),
            self.variants.iter().map(|x| x.to_string())
        )
    }
}
