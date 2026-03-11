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
