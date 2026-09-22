//! Python string names for [`super::inner`] format enums.
//!
//! Conversion lives here so the enums themselves stay data-only.

use super::inner::{
    BitFormat, BoolFormat, BytesFormat, IdentifierEscape, IdentifierFormat, LikeAnySyntax,
    NullTreatmentSyntax, StringEscape, StringFormat,
};

macro_rules! named {
    ($ty:ty, $to_str:ident, $parse:ident { $($v:ident),+ $(,)? }) => {
        #[inline]
        pub(super) fn $to_str(v: $ty) -> &'static str {
            match v {
                $(<$ty>::$v => stringify!($v),)+
            }
        }

        #[inline]
        pub(super) fn $parse(s: &str) -> Option<$ty> {
            $(
                if s.eq_ignore_ascii_case(stringify!($v)) {
                    return Some(<$ty>::$v);
                }
            )+
            None
        }
    };
}

named!(
    BitFormat,
    bit_format_name,
    parse_bit_format {
        PrefixBigB,
        PrefixSmallB,
        PostfixBigB
    }
);
named!(
    NullTreatmentSyntax,
    null_treatment_syntax_name,
    parse_null_treatment_syntax {
        InsideParens,
        OutsideParens
    }
);
named!(
    LikeAnySyntax,
    like_any_syntax_name,
    parse_like_any_syntax {
        ArrayConstructor,
        PatternList,
        OrExpansion
    }
);
named!(
    BoolFormat,
    bool_format_name,
    parse_bool_format { Keyword, Int }
);
named!(
    IdentifierFormat,
    identifier_format_name,
    parse_identifier_format {
        SingleQuote,
        DoubleQuote,
        Backtick,
        SquareBracket
    }
);
named!(
    IdentifierEscape,
    identifier_escape_name,
    parse_identifier_escape {
        Delimiters,
        SameAsString
    }
);
named!(
    StringFormat,
    string_format_name,
    parse_string_format {
        SingleQuote,
        PrefixBigE,
        PrefixBigN
    }
);
named!(
    StringEscape,
    string_escape_name,
    parse_string_escape {
        Quote,
        QuoteStripNul,
        BackslashHex,
        BackslashMysql,
        BackslashUnicode,
        BackslashHexZero,
        BackslashUnicodeQuote
    }
);
named!(
    BytesFormat,
    bytes_format_name,
    parse_bytes_format {
        PrefixBigX,
        PrefixZeroX,
        SlashXBytea,
        PrefixBigBSlashX,
        HexToRaw,
        Unhex,
        ToVarbyte,
        PostfixXbv
    }
);

pub(super) const BIT_FORMATS: &str = "PrefixBigB, PrefixSmallB, PostfixBigB";
pub(super) const NULL_TREATMENT_SYNTAXES: &str = "InsideParens, OutsideParens";
pub(super) const LIKE_ANY_SYNTAXES: &str = "ArrayConstructor, PatternList, OrExpansion";
pub(super) const BOOL_FORMATS: &str = "Keyword, Int";
pub(super) const IDENTIFIER_FORMATS: &str = "SingleQuote, DoubleQuote, Backtick, SquareBracket";
pub(super) const IDENTIFIER_ESCAPES: &str = "Delimiters, SameAsString";
pub(super) const STRING_FORMATS: &str = "SingleQuote, PrefixBigE, PrefixBigN";
pub(super) const STRING_ESCAPES: &str = "Quote, QuoteStripNul, BackslashHex, BackslashMysql, BackslashUnicode, BackslashHexZero, BackslashUnicodeQuote";
pub(super) const BYTES_FORMATS: &str = "PrefixBigX, PrefixZeroX, SlashXBytea, PrefixBigBSlashX, HexToRaw, Unhex, ToVarbyte, PostfixXbv";

#[inline]
pub(super) fn parse_field<T>(
    field: &str,
    value: &str,
    parse: impl Fn(&str) -> Option<T>,
    expected: &str,
) -> pyo3::PyResult<T> {
    match parse(value) {
        Some(v) => Ok(v),
        None => rapidquery_macros::pyerr!(
            Err(PyValueError),
            "unknown {} {:?}; expected {}",
            field,
            value,
            expected
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_is_case_insensitive_and_round_trips() {
        assert_eq!(parse_bit_format("prefixbigb"), Some(BitFormat::PrefixBigB));
        assert_eq!(bit_format_name(BitFormat::PrefixBigB), "PrefixBigB");
        assert_eq!(parse_bool_format("INT"), Some(BoolFormat::Int));
        assert!(parse_bytes_format("nope").is_none());
    }
}
