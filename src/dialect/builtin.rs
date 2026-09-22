use super::inner::*;

macro_rules! impl_builtin_dialects {
    (
        $(
            $(#[$docs:meta])*
            $name:ident = $pyname:literal {
                $inner:expr
            }
        )+
    ) => {
        $(
            $(#[$docs])*
            #[rapidquery_macros::pyclass($pyname, extends = super::base::PySqlDialect, module = "rapidquery._core.dialect")]
            pub struct $name;

            impl $name {
                pub const CONFIG: Dialect = $inner;
            }

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[inline]
                fn __new__() -> pyo3::PyClassInitializer<Self> {
                    pyo3::PyClassInitializer::from(super::base::PySqlDialect(Self::CONFIG))
                        .add_subclass(Self)
                }
            }
        )+
    };
}

impl_builtin_dialects!(
    /// BigQuery SQL dialect.
    PyBigQuery = "BigQuery" {
        Dialect {
            placeholder_sigil: '@',
            bit_format: None,
            supports_array: true,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::InsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::SameAsString,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::PrefixBigBSlashX,
        }
    }

    /// Databricks SQL dialect.
    PyDatabricks = "Databricks" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    /// DuckDB SQL dialect.
    PyDuckDb = "DuckDb" {
        Dialect {
            placeholder_sigil: '$',
            bit_format: Some(BitFormat::PrefixBigB),
            supports_array: true,
            supports_inet: true,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::InsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    /// Hive SQL dialect.
    PyHive = "Hive" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicode,
            bytes_format: BytesFormat::Unhex,
        }
    }

    /// Microsoft SQL Server dialect.
    PyMsSql = "MsSql" {
        Dialect {
            placeholder_sigil: '@',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::SquareBracket,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::PrefixBigN,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::PrefixZeroX,
        }
    }

    /// MySQL dialect.
    PyMySql = "MySql" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: Some(BitFormat::PrefixSmallB),
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashMysql,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    /// Oracle SQL dialect.
    PyOracle = "Oracle" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::HexToRaw,
        }
    }

    /// PostgreSQL dialect.
    PyPostgreSql = "PostgreSql" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: Some(BitFormat::PrefixBigB),
            supports_array: true,
            supports_inet: true,
            supports_mac_address: true,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::ArrayConstructor,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::PrefixBigE,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::SlashXBytea,
        }
    }

    /// Amazon Redshift dialect.
    PyRedshift = "Redshift" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::ToVarbyte,
        }
    }

    /// Snowflake SQL dialect.
    PySnowflake = "Snowflake" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHexZero,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    /// Apache Spark SQL dialect.
    PySpark = "Spark" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    /// SQLite dialect.
    PySqlite = "Sqlite" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::PrefixBigX,
        }
    }

    /// Teradata SQL dialect.
    PyTeradata = "Teradata" {
        Dialect {
            placeholder_sigil: ':',
            bit_format: Some(BitFormat::PostfixBigB),
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: NullTreatmentSyntax::OutsideParens,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::SingleQuote,
            identifier_escape: IdentifierEscape::Delimiters,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::PostfixXbv,
        }
    }
);
