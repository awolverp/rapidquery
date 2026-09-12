//! SQL identifiers.

use std::fmt;

use arcstr::ArcStr;
use smallvec::{SmallVec, smallvec};

use crate::dialect::SqlDialect;
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};

/// One segment of a dotted identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdentPart {
    /// Identifier that should be quoted and escaped for the dialect.
    Quote(ArcStr),
    /// Identifier emitted without delimiters. Must be a simple ASCII name.
    Unquote(ArcStr),
    /// SQL wildcard `*`.
    Asterisk,
}

impl IdentPart {
    /// Quoted part whose body is `s`.
    ///
    /// `"*"` is a quoted identifier named star, not [`IdentPart::Asterisk`].
    pub fn quoted(s: impl AsRef<str>) -> Result<Self, ParseIdentError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(ParseIdentError::EmptyChunk);
        }
        Ok(Self::Quote(ArcStr::from(s)))
    }

    /// Unquoted part whose body is `s`.
    ///
    /// Rejects anything that is not `[A-Za-z_][A-Za-z0-9_]*`.
    pub fn unquoted(s: impl AsRef<str>) -> Result<Self, ParseIdentError> {
        let s = s.as_ref();
        if s.is_empty() {
            return Err(ParseIdentError::EmptyChunk);
        }
        if !is_safe_unquoted(s) {
            return Err(ParseIdentError::InvalidUnquoted);
        }
        Ok(Self::Unquote(ArcStr::from(s)))
    }
}

impl<S: AsRef<str>> From<S> for IdentPart {
    /// Parser helper used by [`Ident::from_str`].
    ///
    /// A literal `"*"` becomes [`IdentPart::Asterisk`]. Everything else is
    /// [`IdentPart::Quote`] with no emptiness check — [`Ident::from_str`]
    /// rejects empty chunks first. Python `str` extract does **not** use
    /// this impl (it always goes through [`IdentPart::quoted`]).
    fn from(value: S) -> Self {
        let s = value.as_ref();
        if s == "*" {
            IdentPart::Asterisk
        } else {
            IdentPart::Quote(ArcStr::from(s))
        }
    }
}

impl WriteSql for IdentPart {
    #[inline]
    fn write_sql<W: SqlWriter>(&self, dialect: &SqlDialect, out: &mut W) -> PrepareResult<()> {
        match self {
            Self::Quote(name) => {
                dialect.quote_identifier(out, name.as_str())?;
            }
            Self::Unquote(name) => {
                out.push_str(name.as_str());
            }
            Self::Asterisk => {
                out.push_char('*');
            }
        }
        Ok(())
    }
}

/// Represents a SQL identifier (`schema.table.column`).
///
/// An identifier is a sequence of parts. Each constructor argument is one
/// part: a quoted name (`str`), an `Unquote` name, an `Asterisk` wildcard,
/// or a nested `Ident` whose parts are flattened.
///
/// Never empty. At most [`Ident::MAX_PARTS`] parts. An [`IdentPart::Asterisk`]
/// may appear only as the last part.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ident(SmallVec<[IdentPart; 4]>);

/// Why building an [`Ident`] failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseIdentError {
    /// No parts (empty string, empty constructor).
    Empty,
    /// A quoted or unquoted part was the empty string (`"a..b"`, `""` as a part).
    EmptyChunk,
    /// More than [`Ident::MAX_PARTS`] parts.
    TooManyParts,
    /// `*` appeared before the last part (`*.id`).
    AsteriskNotLast,
    /// Unquoted body was not a simple ASCII identifier.
    InvalidUnquoted,
}

impl fmt::Display for ParseIdentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "identifier is empty",
            Self::EmptyChunk => "identifier has an empty part",
            Self::TooManyParts => "identifier has too many parts",
            Self::AsteriskNotLast => "asterisk must be the last identifier part",
            Self::InvalidUnquoted => "unquoted identifier is not a simple ASCII name",
        })
    }
}

impl std::error::Error for ParseIdentError {}

/// `[A-Za-z_][A-Za-z0-9_]*`. Empty string is not safe.
fn is_safe_unquoted(s: &str) -> bool {
    let mut bytes = s.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }
    bytes.all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

impl Ident {
    /// Maximum number of dotted parts.
    pub const MAX_PARTS: usize = 10;

    /// Borrowed parts in order.
    #[inline]
    pub fn parts(&self) -> &[IdentPart] {
        &self.0
    }

