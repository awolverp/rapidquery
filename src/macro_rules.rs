/// Implements a `#[pyclass]` with pre-defined pyclass arguments.
///
/// # Example
///
/// ```ignore
/// implement_pyclass! {
///     [] MyClass as "MyClass" { field: type }
/// }
/// ```
#[macro_export]
macro_rules! implement_pyclass {
    (
        $(#[$outer:meta])*
        [$($pyclass_args:tt)*] $struct_name:ident as $python_name:literal $($rest:tt)*
    ) => {
        #[pyo3::pyclass(
            module = "rapidquery._lib",
            name = $python_name,
            immutable_type,
            frozen,
            $($pyclass_args)*
        )]
        $(#[$outer])*
        pub struct $struct_name $($rest)*
    };
}

/// Creates a new [`PyErr`] of the given exception type.
#[macro_export]
macro_rules! new_py_error {
    ($name:ident, $msg:expr $(,)?) => {
        ::pyo3::exceptions::$name::new_err($msg)
    };
    ($name:ident, $fmt:expr, $($args:tt)*) => {
        ::pyo3::exceptions::$name::new_err(
            format!($fmt, $($args)*)
        )
    };
}

/// Converts `String` to enum, and return PyErr if couldn't match.
#[macro_export]
macro_rules! literal_to_enum {
    (
        $string:expr,
        $($literal:literal => $result:expr,)+
    ) => {
        match $string.as_str() {
            $($literal => Ok($result),)+
            _ => Err(
                new_py_error!(
                    PyValueError,
                    String::from(concat!("expected", $($literal,)+)) + &format!(", got {}", $string),
                )
            )
        }
    }
}

/// Builds a type hint from a module name and a member name in the module
///
/// ```
/// use pyo3::type_hint_identifier;
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = type_hint_identifier!("datetime", "date");
/// assert_eq!(T.to_string(), "datetime.date");
///
/// const T2: PyStaticExpr = type_hint_identifier!("builtins", "int");
/// assert_eq!(T2.to_string(), "int");
/// ```
#[macro_export]
#[cfg(feature = "experimental-inspect")]
macro_rules! type_hint_identifier {
    ($name:expr) => {
        pyo3::inspect::PyStaticExpr::Name { id: $name }
    };
    ($module:expr, $name:expr) => {
        pyo3::inspect::PyStaticExpr::Attribute {
            value: &pyo3::inspect::PyStaticExpr::Name { id: $module },
            attr: $name,
        }
    };
}

/// Builds the union of multiple type hints
///
/// ```
/// use pyo3::{type_hint_identifier, type_hint_union};
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = type_hint_union!(type_hint_identifier!("builtins", "int"), type_hint_identifier!("builtins", "float"));
/// assert_eq!(T.to_string(), "int | float");
/// ```
#[macro_export]
#[cfg(feature = "experimental-inspect")]
macro_rules! type_hint_union {
    ($e:expr) => { $e };
    ($l:expr , $($r:expr),+) => { pyo3::inspect::PyStaticExpr::BinOp {
        left: &$l,
        op: pyo3::inspect::PyStaticOperator::BitOr,
        right: &type_hint_union!($($r),+),
    } };
}

/// Builds a subscribed type hint
///
/// ```
/// use pyo3::{type_hint_identifier, type_hint_subscript};
/// use pyo3::inspect::PyStaticExpr;
///
/// const T: PyStaticExpr = type_hint_subscript!(type_hint_identifier!("collections.abc", "Sequence"), type_hint_identifier!("builtins", "float"));
/// assert_eq!(T.to_string(), "collections.abc.Sequence[float]");
///
/// const T2: PyStaticExpr = type_hint_subscript!(type_hint_identifier!("builtins", "dict"), type_hint_identifier!("builtins", "str"), type_hint_identifier!("builtins", "float"));
/// assert_eq!(T2.to_string(), "dict[str, float]");
/// ```
#[macro_export]
#[cfg(feature = "experimental-inspect")]
macro_rules! type_hint_subscript {
    ($l:expr, $r:expr) => {
        pyo3::inspect::PyStaticExpr::Subscript {
            value: &$l,
            slice: &$r
        }
    };
    ($l:expr, $($r:expr),*) => {
        pyo3::inspect::PyStaticExpr::Subscript {
            value: &$l,
            slice: &pyo3::inspect::PyStaticExpr::Tuple { elts: &[$($r),*] }
        }
    };
}
