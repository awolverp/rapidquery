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
