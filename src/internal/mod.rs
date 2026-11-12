pub mod alias;
pub mod modules;
pub mod repr;
pub mod submodule_ext;
pub mod template;
pub mod utils;
pub mod writer;

/// A completely optional parameter extractor
pub enum OptionalExtractor<T> {
    Undefined,
    Defined(T),
}

impl<T> OptionalExtractor<T> {
    #[inline(always)]
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Defined(x) => Some(x),
            Self::Undefined => None,
        }
    }
}

impl<'a, 'py, T> pyo3::FromPyObject<'a, 'py> for OptionalExtractor<T>
where
    T: pyo3::FromPyObject<'a, 'py>,
{
    type Error = <T as pyo3::FromPyObject<'a, 'py>>::Error;

    #[inline]
    fn extract(obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>) -> Result<Self, Self::Error> {
        T::extract(obj).map(OptionalExtractor::Defined)
    }
}
