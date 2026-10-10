use pyo3::types::PyTupleMethods;

use super::enum_::Ident;
use super::enum_::IdentPart;
use super::enum_::ParseIdentError;
use super::extractors::ExtractedIdentArgument;
use super::extractors::extract_ident_argument;
use crate::internal::alias;
use crate::internal::repr;

/// Convert a native part into the matching Python object (`str`, `Unquote`,
/// or `Asterisk`).
#[inline]
fn ident_part_to_pyobject<'py>(
    py: pyo3::Python<'py>,
    ident_part: &IdentPart,
) -> pyo3::PyResult<alias::BoundObject<'py>> {
    match ident_part {
        IdentPart::Quote(s) => Ok(pyo3::types::PyString::new(py, s.as_str()).into_any()),
        IdentPart::Unquote(s) => Ok(pyo3::Bound::new(py, PyUnquote(arcstr::ArcStr::from(s.as_str())))?.into_any()),
        IdentPart::Asterisk => Ok(pyo3::Bound::new(py, PyAsterisk)?.into_any()),
    }
}

/// Dotted SQL name. A string part is quoted; `Asterisk` is the wildcard.
#[rapidquery_macros::pyclass("rapidquery.expr.Ident", skip_from_py_object, extends = super::base::PyExpr)]
#[derive(Debug, Default)]
pub struct PyIdent;

impl PyIdent {
    #[inline(always)]
    fn inner<'a>(slf: &'a pyo3::PyRef<'a, Self>) -> &'a Ident {
        debug_assert!(
            matches!(&slf.as_super().0, super::enum_::ExprInner::Ident(..)),
            "PyIdent instance always holds ExprInner::Ident"
        );

        match &slf.as_super().0 {
            super::enum_::ExprInner::Ident(ident) => ident,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

#[pyo3::pymethods]
impl PyIdent {
    #[new]
    #[pyo3(signature = (*parts))]
    fn __new__(parts: alias::ArgsType) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let ident = match parts.len() {
            0 => return rapidquery_macros::pyerr!(Err(PyValueError), "{}", ParseIdentError::Empty),
            1 => match extract_ident_argument(parts.get_item(0)?.as_borrowed())? {
                ExtractedIdentArgument::Ident(ident) => ident,
                ExtractedIdentArgument::Part(part) => Ident::from_one(part),
            },
            _ => {
                let mut acc = smallvec::SmallVec::<[IdentPart; 4]>::new();
                for item in parts.iter() {
                    match extract_ident_argument(item.as_borrowed())? {
                        ExtractedIdentArgument::Ident(ident) => acc.extend(ident.into_parts()),
                        ExtractedIdentArgument::Part(part) => acc.push(part),
                    }
                }

                Ident::from_parts(acc).map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?
            }
        };

        Ok(
            pyo3::PyClassInitializer::from(super::base::PyExpr(super::enum_::ExprInner::Ident(ident)))
                .add_subclass(PyIdent),
        )
    }

    fn __len__(slf: pyo3::PyRef<'_, Self>) -> usize {
        Self::inner(&slf).len()
    }

    fn __getitem__<'py>(slf: pyo3::PyRef<'py, Self>, index: isize) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let inner = Self::inner(&slf);

        let x = if index < 0 {
            (inner.len() as isize) + index
        } else {
            index
        };

        match inner.parts().get(x as usize) {
            Some(part) => ident_part_to_pyobject(slf.py(), part),
            None => rapidquery_macros::pyerr!(Err(PyIndexError), "index out of range"),
        }
    }

    /// Copy the parts into a list.
    fn to_list(slf: pyo3::PyRef<'_, Self>, py: pyo3::Python<'_>) -> pyo3::PyResult<Vec<alias::PyObject>> {
        Self::inner(&slf)
            .parts()
            .iter()
            .map(|part| ident_part_to_pyobject(py, part).map(|bound| bound.unbind()))
            .collect()
    }
}

/// Identifier body written with no quote characters.
#[rapidquery_macros::pyclass("rapidquery.expr.Unquote", skip_from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyUnquote(pub arcstr::ArcStr);

#[pyo3::pymethods]
impl PyUnquote {
    #[new]
    #[pyo3(signature = (name, /))]
    fn __new__(name: &str) -> pyo3::PyResult<Self> {
        // validate the `name`
        IdentPart::unquoted(name).map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?;

        Ok(Self(arcstr::ArcStr::from(name)))
    }

    /// The identifier body.
    #[getter]
    fn get_name(&self) -> &str {
        self.0.as_str()
    }

    fn __repr__(&self) -> String {
        let mut fmt = repr::ReprFormatter::new_call("Unquote");
        fmt.positional(self.0.as_str());
        fmt.into_string()
    }
}

/// SQL wildcard `*`, used as the last part of an identifier.
#[rapidquery_macros::pyclass("rapidquery.expr.Asterisk", skip_from_py_object, eq, hash)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyAsterisk;

#[pyo3::pymethods]
impl PyAsterisk {
    #[new]
    fn __new__() -> Self {
        Self
    }

    fn __repr__(&self) -> &'static str {
        "Asterisk()"
    }
}
