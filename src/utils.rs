use pyo3::types::PyStringMethods;
use pyo3::types::PyTypeMethods;

/// Implement python classes using `#[pyo3::pyclass]` macro.
///
/// Note: All of python classes are flagged as frozen, immutable_type, and
/// skip_from_py_object;
///
/// Usage:
/// ```rust
/// implement_pyclass! {
///     (
///         /// A simple pyclass
///         struct [] PyMyClass as "MyClass" { field: String }
///     )
///
///     (
///         /// A subclass and generic class
///         struct [generic, subclass] PyMyClass as "MyClass";
///     )
/// }
/// ```
#[macro_export]
macro_rules! implement_pyclass {
    (
        $(
            (
                $(#[$outer:meta])*
                pub struct [$($pyclass_args:tt)*] $struct_name:ident as $literal_name:literal $($rest:tt)*
            )
        )*
    ) => {
        $(
            #[
                pyo3::pyclass(
                    module = "rapidquery._lib",
                    name = $literal_name,
                    frozen,
                    immutable_type,
                    skip_from_py_object,
                    $($pyclass_args)*
                )
            ]
            $(#[$outer])*
            pub struct $struct_name $($rest)*
        )*
    };

    (
        $(#[$outer:meta])*
        pub struct [$($pyclass_args:tt)*] $struct_name:ident as $literal_name:literal $($rest:tt)*
    ) => {
        implement_pyclass! {
            (
                $(#[$outer])*
                pub struct [$($pyclass_args)*] $struct_name as $literal_name $($rest)*
            )
        }
    };
}

/// Implement python classes using `#[pyo3::pyclass]` macro which wrapes
/// [`parking_lot::Mutex`].
///
/// Note: All of python classes are flagged as frozen, immutable_type, and
/// skip_from_py_object;
///
/// Usage:
/// ```rust
/// implement_state_pyclass! {
///     /// A simple pyclass
///     pub struct [] PyMyClass(MyClassState) as "MyClass" { field: String }
///
///     /// A subclass and generic class
///     pub struct [generic, subclass] PyMyClass(MyClassState) as "MyClass";
/// }
/// ```
#[macro_export]
macro_rules! implement_state_pyclass {
    (
        $(#[$outer:meta])*
        pub struct [$($pyclass_args:tt)*] $struct_name:ident($state_name:ident) as $literal_name:literal $($rest:tt)*
    ) => {
        $(#[$outer])*
        pub struct $state_name $($rest)*

        implement_pyclass! {
            $(#[$outer])*
            pub struct [$($pyclass_args)*] $struct_name as $literal_name (pub ::parking_lot::Mutex<$state_name>);
        }

        impl From<$state_name> for $struct_name {
            fn from(value: $state_name) -> Self {
                $struct_name(::parking_lot::Mutex::new(value))
            }
        }
    };

    (
        $(#[$outer:meta])*
        pub enum [$($pyclass_args:tt)*] $struct_name:ident($state_name:ident) as $literal_name:literal $($rest:tt)*
    ) => {
        $(#[$outer])*
        pub enum $state_name $($rest)*

        implement_pyclass! {
            $(#[$outer])*
            pub struct [$($pyclass_args)*] $struct_name as $literal_name (pub ::parking_lot::Mutex<$state_name>);
        }

        impl From<$state_name> for $struct_name {
            fn from(value: $state_name) -> Self {
                $struct_name(::parking_lot::Mutex::new(value))
            }
        }
    };
}

/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
pub unsafe fn get_type_name<'a>(py: pyo3::Python<'a>, obj: *mut pyo3::ffi::PyObject) -> String {
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

/// Creates a new [`pyo3::exceptions::PyTypeError`]
///
/// ```rust
/// typeerror!(
///     "expected str, got {}",
///     py,
///     value.as_ptr(),
/// )
///
/// typeerror!("type error description")
/// ```
#[macro_export]
macro_rules! typeerror {
    (
        $message:expr $(,)?
    ) => {{
        pyo3::exceptions::PyTypeError::new_err($message)
    }};

    (
        $message:expr,
        $py:expr,
        $ptr:expr
    ) => {{
        #[allow(unused_unsafe)]
        pyo3::exceptions::PyTypeError::new_err(
            format!($message, unsafe { $crate::utils::get_type_name($py, $ptr) })
        )
    }};

    (
        $message:expr,
        $py:expr,
        $($ptr:expr,)*
    ) => {{
        #[allow(unused_unsafe)]
        pyo3::exceptions::PyTypeError::new_err(
            format!(
                $message,
                $(
                    unsafe { $crate::utils::get_type_name($py, $ptr) },
                )*
            )
        )
    }};
}

#[macro_export]
macro_rules! invalid_value_for_deserialize {
    ($expected:literal, $value:expr) => {
        Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "expected {} for deserialization, got {:?}",
            $expected, $value
        )))
    };
}

pub enum OptionalParam<'a> {
    Undefined,
    Defined(pyo3::Bound<'a, pyo3::PyAny>),
}

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for OptionalParam<'py> {
    type Error = pyo3::PyErr;

    fn extract(obj: pyo3::Borrowed<'a, 'py, pyo3::PyAny>) -> Result<Self, Self::Error> {
        Ok(Self::Defined(obj.to_owned()))
    }
}

pub trait ToSeaQuery<Output> {
    /// Convert to sea_query structures.
    fn to_sea_query<'a>(&self, py: pyo3::Python<'a>) -> Output;
}

#[inline]
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

#[macro_export]
macro_rules! build_schema_statement {
    ($backend:expr, $stmt:expr) => {{
        let builder = $crate::utils::get_schema_builder($backend)?;
        let assert_unwind = std::panic::AssertUnwindSafe(|| $stmt.build_any(&*builder));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}