    /// Number of parts. Always ≥ 1 for a successfully built `Ident`.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` when there are no parts. Unreachable for a validated `Ident`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Consume and yield the parts.
    #[inline]
    pub fn into_parts(self) -> SmallVec<[IdentPart; 4]> {
        self.0
    }

    pub(crate) fn python_repr(&self) -> String {
        use std::fmt::Write;

        let mut result = String::from("Ident(");
        let parts = self.parts();
        for (index, item) in parts.iter().enumerate() {
            match item {
                IdentPart::Quote(x) => write!(result, "{x:?}").unwrap(),
                IdentPart::Unquote(x) => write!(result, "Unquote({x:?})").unwrap(),
                IdentPart::Asterisk => result.push_str("Asterisk"),
            }
            if index + 1 < parts.len() {
                result.push_str(", ");
            }
        }
        result.push(')');
        result
    }

    /// One already-validated part. Prefer this over [`from_parts`] for a
    /// single `str` / `Unquote` / `Asterisk`.
    #[inline]
    pub fn from_one(part: IdentPart) -> Self {
        Self(smallvec![part])
    }

    /// Build from parts. Composition only: empty ident, empty chunk,
    /// [`MAX_PARTS`](Self::MAX_PARTS), asterisk-not-last.
    ///
    /// Unquoted charset is enforced by [`IdentPart::unquoted`], not here.
    #[cfg_attr(feature = "nightly", optimize(speed))]
    pub fn from_parts(iter: impl IntoIterator<Item = IdentPart>) -> Result<Self, ParseIdentError> {
        let mut parts = SmallVec::<[IdentPart; 4]>::new();

        for part in iter {
            if let Some(IdentPart::Asterisk) = parts.last() {
                return Err(ParseIdentError::AsteriskNotLast);
            }
            if parts.len() >= Self::MAX_PARTS {
                return Err(ParseIdentError::TooManyParts);
            }
            match &part {
                IdentPart::Quote(name) | IdentPart::Unquote(name) if name.is_empty() => {
                    return Err(ParseIdentError::EmptyChunk);
                }
                IdentPart::Quote(_) | IdentPart::Unquote(_) | IdentPart::Asterisk => {}
            }
            parts.push(part);
        }

        if parts.is_empty() {
            return Err(ParseIdentError::Empty);
        }

        Ok(Self(parts))
    }
}

impl std::str::FromStr for Ident {
    type Err = ParseIdentError;

    #[cfg_attr(feature = "nightly", optimize(speed))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseIdentError::Empty);
        }

        let mut parts = SmallVec::<[IdentPart; 4]>::new();

        for chunk in s.splitn(Self::MAX_PARTS + 1, '.') {
            if chunk.is_empty() {
                return Err(ParseIdentError::EmptyChunk);
            }
            if parts.len() >= Self::MAX_PARTS {
                return Err(ParseIdentError::TooManyParts);
            }
            // A bare `*` only makes sense as the final part (e.g. `table.*`).
            // If another part follows, demote the previous `*` to a quoted star.
            if let Some(last) = parts.last_mut()
                && matches!(last, IdentPart::Asterisk)
            {
                *last = IdentPart::Quote(ArcStr::from("*"));
            }
            parts.push(chunk.into());
        }

        Ok(Self(parts))
    }
}

impl WriteSql for Ident {
    #[inline]
    fn write_sql<W: SqlWriter>(&self, dialect: &SqlDialect, out: &mut W) -> PrepareResult<()> {
        write_separated(dialect, out, &self.0, ".")
    }
}

// ---------------------------------------------------------------------------
// Python surface (`Ident`, `Unquote`, `Asterisk`)
// ---------------------------------------------------------------------------
//
// A Python `str` is always a quoted part, including `"*"`. The wildcard is
// `Asterisk`. Dotted names are `Ident("schema", "table")`; the native
// [`FromStr`](std::str::FromStr) parser is not used on the Python constructor.

use pyo3::types::{PyAnyMethods, PySliceMethods, PyTupleMethods};

use crate::internal::alias;
use crate::internal::utils;
use crate::sql::write_sql;

