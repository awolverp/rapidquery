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
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            supports_array: true,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Backslash,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::PrefixBigBSlashX,
            function_filter_syntax: Some(FunctionFilterSyntax::InsideWhere),
            ilike_expression_options: ILikeExpressionOptions::UseLikeLower,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::RegexpContains),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: None,
            parenthesize_in_between: false,
        }
    }

    /// Databricks SQL dialect.
    PyDatabricks = "Databricks" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Struct),
            parenthesize_in_between: false,
        }
    }

    /// DuckDB SQL dialect.
    PyDuckDb = "DuckDb" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: Some(BitFormat::PrefixBigB),
            supports_array: true,
            supports_inet: true,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixBigX,
            function_filter_syntax: Some(FunctionFilterSyntax::Filter),
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: true,
            rlike_expression_options: Some(RLikeExpressionOptions::RegexpMatches),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::XorCall),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Row { empty_ok: true }),
            parenthesize_in_between: false,
        }
    }

    /// Hive SQL dialect.
    PyHive = "Hive" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicode,
            bytes_format: BytesFormat::Unhex,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLikeLower,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::NullSafeEqual),
            tuple_syntax: Some(TupleSyntax::Struct),
            parenthesize_in_between: false,
        }
    }

    /// Microsoft SQL Server dialect.
    PyMsSql = "MsSql" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::SquareBracket,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::PrefixBigN,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::PrefixZeroX,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLike,
            supports_similar_to: false,
            rlike_expression_options: None,
            string_concat_syntax: StringConcatSyntax::Plus,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: None,
            parenthesize_in_between: true,
        }
    }

    /// MySQL dialect.
    PyMySql = "MySql" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: Some(BitFormat::PrefixSmallB),
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashMysql,
            bytes_format: BytesFormat::PrefixBigX,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLike,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::Concat,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::NullSafeEqual),
            tuple_syntax: Some(TupleSyntax::Row { empty_ok: false }),
            parenthesize_in_between: false,
        }
    }

    /// Oracle SQL dialect.
    PyOracle = "Oracle" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered(':'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::HexToRaw,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::UseLikeLower,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::RegexpLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: None,
            modulo_syntax: ModuloSyntax::Call,
            is_distinct_from_syntax: None,
            tuple_syntax: None,
            parenthesize_in_between: false,
        }
    }

    /// PostgreSQL dialect.
    PyPostgreSql = "PostgreSql" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: Some(BitFormat::PrefixBigB),
            supports_array: true,
            supports_inet: true,
            supports_mac_address: true,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::ArrayConstructor,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::PrefixBigE,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::SlashXBytea,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: true,
            rlike_expression_options: Some(RLikeExpressionOptions::Tilde),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::Hash),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Row { empty_ok: true }),
            parenthesize_in_between: false,
        }
    }

    /// Amazon Redshift dialect.
    PyRedshift = "Redshift" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::ToVarbyte,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: true,
            rlike_expression_options: Some(RLikeExpressionOptions::Tilde),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: None,
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: None,
            tuple_syntax: None,
            parenthesize_in_between: false,
        }
    }

    /// Snowflake SQL dialect.
    PySnowflake = "Snowflake" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered(':'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHexZero,
            bytes_format: BytesFormat::PrefixBigX,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::BitXorCall),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: None,
            parenthesize_in_between: false,
        }
    }

    /// Apache Spark SQL dialect.
    PySpark = "Spark" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Struct),
            parenthesize_in_between: false,
        }
    }

    /// SQLite dialect.
    PySqlite = "Sqlite" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('?'),
            bit_format: None,
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::PrefixBigX,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::UseLike,
            supports_similar_to: false,
            rlike_expression_options: None,
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: None,
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Parentheses),
            parenthesize_in_between: false,
        }
    }

    /// Teradata SQL dialect.
    PyTeradata = "Teradata" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: Some(BitFormat::PostfixBigB),
            supports_array: false,
            supports_inet: false,
            supports_mac_address: false,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::QuoteStripNul,
            bytes_format: BytesFormat::PostfixXbv,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLikeLower,
            supports_similar_to: false,
            rlike_expression_options: None,
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::BitXorCall),
            modulo_syntax: ModuloSyntax::Infix,
            is_distinct_from_syntax: None,
            tuple_syntax: None,
            parenthesize_in_between: false,
        }
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    fn expect(
        dialect: Dialect,
        concat: StringConcatSyntax,
        xor: Option<BitXorSyntax>,
        modulo: ModuloSyntax,
        distinct: Option<IsDistinctFromSyntax>,
        tuple: Option<TupleSyntax>,
        parenthesize_in_between: bool,
    ) {
        assert_eq!(dialect.string_concat_syntax, concat);
        assert_eq!(dialect.bit_xor_syntax, xor);
        assert_eq!(dialect.modulo_syntax, modulo);
        assert_eq!(dialect.is_distinct_from_syntax, distinct);
        assert_eq!(dialect.tuple_syntax, tuple);
        assert_eq!(dialect.parenthesize_in_between, parenthesize_in_between);
    }

    #[test]
    fn operator_syntax_matrix() {
        use BitXorSyntax::*;
        use IsDistinctFromSyntax::*;
        use ModuloSyntax::*;
        use StringConcatSyntax::*;
        use TupleSyntax::*;

        expect(
            PyBigQuery::CONFIG,
            Pipes,
            Some(Caret),
            Percent,
            Some(Standard),
            None,
            false,
        );
        expect(
            PyDatabricks::CONFIG,
            Pipes,
            Some(Caret),
            Percent,
            Some(Standard),
            Some(Struct),
            false,
        );
        expect(
            PyDuckDb::CONFIG,
            Pipes,
            Some(XorCall),
            Percent,
            Some(Standard),
            Some(Row { empty_ok: true }),
            false,
        );
        expect(
            PyHive::CONFIG,
            Pipes,
            Some(Caret),
            Percent,
            Some(NullSafeEqual),
            Some(Struct),
            false,
        );
        expect(
            PyMsSql::CONFIG,
            Plus,
            Some(Caret),
            Percent,
            Some(Standard),
            None,
            true,
        );
        expect(
            PyMySql::CONFIG,
            Concat,
            Some(Caret),
            Percent,
            Some(NullSafeEqual),
            Some(Row { empty_ok: false }),
            false,
        );
        expect(PyOracle::CONFIG, Pipes, None, Call, None, None, false);
        expect(
            PyPostgreSql::CONFIG,
            Pipes,
            Some(Hash),
            Percent,
            Some(Standard),
            Some(Row { empty_ok: true }),
            false,
        );
        expect(PyRedshift::CONFIG, Pipes, None, Percent, None, None, false);
        expect(
            PySnowflake::CONFIG,
            Pipes,
            Some(BitXorCall),
            Percent,
            Some(Standard),
            None,
            false,
        );
        expect(
            PySpark::CONFIG,
            Pipes,
            Some(Caret),
            Percent,
            Some(Standard),
            Some(Struct),
            false,
        );
        expect(
            PySqlite::CONFIG,
            Pipes,
            None,
            Percent,
            Some(Standard),
            Some(Parentheses),
            false,
        );
        expect(
            PyTeradata::CONFIG,
            Pipes,
            Some(BitXorCall),
            Infix,
            None,
            None,
            false,
        );
    }
}
