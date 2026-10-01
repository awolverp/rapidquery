use pyo3::types::PyTupleMethods;

use super::enum_::Ident;
use super::enum_::IdentPart;
use super::enum_::ParseIdentError;
use super::extractors::ExtractedIdentArgument;
use super::extractors::extract_ident_argument;
use crate::internal::alias;

/// Convert a native part into the matching Python object (`str`, `Unquote`,
/// or `Asterisk`).
#[inline]
fn ident_part_to_pyobject<'py>(
    py: pyo3::Python<'py>,
    ident_part: &IdentPart,
) -> pyo3::PyResult<alias::BoundObject<'py>> {
    match ident_part {
        IdentPart::Quote(s) => Ok(pyo3::types::PyString::new(py, s.as_str()).into_any()),
        IdentPart::Unquote(s) => {
            Ok(pyo3::Bound::new(py, PyUnquote(arcstr::ArcStr::from(s.as_str())))?.into_any())
        }
        IdentPart::Asterisk => Ok(pyo3::Bound::new(py, PyAsterisk)?.into_any()),
    }
}

/// Represents a SQL identifier (`schema.table.column`).
///
/// An identifier is a sequence of parts. Each constructor argument is one
/// part: a quoted name (`str`), an `Unquote` name, an `Asterisk` wildcard,
/// or a nested `Ident` whose parts are flattened.
///
/// Never empty. At most 10 parts. An asterisk may appear only as the last
/// part.
///
/// Args:
///     parts: One argument per part. A `str` is always quoted, including
///             `"*"` (a column named star). Pass `Asterisk` for the SQL
///             wildcard. Dotted names are `Ident("schema", "table")`, not
///             `Ident("schema.table")`.
///
/// Raises:
///     TypeError: If an argument is not `str`, `Unquote`, `Asterisk`, or
///             `Ident`.
///     ValueError: If there are no parts, a part is empty, `Asterisk` is
///             not last, an `Unquote` body is not a simple ASCII name, or
///             there are more than 10 parts.
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

                Ident::from_parts(acc)
                    .map_err(|e| rapidquery_macros::pyerr!(PyValueError, "{}", e))?
            }
        };

        Ok(
            pyo3::PyClassInitializer::from(super::base::PyExpr(super::enum_::ExprInner::Ident(
                ident,
            )))
            .add_subclass(PyIdent),
        )
    }

    fn __len__(slf: pyo3::PyRef<'_, Self>) -> usize {
        Self::inner(&slf).len()
    }

    fn __getitem__<'py>(
        slf: pyo3::PyRef<'py, Self>,
        index: isize,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
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

    /// Returns the parts as a list (`str`, `Unquote`, or `Asterisk`).
    fn to_list(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<Vec<alias::PyObject>> {
        Self::inner(&slf)
            .parts()
            .iter()
            .map(|part| ident_part_to_pyobject(py, part).map(|bound| bound.unbind()))
            .collect()
    }

    fn __hash__(slf: pyo3::PyRef<'_, Self>) -> u64 {
        use std::hash::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;

        let mut state = DefaultHasher::default();
        let inner = Self::inner(&slf);
        inner.hash(&mut state);
        state.finish()
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return true;
        }

        Self::inner(&slf).eq(Self::inner(&other))
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return true;
        }

        Self::inner(&slf).ne(Self::inner(&other))
    }
}

/// Marker for an unquoted identifier part.
///
/// The body is emitted without dialect delimiters. It must be a simple
/// ASCII name (`[A-Za-z_][A-Za-z0-9_]*`). This is **not** a raw-SQL
/// fragment (`COUNT(*)` belongs on a later expression type).
///
/// Args:
///     name: The identifier body, unquoted.
///
/// Raises:
///     ValueError: If `name` is empty or not a simple ASCII identifier
///             (letters, digits, underscore; first character not a digit).
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

    /// Returns the unquoted identifier body.
    ///
    /// Returns:
    ///     The string passed to the constructor.
    #[getter]
    fn get_name(&self) -> &str {
        self.0.as_str()
    }

    fn __repr__(&self) -> String {
        format!("Unquote({:?})", self.0.as_str())
    }
}

/// SQL wildcard `*`.
///
/// Pass the class (`Asterisk`) or an instance (`Asterisk()`) as an
/// identifier part. It must be the last part of an `Ident` (`t.*`).
/// `Ident("*")` is a quoted column named star, not this wildcard.
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
