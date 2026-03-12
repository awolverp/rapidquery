/// Implement python classes using `#[pyo3::pyclass]` macro.
///
/// Usage:
/// ```rust,no-run
/// implement_pyclass! {
///     [generic] PyMyClass as "MyClass" { field: String }
/// }
/// ```
#[macro_export]
macro_rules! implement_pyclass {
    (
        $(#[$outer:meta])*
        [$($pyclass_args:tt)*] $struct_name:ident as $python_name:literal $($rest:tt)*
    ) => {
        #[
            pyo3::pyclass(
                module = "rapidquery._lib",
                name = $python_name,
                frozen,
                immutable_type,
                skip_from_py_object,
                $($pyclass_args)*
            )
        ]
        $(#[$outer])*
        pub struct $struct_name $($rest)*
    };

    (
        $(#[$outer:meta])*
        immutable [$($pyclass_args:tt)*] $struct_name:ident($state_name:ident) as $python_name:literal $($rest:tt)*
    ) => {
        $(#[$outer])*
        pub struct $state_name $($rest)*

        $crate::implement_pyclass! {
            $(#[$outer])*
            [$($pyclass_args)*] $struct_name as $python_name (pub $crate::internal::uninitialized::ImmutableUninit<$state_name>);
        }

        impl $struct_name {
            #[inline]
            #[must_use]
            pub fn uninit() -> Self {
                Self(
                    $crate::internal::uninitialized::ImmutableUninit::uninit()
                )
            }
        }

        impl From<$state_name> for $struct_name {
            fn from(value: $state_name) -> Self {
                Self(
                    $crate::internal::uninitialized::ImmutableUninit::new(value)
                )
            }
        }

        impl AsRef<$state_name> for $struct_name {
            fn as_ref(&self) -> &$state_name {
                self.0.as_ref()
            }
        }
    };

    (
        $(#[$outer:meta])*
        mutable [$($pyclass_args:tt)*] $struct_name:ident($state_name:ident) as $python_name:literal $($rest:tt)*
    ) => {
        $(#[$outer])*
        pub struct $state_name $($rest)*

        $crate::implement_pyclass! {
            $(#[$outer])*
            [$($pyclass_args)*] $struct_name as $python_name (pub $crate::internal::uninitialized::MutableUninit<$state_name>);
        }

        impl $struct_name {
            #[inline]
            #[must_use]
            pub fn uninit() -> Self {
                Self(
                    $crate::internal::uninitialized::MutableUninit::uninit()
                )
            }
        }

        impl From<$state_name> for $struct_name {
            fn from(value: $state_name) -> Self {
                Self(
                    $crate::internal::uninitialized::MutableUninit::new(value)
                )
            }
        }
    };
}

/// Creates new [`PyErr`]
///
/// Usage:
/// ```rust,no-run
/// new_raw_error!(PyValueError, "Message")
/// new_raw_error!(PyValueError, "Message {}", arg1)
/// ```
#[macro_export]
macro_rules! new_py_error {
    ($name:ident, $message:expr) => {
        ::pyo3::exceptions::$name::new_err($message)
    };
    ($name:ident, $message:expr, $($args:tt)*) => {
        ::pyo3::exceptions::$name::new_err(
            format!($message, $($args)*)
        )
    };
}

/// Creates new [`Err(PyErr)`]
///
/// Usage:
/// ```rust,no-run
/// new_error!(PyValueError, "Message")
/// new_error!(PyValueError, "Message {}", arg1)
/// ```
#[macro_export]
macro_rules! new_error {
    ($name:ident, $message:expr) => {
        Err($crate::new_py_error!($name, $message))
    };
    ($name:ident, $message:expr, $($args:tt)*) => {
        Err($crate::new_py_error!($name, $message, $($args)*))
    };
}

#[macro_export]
macro_rules! build_schema_statement {
    ($backend:expr, $stmt:expr) => {{
        let builder = $crate::internal::get_schema_builder($backend)?;
        let assert_unwind = std::panic::AssertUnwindSafe(|| $stmt.build_any(&*builder));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}

#[macro_export]
macro_rules! build_query_statement {
    ($backend:expr, $stmt:expr) => {{
        let builder = $crate::internal::get_query_builder($backend)?;
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
        let builder = $crate::internal::get_query_builder($backend)?;

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
