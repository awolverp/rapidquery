/// Creates a new [`PyErr`] of the given exception type.
#[macro_export]
macro_rules! new_pyerr {
    (Err($name:ident), $msg:expr $(,)?) => {
        Err(::pyo3::exceptions::$name::new_err($msg))
    };
    (Err($name:ident), $fmt:expr, $($args:tt)*) => {
        Err(::pyo3::exceptions::$name::new_err(format!($fmt, $($args)*)))
    };
    ($name:ident, $msg:expr $(,)?) => {
        ::pyo3::exceptions::$name::new_err($msg)
    };
    ($name:ident, $fmt:expr, $($args:tt)*) => {
        ::pyo3::exceptions::$name::new_err(format!($fmt, $($args)*))
    };
}
