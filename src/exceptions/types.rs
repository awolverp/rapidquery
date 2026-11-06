pyo3::create_exception!(
    rapidquery.exception,
    UnsupportedError,
    pyo3::exceptions::PyException,
    "Related to unsupported dialect features"
);