#[inline]
fn parse_err(err: ParseIdentError) -> pyo3::PyErr {
    rapidquery_macros::pyerr!(PyValueError, "{}", err)
}

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
    Ident(Ident),
    Part(IdentPart),
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
            let s = utils::py_str(obj.py(), ptr)?;
            let part = IdentPart::quoted(s).map_err(parse_err)?;
            return Ok(Extracted::Part(part));
        }

        if crate::typeref::check_exact(ptr, crate::typeref::IDENT_TYPE) {
            let expr = obj.cast_unchecked::<super::PyExpr>();
            match &expr.get().0 {
                super::ExprEnum::Ident(ident) => {
                    return Ok(Extracted::Ident(ident.clone()));
                }
                _ => unreachable!("Ident instance always holds ExprEnum::Ident"),
            }
        }

        if crate::typeref::check_exact(ptr, crate::typeref::UNQUOTE_TYPE) {
            let unquote = obj.cast_unchecked::<PyUnquote>();
            let part = IdentPart::unquoted(unquote.get().0.as_str()).map_err(parse_err)?;
            return Ok(Extracted::Part(part));
        }

        if is_asterisk(ptr) {
            return Ok(Extracted::Part(IdentPart::Asterisk));
        }
    }

    rapidquery_macros::pyerr!(
        Err(PyTypeError),
        "expected str, Ident, Unquote or Asterisk, got {}",
        unsafe { crate::internal::utils::get_type_name(obj.py(), obj.as_ptr()) }
    )
}

/// Extracts a native [`Ident`](crate::Ident) from a Python object.
///
/// Accepts `Ident`, `str` (one quoted part, including `"*"`), `Unquote`,
/// `Asterisk`, and `type[Asterisk]`. Sequences are rejected.
pub struct IdentExtractor(pub Ident);

impl<'a, 'py> pyo3::FromPyObject<'a, 'py> for IdentExtractor {
    type Error = pyo3::PyErr;

    #[inline]
    fn extract(
        obj: pyo3::prelude::Borrowed<'a, 'py, pyo3::prelude::PyAny>,
    ) -> Result<Self, Self::Error> {
        match extract_arg(obj)? {
            Extracted::Ident(ident) => Ok(Self(ident)),
            Extracted::Part(part) => Ok(Self(Ident::from_one(part))),
        }
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
#[rapidquery_macros::pyclass(
    "Ident",
    skip_from_py_object,
    extends = super::PyExpr,
    module = "rapidquery._core.expr"
)]
#[derive(Debug, Default)]
pub struct PyIdent;

#[inline]
fn ident_inner<'a>(slf: &'a pyo3::PyRef<'a, PyIdent>) -> &'a Ident {
    match &slf.as_super().0 {
        super::ExprEnum::Ident(ident) => ident,
        _ => unreachable!("Ident instance always holds ExprEnum::Ident"),
    }
}

#[inline]
fn wrap_ident(ident: Ident) -> pyo3::PyClassInitializer<PyIdent> {
    pyo3::PyClassInitializer::from(super::PyExpr(super::ExprEnum::Ident(ident)))
        .add_subclass(PyIdent)
}

