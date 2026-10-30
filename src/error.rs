use crate::expr::enum_::IdentParseError;
use crate::internal::template::TemplateParseError;

// Used by rapidquery_macros::pyo3_constructor
pub(crate) fn format_options(names: &[&str], values: &[u8]) -> String {
    let parts: Vec<String> = names
        .iter()
        .zip(values)
        .map(|(name, value)| format!("{name} ({value})"))
        .collect();

    match parts.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [left, right] => format!("{left} or {right}"),
        [head @ .., last] => format!("{}, or {last}", head.join(", ")),
    }
}

/// Project error. Convert it with `?` from a function that returns `Result<T, Error>`.
#[derive(Debug, derive_more::Display)]
pub enum Error {
    /// A Python exception that already occurred (`PyErr::fetch`). Re-raised as-is.
    #[display("{_0}")]
    Python(pyo3::PyErr),

    /// `expected` is the requirement. `got` is the Python type name that arrived.
    #[display("{expected}, got {got}")]
    Expected { expected: &'static str, got: String },

    /// `feature` says what this dialect cannot do. `fix` says which field to change.
    #[display("{feature} (hint: {fix})")]
    Unsupported { feature: &'static str, fix: &'static str },

    #[display("invalid template for `{field}`: {reason}")]
    InvalidTemplate {
        field: &'static str,
        reason: TemplateParseError,
    },

    #[display("{_0}")]
    Ident(IdentParseError),

    #[display("string literal contains a NUL byte; a SQL string literal cannot carry NUL")]
    NulInString,

    #[display("expected a string of '0' and '1' for bit serialization, got {value:?}")]
    InvalidBitString { value: String },

    #[display("cannot serialize non-finite float {value} as a numeric literal")]
    NonFiniteFloat { value: f64 },

    #[display("invalid uuid str: {detail}")]
    InvalidUuid { detail: String },

    #[display("invalid inet address {text:?}: {detail}")]
    InvalidInet { text: String, detail: String },

    #[display("invalid mac address {text:?}: {detail}")]
    InvalidMac { text: String, detail: String },

    #[display("LIKE ANY pattern list is empty; pass at least one pattern")]
    EmptyLikeAny,

    #[display("unknown unary operator {operator:?}; expected NOT, +, -, or ~")]
    UnknownUnaryOperator { operator: String },

    #[display("unknown IS kind {kind:?}; expected NULL, TRUE, FALSE, or UNKNOWN")]
    UnknownIsKind { kind: String },

    #[display("unknown cast kind {kind:?}; expected CAST, TRY_CAST, or ::")]
    UnknownCastKind { kind: String },

    #[display("unknown TRIM where {kind:?}; expected BOTH, LEADING, or TRAILING")]
    UnknownTrimWhere { kind: String },

    #[display("invalid value {value} for `{field}`; expected {expected}")]
    InvalidEnumOptionField {
        field: &'static str,
        value: u8,
        expected: String,
    },

    #[display("invalid bits {value:#b} for `{field}`; expected a combination of {expected}")]
    InvalidBitOptionField {
        field: &'static str,
        value: u8,
        expected: String,
    },

    #[display("invalid value {value:?} for behavior; expected CASCADE or RESTRICT")]
    InvalidSchemaBehavior { value: String },

    #[display("custom expression has {placeholders} placeholders but {arguments} arguments")]
    CustomPlaceholderCount { placeholders: usize, arguments: usize },

    #[display(
        "collation must be a single quoted or unquoted name when operator.collate is OPT_OPERATOR_COLLATE_FUNCTION_STRING"
    )]
    CollationNotSingleName,

    #[display("identifier index {index} is out of range (length {len})")]
    IdentIndex { index: isize, len: usize },

    #[display("integer is out of range for datetime serialization")]
    DatetimeOutOfRange,

    #[display("timestamp {value} is out of range for datetime serialization")]
    InvalidTimestamp { value: String },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ident(err) => Some(err),
            Self::InvalidTemplate { reason, .. } => Some(reason),
            _ => None,
        }
    }
}

impl From<IdentParseError> for Error {
    fn from(err: IdentParseError) -> Self {
        Self::Ident(err)
    }
}

impl From<TemplateParseError> for Error {
    fn from(reason: TemplateParseError) -> Self {
        Self::InvalidTemplate {
            field: "template",
            reason,
        }
    }
}

impl From<pyo3::PyErr> for Error {
    fn from(err: pyo3::PyErr) -> Self {
        Self::Python(err)
    }
}

impl From<pyo3::CastIntoError<'_>> for Error {
    fn from(err: pyo3::CastIntoError<'_>) -> Self {
        pyo3::PyErr::from(err).into()
    }
}

impl From<Error> for pyo3::PyErr {
    fn from(err: Error) -> Self {
        match err {
            Error::Python(err) => err,
            err @ Error::Expected { .. } => pyo3::exceptions::PyTypeError::new_err(err.to_string()),
            err @ Error::Unsupported { .. } => pyo3::exceptions::PyNotImplementedError::new_err(err.to_string()),
            err @ Error::IdentIndex { .. } => pyo3::exceptions::PyIndexError::new_err(err.to_string()),
            err @ (Error::DatetimeOutOfRange | Error::InvalidTimestamp { .. }) => {
                pyo3::exceptions::PyOverflowError::new_err(err.to_string())
            }
            err => pyo3::exceptions::PyValueError::new_err(err.to_string()),
        }
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;
