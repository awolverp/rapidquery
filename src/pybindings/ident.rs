//! Python wrappers around [`crate::native::ident`].
//!
//! Native [`Ident`](crate::native::Ident) / [`IdentPart`](crate::native::IdentPart)
//! stay PyO3-free. This module owns the pyclasses and the extractor used by
//! later APIs:
//!
//! * [`PyIdent`] — `rapidquery._core.Ident`
//! * [`PyUnquote`] — `rapidquery._core.Unquote`
//! * [`PyAsterisk`] — `rapidquery._core.Asterisk`
//! * [`IdentExtractor`] — `str | Ident | Unquote | Asterisk | type[Asterisk]`
//!
//! A Python `str` is always a quoted part, including `"*"`. The wildcard is
//! `Asterisk`. Dotted names are `Ident("schema", "table")`; the native
//! [`FromStr`](std::str::FromStr) parser is not used on the Python constructor.

use pyo3::types::{PyAnyMethods, PySliceMethods, PyTupleMethods};
use smallvec::SmallVec;

use crate::{
    internal::{
        alias,
        utils::{IndexOrSlice, py_str},
    },
    native::{self, prepare_statement::prepare},
};

#[inline]
fn parse_err(err: native::ParseIdentError) -> pyo3::PyErr {
    rapidquery_macros::pyerr!(PyValueError, "{}", err)
}

/// Convert a native part into the matching Python object (`str`, `Unquote`,
/// or `Asterisk`).
#[inline]
fn ident_part_to_pyobject<'py>(
    py: pyo3::Python<'py>,
    ident_part: &native::IdentPart,
) -> pyo3::PyResult<alias::BoundObject<'py>> {
    match ident_part {
        native::IdentPart::Quote(s) => Ok(pyo3::types::PyString::new(py, s.as_str()).into_any()),
        native::IdentPart::Unquote(s) => {
            Ok(pyo3::Bound::new(py, PyUnquote(arcstr::ArcStr::from(s.as_str())))?.into_any())
        }
        native::IdentPart::Asterisk => Ok(pyo3::Bound::new(py, PyAsterisk)?.into_any()),
    }
}

/// `true` when `obj` is `Asterisk` or `type[Asterisk]`.
///
/// # Safety
///
/// `obj` must be a valid, non-null `PyObject*`. `ASTERISK_TYPE` must have
/// been interned by [`crate::typeref::initialize_typeref`]. The GIL must
/// be held.
#[inline]
unsafe fn is_asterisk(obj: *mut pyo3::ffi::PyObject) -> bool {
    let type_ptr = if pyo3::ffi::PyType_CheckExact(obj) == 1 {
        obj.cast::<pyo3::ffi::PyTypeObject>()
    } else {
        pyo3::ffi::Py_TYPE(obj)
    };
    type_ptr == crate::typeref::ASTERISK_TYPE
}

enum Extracted {
    Ident(native::Ident),
    Part(native::IdentPart),
}

/// Hot-path extract: `str` first, then Ident / Unquote / Asterisk.
#[inline]
fn extract_arg(
    obj: pyo3::prelude::Borrowed<'_, '_, pyo3::prelude::PyAny>,
) -> pyo3::PyResult<Extracted> {
    let ptr = obj.as_ptr();
    unsafe {
        // SAFETY: `ptr` is a live borrowed `PyObject`. Type pointers are
        // interned in `initialize_typeref` before any extractor runs.
        if pyo3::ffi::PyUnicode_CheckExact(ptr) == 1 {
            let s = py_str(obj.py(), ptr)?;
            let part = native::IdentPart::quoted(s).map_err(parse_err)?;
            return Ok(Extracted::Part(part));
        }

        if crate::typeref::check_exact(ptr, crate::typeref::IDENT_TYPE) {
            let ident = obj.cast_unchecked::<PyIdent>();
            return Ok(Extracted::Ident(ident.get().0.clone()));
        }

        if crate::typeref::check_exact(ptr, crate::typeref::UNQUOTE_TYPE) {
            let unquote = obj.cast_unchecked::<PyUnquote>();
            let part = native::IdentPart::unquoted(unquote.get().0.as_str()).map_err(parse_err)?;
            return Ok(Extracted::Part(part));
        }

        if is_asterisk(ptr) {
            return Ok(Extracted::Part(native::IdentPart::Asterisk));
        }
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected str, Ident, Unquote or Asterisk, got {}",
        unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
    )
}

/// Extracts a native [`Ident`](crate::native::Ident) from a Python object.
///
/// Accepts `Ident`, `str` (one quoted part, including `"*"`), `Unquote`,
/// `Asterisk`, and `type[Asterisk]`. Sequences are rejected.
pub struct IdentExtractor(pub native::Ident);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for IdentExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        match extract_arg(obj)? {
            Extracted::Ident(ident) => Ok(Self(ident)),
            Extracted::Part(part) => Ok(Self(native::Ident::from_one(part))),
        }
    }
}

/// Represents a SQL identifier (`schema.table.column`).
///
/// An identifier is a sequence of parts. Each constructor argument is one
/// part: a quoted name (`str`), an `Unquote` name, an `Asterisk` wildcard,
/// or a nested `Ident` whose parts are flattened.
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
#[rapidquery_macros::pyclass("Ident", skip_from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyIdent(pub native::Ident);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for PyIdent {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        IdentExtractor::extract(obj).map(|IdentExtractor(inner)| Self(inner))
    }
}

