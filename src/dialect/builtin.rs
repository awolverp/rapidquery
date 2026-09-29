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
            array_format: Some(ArrayFormat::Constructor),
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Backslash,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::PrefixBigBSlashX,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::TypedKeyword,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::JsonKeyword,
            uuid_format: UuidFormat::Quoted,
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
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
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
            array_format: Some(ArrayFormat::List),
            inet_format: Some(InetFormat::Cast),
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::SlashXBlob,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::TypedKeyword,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::UuidKeyword,
            function_filter_syntax: Some(FunctionFilterSyntax::Filter),
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: true,
            rlike_expression_options: Some(RLikeExpressionOptions::RegexpMatches),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::XORFunction),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Row),
            parenthesize_in_between: false,
        }
    }

    /// Hive SQL dialect.
    PyHive = "Hive" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            array_format: Some(ArrayFormat::Function),
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicode,
            bytes_format: BytesFormat::Unhex,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
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
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::SquareBracket,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::PrefixBigN,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixZeroX,
            date_format: TemporalLiteral::Quoted,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::Quoted,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
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
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashMysql,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: TemporalLiteral::Quoted,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::Quoted,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLike,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::ConcatFunction,
            bit_xor_syntax: Some(BitXorSyntax::Caret),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::NullSafeEqual),
            tuple_syntax: Some(TupleSyntax::RowRejectEmpty),
            parenthesize_in_between: false,
        }
    }

    /// Oracle SQL dialect.
    PyOracle = "Oracle" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered(':'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::HexToRaw,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::UseLikeLower,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::RegexpLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: None,
            modulo_syntax: ModuloSyntax::Function,
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
            array_format: Some(ArrayFormat::Constructor),
            inet_format: Some(InetFormat::Cast),
            mac_format: Some(MacFormat::Cast),
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::ArrayConstructor,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::PrefixBigE,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::SlashXBytea,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::TypedKeyword,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::CastJsonb,
            uuid_format: UuidFormat::CastUuid,
            function_filter_syntax: Some(FunctionFilterSyntax::FilterWhere),
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: true,
            rlike_expression_options: Some(RLikeExpressionOptions::Tilde),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::Hash),
            modulo_syntax: ModuloSyntax::Percent,
            is_distinct_from_syntax: Some(IsDistinctFromSyntax::Standard),
            tuple_syntax: Some(TupleSyntax::Row),
            parenthesize_in_between: false,
        }
    }

    /// Amazon Redshift dialect.
    PyRedshift = "Redshift" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::ToVarbyte,
            date_format: TemporalLiteral::Quoted,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::Quoted,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
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
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHexZero,
            bytes_format: BytesFormat::ToBinary,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::ParseJson,
            uuid_format: UuidFormat::Quoted,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::Supported,
            supports_similar_to: false,
            rlike_expression_options: Some(RLikeExpressionOptions::InfixRLike),
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::BitXORFunction),
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
            array_format: Some(ArrayFormat::Function),
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
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
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: TemporalLiteral::Quoted,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::Quoted,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
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
            array_format: None,
            inet_format: None,
            mac_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PostfixXbv,
            date_format: TemporalLiteral::TypedKeyword,
            time_format: TemporalLiteral::Quoted,
            timestamp_format: TemporalLiteral::TypedKeyword,
            json_format: JsonFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            function_filter_syntax: None,
            ilike_expression_options: ILikeExpressionOptions::UseLikeLower,
            supports_similar_to: false,
            rlike_expression_options: None,
            string_concat_syntax: StringConcatSyntax::Pipes,
            bit_xor_syntax: Some(BitXorSyntax::BitXORFunction),
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
            Some(XORFunction),
            Percent,
            Some(Standard),
            Some(Row),
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
            ConcatFunction,
            Some(Caret),
            Percent,
            Some(NullSafeEqual),
            Some(RowRejectEmpty),
            false,
        );
        expect(PyOracle::CONFIG, Pipes, None, Function, None, None, false);
        expect(
            PyPostgreSql::CONFIG,
            Pipes,
            Some(Hash),
            Percent,
            Some(Standard),
            Some(Row),
            false,
        );
        expect(PyRedshift::CONFIG, Pipes, None, Percent, None, None, false);
        expect(
            PySnowflake::CONFIG,
            Pipes,
            Some(BitXORFunction),
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
            Some(BitXORFunction),
            Infix,
            None,
            None,
            false,
        );
    }
}
