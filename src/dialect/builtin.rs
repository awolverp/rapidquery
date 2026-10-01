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
            #[rapidquery_macros::pyclass("rapidquery._core.dialect" + $pyname, extends = super::base::PySqlDialect)]
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
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Backslash,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::PrefixBigBSlashX,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Keyword,
            timestamp_format: TimestampFormat::WithTimezone,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::JsonKeyword,
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
            try_cast_syntax: Some(TryCastSyntax::SafeCast),
            supports_double_colon: false,
            trim_syntax: TrimSyntax::Function,
            collate_syntax: Some(CollateSyntax::FunctionString),
            like_escape_syntax: LikeEscapeSyntax::None,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// Databricks SQL dialect.
    PyDatabricks = "Databricks" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::WithTimezone,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: Some(TryCastSyntax::TryCast),
            supports_double_colon: true,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// DuckDB SQL dialect.
    PyDuckDb = "DuckDb" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: Some(BitFormat::PrefixBigB),
            array_format: Some(ArrayFormat::List),
            inet_format: Some(InetFormat::Cast),
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::SlashXBlob,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Keyword,
            timestamp_format: TimestampFormat::Timestamptz,
            uuid_format: UuidFormat::UuidKeyword,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: Some(TryCastSyntax::TryCast),
            supports_double_colon: true,
            trim_syntax: TrimSyntax::Function,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// Hive SQL dialect.
    PyHive = "Hive" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            array_format: Some(ArrayFormat::Function),
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicode,
            bytes_format: BytesFormat::Unhex,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::WithTimezone,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: None,
            supports_double_colon: false,
            trim_syntax: TrimSyntax::SpacesOnly,
            collate_syntax: None,
            like_escape_syntax: LikeEscapeSyntax::None,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// Microsoft SQL Server dialect.
    PyMsSql = "MsSql" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::SquareBracket,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::PrefixBigN,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixZeroX,
            date_format: DateFormat::Quoted,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: Some(TryCastSyntax::TryCast),
            supports_double_colon: false,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::InfixNoShift,
        }
    }

    /// MySQL dialect.
    PyMySql = "MySql" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: Some(BitFormat::PrefixSmallB),
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashMysql,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: DateFormat::Quoted,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: None,
            supports_double_colon: false,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// Oracle SQL dialect.
    PyOracle = "Oracle" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered(':'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::HexToRaw,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::WithTimezone,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: None,
            supports_double_colon: false,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::BitAndOnly,
        }
    }

    /// PostgreSQL dialect.
    PyPostgreSql = "PostgreSql" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: Some(BitFormat::PrefixBigB),
            array_format: Some(ArrayFormat::Constructor),
            inet_format: Some(InetFormat::Cast),
            macaddr_format: Some(MacAddrFormat::Cast),
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::ArrayConstructor,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::PrefixBigE,
            string_escape: StringEscape::BackslashHex,
            bytes_format: BytesFormat::SlashXBytea,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Keyword,
            timestamp_format: TimestampFormat::Timestamptz,
            uuid_format: UuidFormat::CastUuid,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: None,
            supports_double_colon: true,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// Amazon Redshift dialect.
    PyRedshift = "Redshift" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('$'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::ToVarbyte,
            date_format: DateFormat::Quoted,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: Some(TryCastSyntax::TryCast),
            supports_double_colon: true,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: Some(CollateSyntax::FunctionString),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::InfixAndShift,
        }
    }

    /// Snowflake SQL dialect.
    PySnowflake = "Snowflake" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered(':'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashHexZero,
            bytes_format: BytesFormat::ToBinary,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::Timestamptz,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::ParseJson,
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
            try_cast_syntax: Some(TryCastSyntax::TryCast),
            supports_double_colon: true,
            trim_syntax: TrimSyntax::Function,
            collate_syntax: Some(CollateSyntax::FunctionString),
            like_escape_syntax: LikeEscapeSyntax::IncludingAny,
            bitwise_syntax: BitwiseSyntax::BitShiftFunctions,
        }
    }

    /// Apache Spark SQL dialect.
    PySpark = "Spark" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: None,
            array_format: Some(ArrayFormat::Function),
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::Postfix),
            like_any_syntax: LikeAnySyntax::PatternList,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::Backtick,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::BackslashUnicodeQuote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::WithTimezone,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: Some(TryCastSyntax::TryCast),
            supports_double_colon: true,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: None,
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// SQLite dialect.
    PySqlite = "Sqlite" {
        Dialect {
            positional_parameter: PositionalParameter::Numbered('?'),
            bit_format: None,
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: None,
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Int,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PrefixBigX,
            date_format: DateFormat::Quoted,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::Quoted,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: None,
            supports_double_colon: false,
            trim_syntax: TrimSyntax::Function,
            collate_syntax: Some(CollateSyntax::InfixIdent),
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::Infix,
        }
    }

    /// Teradata SQL dialect.
    PyTeradata = "Teradata" {
        Dialect {
            positional_parameter: PositionalParameter::Simple('?'),
            bit_format: Some(BitFormat::PostfixBigB),
            array_format: None,
            inet_format: None,
            macaddr_format: None,
            null_treatment_syntax: Some(NullTreatmentSyntax::InsideParens),
            like_any_syntax: LikeAnySyntax::OrExpansion,
            bool_format: BoolFormat::Keyword,
            identifier_format: IdentifierFormat::DoubleQuote,
            identifier_escape: IdentifierEscape::Double,
            string_format: StringFormat::SingleQuote,
            string_escape: StringEscape::Quote,
            bytes_format: BytesFormat::PostfixXbv,
            date_format: DateFormat::Keyword,
            time_format: TimeFormat::Quoted,
            timestamp_format: TimestampFormat::WithTimezone,
            uuid_format: UuidFormat::Quoted,
            json_format: JsonFormat::Quoted,
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
            try_cast_syntax: Some(TryCastSyntax::TryCastWord),
            supports_double_colon: false,
            trim_syntax: TrimSyntax::SqlFrom,
            collate_syntax: None,
            like_escape_syntax: LikeEscapeSyntax::Scalar,
            bitwise_syntax: BitwiseSyntax::ShiftFunctions,
        }
    }
);