#[pyo3::pymethods]
impl PyIdent {
    #[new]
    #[pyo3(signature = (*parts))]
    fn __new__(parts: &pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<Self> {
        match parts.len() {
            0 => Err(parse_err(native::ParseIdentError::Empty)),
            1 => match extract_arg(parts.get_item(0)?.as_borrowed())? {
                Extracted::Ident(ident) => Ok(Self(ident)),
                Extracted::Part(part) => Ok(Self(native::Ident::from_one(part))),
            },
            _ => {
                let mut acc = SmallVec::<[native::IdentPart; 4]>::new();
                for item in parts.iter() {
                    match extract_arg(item.as_borrowed())? {
                        Extracted::Ident(ident) => acc.extend(ident.into_parts()),
                        Extracted::Part(part) => acc.push(part),
                    }
                }
                native::Ident::from_parts(acc).map(Self).map_err(parse_err)
            }
        }
    }

    fn __len__(&self) -> usize {
        self.0.len()
    }

    /// Returns a part by index, or a sub-identifier by slice.
    ///
    /// Args:
    ///     index: An `int` (negative indices count from the end) or a
    ///             `slice`.
    ///
    /// Returns:
    ///     A `str`, `Unquote`, or `Asterisk` for an integer index. An
    ///     `Ident` for a slice.
    ///
    /// Raises:
    ///     IndexError: If the integer index is out of range.
    ///     ValueError: If the slice selects no parts.
    fn __getitem__<'py>(
        &self,
        py: pyo3::Python<'py>,
        index: IndexOrSlice,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let parts = self.0.parts();
        let len = parts.len() as isize;

        match index {
            IndexOrSlice::Index(i) => {
                let idx = if i < 0 { i + len } else { i };
                if idx < 0 || idx >= len {
                    return rapidquery_macros::pyerr!(
                        Err(PyIndexError),
                        "Ident index out of range"
                    );
                }
                ident_part_to_pyobject(py, &parts[idx as usize])
            }
            IndexOrSlice::Slice(slice) => {
                let indices = slice.bind(py).indices(len)?;
                let mut selected = Vec::with_capacity(indices.slicelength);

                let mut i = indices.start;
                if indices.step > 0 {
                    while i < indices.stop {
                        selected.push(parts[i as usize].clone());
                        i += indices.step;
                    }
                } else {
                    while i > indices.stop {
                        selected.push(parts[i as usize].clone());
                        i += indices.step;
                    }
                }

                let inner = native::Ident::from_parts(selected).map_err(parse_err)?;
                Ok(pyo3::Bound::new(py, PyIdent(inner))?.into_any())
            }
        }
    }

    fn __iter__<'py>(
        &self,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::types::PyIterator>> {
        let items = self
            .0
            .parts()
            .iter()
            .map(|part| ident_part_to_pyobject(py, part))
            .collect::<pyo3::PyResult<Vec<_>>>()?;
        let list = pyo3::types::PyList::new(py, items)?;
        list.try_iter()
    }

    /// Returns the parts as a list (`str`, `Unquote`, or `Asterisk`).
    fn to_list(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<Vec<alias::PyObject>> {
        self.0
            .parts()
            .iter()
            .map(|part| ident_part_to_pyobject(py, part).map(|bound| bound.unbind()))
            .collect()
    }

    /// Renders this identifier as SQL for `dialect`.
    ///
    /// Args:
    ///     dialect: A dialect class (`PostgreSQL`) or instance
    ///             (`PostgreSQL()`).
    ///
    /// Returns:
    ///     The dotted identifier with each part quoted or emitted according
    ///     to its kind.
    fn to_sql(
        &self,
        py: pyo3::Python<'_>,
        dialect: super::dialect::PySqlDialect,
    ) -> pyo3::PyResult<String> {
        let (sql, _) = prepare(py, &self.0, &dialect.0)
            .map_err(|e| rapidquery_macros::pyerr!(PyRuntimeError, "{}", e))?;

        Ok(sql)
    }

    fn __repr__(&self) -> String {
        use std::fmt::Write;

        let mut result = String::from("Ident(");
        let parts = self.0.parts();
        for (index, item) in parts.iter().enumerate() {
            match item {
                native::IdentPart::Quote(x) => write!(result, "{x:?}").unwrap(),
                native::IdentPart::Unquote(x) => write!(result, "Unquote({x:?})").unwrap(),
                native::IdentPart::Asterisk => result.push_str("Asterisk"),
            }
            if index + 1 < parts.len() {
                result.push_str(", ");
            }
        }
        result.push(')');
        result
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
#[rapidquery_macros::pyclass("Unquote", skip_from_py_object, eq, hash)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyUnquote(pub arcstr::ArcStr);

#[pyo3::pymethods]
impl PyUnquote {
    #[new]
    #[pyo3(signature = (name, /))]
    fn __new__(name: &str) -> pyo3::PyResult<Self> {
        native::IdentPart::unquoted(name).map_err(parse_err)?;
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
#[rapidquery_macros::pyclass("Asterisk", skip_from_py_object, eq, hash)]
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
