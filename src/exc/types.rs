//! Python exception types exported from `rapidquery.exc`.

pyo3::create_exception!(
    rapidquery.exc,
    Error,
    pyo3::exceptions::PyException,
    "Base of RapidQuery errors that are not a builtin."
);

pyo3::create_exception!(
    rapidquery.exc,
    DialectError,
    Error,
    "This dialect cannot render a construct RapidQuery otherwise accepts."
);

pyo3::create_exception!(
    rapidquery.exc,
    SerializeError,
    Error,
    "A Python object could not be written as a SQL literal."
);
