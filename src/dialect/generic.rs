use crate::dialect::convert::parse_positional_parameter;

use super::base::PySqlDialect;
use super::convert;
use super::inner::Dialect;

/// Frozen SQL dialect whose quoting and literal rules are set at construction.
#[rapidquery_macros::pyclass(
    "GenericDialect",
    extends = PySqlDialect,
    module = "rapidquery._core.dialect"
)]
pub struct PyGenericDialect;

#[pyo3::pymethods]
impl PyGenericDialect {
    #[new]
    #[pyo3(signature = (
        *,
        positional_paremeter,
        bit_format,
        supports_array,
        supports_inet,
        supports_mac_address,
        null_treatment_syntax,
        like_any_syntax,
        bool_format,
        identifier_format,
        identifier_escape,
        string_format,
        string_escape,
        bytes_format,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn __new__(
        positional_paremeter: &str,
        bit_format: Option<&str>,
        supports_array: bool,
        supports_inet: bool,
        supports_mac_address: bool,
        null_treatment_syntax: &str,
        like_any_syntax: &str,
        bool_format: &str,
        identifier_format: &str,
        identifier_escape: &str,
        string_format: &str,
        string_escape: &str,
        bytes_format: &str,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let bit_format = match bit_format {
            None => None,
            Some(s) => Some(convert::parse_field(
                "bit_format",
                s,
                convert::parse_bit_format,
                convert::BIT_FORMATS,
            )?),
        };

        let positional_parameter =
            parse_positional_parameter(positional_paremeter).ok_or_else(|| {
                rapidquery_macros::pyerr!(
                    PyValueError,
                    "invalid positional_parameter: {:?}. expected \"X\" or \"XNN\" formats which X is placeholder character",
                    positional_paremeter
                )
            })?;

        let inner = Dialect {
            positional_parameter,
            bit_format,
            supports_array,
            supports_inet,
            supports_mac_address,
            null_treatment_syntax: convert::parse_field(
                "null_treatment_syntax",
                null_treatment_syntax,
                convert::parse_null_treatment_syntax,
                convert::NULL_TREATMENT_SYNTAXES,
            )?,
            like_any_syntax: convert::parse_field(
                "like_any_syntax",
                like_any_syntax,
                convert::parse_like_any_syntax,
                convert::LIKE_ANY_SYNTAXES,
            )?,
            bool_format: convert::parse_field(
                "bool_format",
                bool_format,
                convert::parse_bool_format,
                convert::BOOL_FORMATS,
            )?,
            identifier_format: convert::parse_field(
                "identifier_format",
                identifier_format,
                convert::parse_identifier_format,
                convert::IDENTIFIER_FORMATS,
            )?,
            identifier_escape: convert::parse_field(
                "identifier_escape",
                identifier_escape,
                convert::parse_identifier_escape,
                convert::IDENTIFIER_ESCAPES,
            )?,
            string_format: convert::parse_field(
                "string_format",
                string_format,
                convert::parse_string_format,
                convert::STRING_FORMATS,
            )?,
            string_escape: convert::parse_field(
                "string_escape",
                string_escape,
                convert::parse_string_escape,
                convert::STRING_ESCAPES,
            )?,
            bytes_format: convert::parse_field(
                "bytes_format",
                bytes_format,
                convert::parse_bytes_format,
                convert::BYTES_FORMATS,
            )?,
        };
        Ok(pyo3::PyClassInitializer::from(PySqlDialect(inner)).add_subclass(Self))
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let d = &slf.as_super().0;
        let bit = match d.bit_format {
            None => String::from("None"),
            Some(fmt) => format!("{:?}", convert::bit_format_name(fmt)),
        };
        format!(
            "GenericDialect(placeholder_sigil={:?}, bit_format={bit}, supports_array={}, supports_inet={}, supports_mac_address={}, null_treatment_syntax={:?}, like_any_syntax={:?}, bool_format={:?}, identifier_format={:?}, identifier_escape={:?}, string_format={:?}, string_escape={:?}, bytes_format={:?})",
            convert::positional_parameter_name(d.positional_parameter),
            py_bool(d.supports_array),
            py_bool(d.supports_inet),
            py_bool(d.supports_mac_address),
            convert::null_treatment_syntax_name(d.null_treatment_syntax),
            convert::like_any_syntax_name(d.like_any_syntax),
            convert::bool_format_name(d.bool_format),
            convert::identifier_format_name(d.identifier_format),
            convert::identifier_escape_name(d.identifier_escape),
            convert::string_format_name(d.string_format),
            convert::string_escape_name(d.string_escape),
            convert::bytes_format_name(d.bytes_format),
        )
    }
}

#[inline]
fn py_bool(v: bool) -> &'static str {
    if v { "True" } else { "False" }
}
