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
#[derive(Debug, strum::Display)]
pub enum Error {
    /// A Python exception that already occurred (`PyErr::fetch`). Re-raised as-is.
    #[strum(to_string = "{0}")]
    Python(pyo3::PyErr),

    /// `expected` is the requirement. `got` is the Python type name that arrived.
    #[strum(to_string = "{expected}, got {got}")]
    Expected { expected: &'static str, got: String },

    #[strum(to_string = "{feature} is not supported by this dialect")]
    Unsupported { feature: &'static str },

    #[strum(to_string = "invalid template for `{field}`: {reason}")]
    InvalidTemplate {
        field: &'static str,
        reason: TemplateParseError,
    },

    #[strum(to_string = "{0}")]
    Ident(IdentParseError),

    #[strum(to_string = "string literal contains a NUL byte; a SQL string literal cannot carry NUL")]
    NulInString,

    #[strum(to_string = "expected a string of '0' and '1' for bit serialization, got {value:?}")]
    InvalidBitString { value: String },

    #[strum(to_string = "cannot serialize non-finite float {value} as a numeric literal")]
    NonFiniteFloat { value: f64 },

    #[strum(to_string = "invalid uuid str: {detail}")]
    InvalidUuid { detail: String },

    #[strum(to_string = "invalid inet address {text:?}: {detail}")]
    InvalidInet { text: String, detail: String },

    #[strum(to_string = "invalid mac address {text:?}: {detail}")]
    InvalidMac { text: String, detail: String },

    #[strum(to_string = "LIKE ANY pattern list is empty; pass at least one pattern")]
    EmptyLikeAny,

    #[strum(to_string = "unknown unary operator {operator:?}; expected NOT, +, -, or ~")]
    UnknownUnaryOperator { operator: String },

    #[strum(to_string = "unknown IS kind {kind:?}; expected NULL, TRUE, FALSE, or UNKNOWN")]
    UnknownIsKind { kind: String },

    #[strum(to_string = "unknown cast kind {kind:?}; expected CAST, TRY_CAST, or ::")]
    UnknownCastKind { kind: String },

    #[strum(to_string = "unknown TRIM where {kind:?}; expected BOTH, LEADING, or TRAILING")]
    UnknownTrimWhere { kind: String },

    #[strum(to_string = "invalid value {value} for `{field}`; expected {expected}")]
    InvalidEnumOptionField {
        field: &'static str,
        value: u8,
        expected: String,
    },

    #[strum(to_string = "invalid bits {value:#b} for `{field}`; expected a combination of {expected}")]
    InvalidBitOptionField {
        field: &'static str,
        value: u8,
        expected: String,
    },

    #[strum(to_string = "invalid value {value:?} for behavior; expected CASCADE or RESTRICT")]
    InvalidSchemaBehavior { value: String },

    #[strum(to_string = "invalid value {value:?} for order; expected ASC or DESC")]
    InvalidIndexOrder { value: String },

    #[strum(to_string = "invalid value {value:?} for behavior; expected CASCADE or RESTRICT")]
    InvalidIndexBehavior { value: String },

    #[strum(to_string = "CREATE INDEX requires at least one column")]
    EmptyIndexColumns,

    #[strum(to_string = "index name is empty")]
    EmptyIndexName,

    #[strum(to_string = "index type is empty")]
    EmptyIndexType,

    #[strum(to_string = "INCLUDE column name is empty")]
    EmptyIndexInclude,

    #[strum(to_string = "table_name is required in DROP INDEX for this dialect")]
    MissingDropIndexTable,

    #[strum(to_string = "custom expression has {placeholders} placeholders but {arguments} arguments")]
    CustomPlaceholderCount { placeholders: usize, arguments: usize },

    #[strum(
        to_string = "collation must be a single quoted or unquoted name when operator.collate is OPT_OPERATOR_COLLATE_FUNCTION_STRING"
    )]
    CollationNotSingleName,

    #[strum(to_string = "identifier index {index} is out of range (length {len})")]
    IdentIndex { index: isize, len: usize },

    #[strum(to_string = "integer is out of range for datetime serialization")]
    DatetimeOutOfRange,

    #[strum(to_string = "timestamp {value} is out of range for datetime serialization")]
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