#[pyo3::pymethods]
impl PyIdent {
    #[new]
    #[pyo3(signature = (*parts))]
    fn __new__(parts: alias::ArgsType) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let ident = match parts.len() {
            0 => return Err(parse_err(ParseIdentError::Empty)),
            1 => match extract_arg(parts.get_item(0)?.as_borrowed())? {
                Extracted::Ident(ident) => ident,
                Extracted::Part(part) => Ident::from_one(part),
            },
            _ => {
                let mut acc = SmallVec::<[IdentPart; 4]>::new();
                for item in parts.iter() {
                    match extract_arg(item.as_borrowed())? {
                        Extracted::Ident(ident) => acc.extend(ident.into_parts()),
                        Extracted::Part(part) => acc.push(part),
                    }
                }
                Ident::from_parts(acc).map_err(parse_err)?
            }
        };
        Ok(wrap_ident(ident))
    }

    fn __len__(slf: pyo3::PyRef<'_, Self>) -> usize {
        ident_inner(&slf).len()
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
        slf: pyo3::PyRef<'py, Self>,
        py: pyo3::Python<'py>,
        index: utils::IndexOrSlice,
    ) -> pyo3::PyResult<alias::BoundObject<'py>> {
        let parts = ident_inner(&slf).parts();
        let len = parts.len() as isize;

        match index {
            utils::IndexOrSlice::Index(i) => {
                let idx = if i < 0 { i + len } else { i };
                if idx < 0 || idx >= len {
                    return rapidquery_macros::pyerr!(
                        Err(PyIndexError),
                        "Ident index out of range"
                    );
                }
                ident_part_to_pyobject(py, &parts[idx as usize])
            }
            utils::IndexOrSlice::Slice(slice) => {
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

                let inner = Ident::from_parts(selected).map_err(parse_err)?;
                super::PyExpr(super::ExprEnum::Ident(inner)).into_subclasses(py)
            }
        }
    }

    fn __iter__<'py>(
        slf: pyo3::PyRef<'py, Self>,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::types::PyIterator>> {
        let items = ident_inner(&slf)
            .parts()
            .iter()
            .map(|part| ident_part_to_pyobject(py, part))
            .collect::<pyo3::PyResult<Vec<_>>>()?;
        let list = pyo3::types::PyList::new(py, items)?;
        list.try_iter()
    }

    /// Returns the parts as a list (`str`, `Unquote`, or `Asterisk`).
    fn to_list(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<Vec<alias::PyObject>> {
        ident_inner(&slf)
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
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
        dialect: crate::dialect::py::PySqlDialect,
    ) -> pyo3::PyResult<String> {
        write_sql(py, ident_inner(&slf), &dialect.0)
            .map_err(|e| rapidquery_macros::pyerr!(PyRuntimeError, "{}", e))
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
#[rapidquery_macros::pyclass(
    "Unquote",
    skip_from_py_object,
    eq,
    hash,
    module = "rapidquery._core.expr"
)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyUnquote(pub arcstr::ArcStr);

#[pyo3::pymethods]
impl PyUnquote {
    #[new]
    #[pyo3(signature = (name, /))]
    fn __new__(name: &str) -> pyo3::PyResult<Self> {
        IdentPart::unquoted(name).map_err(parse_err)?;
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
#[rapidquery_macros::pyclass(
    "Asterisk",
    skip_from_py_object,
    eq,
    hash,
    module = "rapidquery._core.expr"
)]
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn ident_from(parts: impl IntoIterator<Item = IdentPart>) -> Ident {
        Ident::from_parts(parts).unwrap()
    }

    fn quote(s: &str) -> IdentPart {
        IdentPart::quoted(s).unwrap()
    }

    #[test]
    fn from_str_splits_on_dots() {
        let ident = Ident::from_str("public.users.id").unwrap();
        assert_eq!(
            ident.parts(),
            &[quote("public"), quote("users"), quote("id"),]
        );
    }

    #[test]
    fn from_str_bare_star_is_asterisk() {
        let ident = Ident::from_str("*").unwrap();
        assert_eq!(ident.parts(), &[IdentPart::Asterisk]);
    }

    #[test]
    fn from_str_table_star_keeps_asterisk_last() {
        let ident = Ident::from_str("t.*").unwrap();
        assert_eq!(ident.parts(), &[quote("t"), IdentPart::Asterisk]);
    }

    #[test]
    fn from_str_mid_star_is_demoted_to_quoted() {
        let ident = Ident::from_str("*.id").unwrap();
        assert_eq!(ident.parts(), &[quote("*"), quote("id")]);
    }

    #[test]
    fn from_str_rejects_empty_and_empty_chunks() {
        assert_eq!(Ident::from_str("").unwrap_err(), ParseIdentError::Empty);
        assert_eq!(
            Ident::from_str("a.").unwrap_err(),
            ParseIdentError::EmptyChunk
        );
        assert_eq!(
            Ident::from_str(".a").unwrap_err(),
            ParseIdentError::EmptyChunk
        );
        assert_eq!(
            Ident::from_str("a..b").unwrap_err(),
            ParseIdentError::EmptyChunk
        );
    }

    #[test]
    fn from_str_rejects_too_many_parts() {
        let ten = (0..Ident::MAX_PARTS)
            .map(|i| format!("p{i}"))
            .collect::<Vec<_>>()
            .join(".");
        assert!(Ident::from_str(&ten).is_ok());

        let eleven = format!("{ten}.extra");
        assert_eq!(
            Ident::from_str(&eleven).unwrap_err(),
            ParseIdentError::TooManyParts
        );
    }

    #[test]
    fn from_parts_rejects_empty() {
        assert_eq!(Ident::from_parts([]).unwrap_err(), ParseIdentError::Empty);
    }

    #[test]
    fn from_parts_rejects_empty_quoted_part() {
        assert_eq!(
            Ident::from_parts([IdentPart::Quote(ArcStr::from(""))]).unwrap_err(),
            ParseIdentError::EmptyChunk
        );
    }

    #[test]
    fn from_parts_rejects_asterisk_not_last() {
        assert_eq!(
            Ident::from_parts([IdentPart::Asterisk, quote("id")]).unwrap_err(),
            ParseIdentError::AsteriskNotLast
        );
    }

    #[test]
    fn unquoted_rejects_invalid() {
        for bad in ["", "*", "a;b", "a.b", "1id", "a-b", "a b", "café"] {
            let err = IdentPart::unquoted(bad).unwrap_err();
            assert!(
                err == ParseIdentError::InvalidUnquoted || err == ParseIdentError::EmptyChunk,
                "{bad:?} -> {err:?}"
            );
        }
    }

    #[test]
    fn unquoted_accepts_simple_ascii_names() {
        for ok in ["Users", "id", "_x", "a1", "created_at"] {
            let part = IdentPart::unquoted(ok).unwrap();
            match part {
                IdentPart::Unquote(n) => assert_eq!(n.as_str(), ok),
                _ => panic!("expected Unquote"),
            }
        }
    }

    #[test]
    fn quoted_star_is_not_asterisk() {
        let part = quote("*");
        assert_ne!(part, IdentPart::Asterisk);
        match part {
            IdentPart::Quote(n) => assert_eq!(n.as_str(), "*"),
            _ => panic!("expected Quote"),
        }
    }

    #[test]
    fn quote_unquote_asterisk_are_not_equal() {
        let q = quote("id");
        let u = IdentPart::unquoted("id").unwrap();
        assert_ne!(q, u);
        assert_ne!(q, IdentPart::Asterisk);
        assert_ne!(u, IdentPart::Asterisk);
        assert_ne!(ident_from([q.clone()]), ident_from([u.clone()]));
    }

    #[test]
    fn from_one_is_single_part() {
        let ident = Ident::from_one(quote("id"));
        assert_eq!(ident.len(), 1);
        assert_eq!(ident.parts(), &[quote("id")]);
    }

    #[test]
    fn parse_ident_error_display_is_not_todo() {
        let msg = ParseIdentError::Empty.to_string();
        assert!(!msg.is_empty());
        assert!(!msg.contains("todo"));
    }

    #[test]
    fn prepare_uses_dialect_delimiters() {
        fn render(ident: &Ident, dialect: SqlDialect) -> String {
            let mut buf = String::new();
            for (i, part) in ident.parts().iter().enumerate() {
                if i > 0 {
                    buf.push('.');
                }
                match part {
                    IdentPart::Quote(n) => {
                        dialect.quote_identifier(&mut buf, n.as_str()).unwrap();
                    }
                    IdentPart::Unquote(n) => buf.push_str(n.as_str()),
                    IdentPart::Asterisk => buf.push('*'),
                }
            }
            buf
        }

        let ident = ident_from([quote("id")]);
        assert_eq!(render(&ident, SqlDialect::PostgreSql), "\"id\"");
        assert_eq!(render(&ident, SqlDialect::MySql), "`id`");
        assert_eq!(render(&ident, SqlDialect::MsSql), "[id]");
        assert_eq!(render(&ident, SqlDialect::Teradata), "'id'");

        let ident = ident_from([quote("a\"b")]);
        assert_eq!(render(&ident, SqlDialect::PostgreSql), "\"a\"\"b\"");

        let ident = ident_from([quote("a`b")]);
        assert_eq!(render(&ident, SqlDialect::MySql), "`a``b`");

        let ident = ident_from([quote("a]b")]);
        assert_eq!(render(&ident, SqlDialect::MsSql), "[a]]b]");

        let ident = ident_from([quote("a'b")]);
        assert_eq!(render(&ident, SqlDialect::Teradata), "'a''b'");

        let ident = ident_from([quote("a; DROP TABLE t--")]);
        assert_eq!(
            render(&ident, SqlDialect::PostgreSql),
            "\"a; DROP TABLE t--\""
        );

        let ident = ident_from([
            quote("public"),
            IdentPart::unquoted("Users").unwrap(),
            quote("id"),
        ]);
        assert_eq!(
            render(&ident, SqlDialect::PostgreSql),
            "\"public\".Users.\"id\""
        );

        let ident = ident_from([quote("t"), IdentPart::Asterisk]);
        assert_eq!(render(&ident, SqlDialect::MySql), "`t`.*");
    }
}
