/// SQL data types
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DataTypeName(arcstr::ArcStr);

// Create all no-argument data types once. This reduces memory footprint, thanks to `arcstr::ArcStr`.
macro_rules! simple_data_types {
    (
        $(
            $(#[$docs:meta])*
            ($upcase:ident, $realname:literal, $($possiblenames:literal,)*);
        )+
    ) => {
        $(
            $(#[$docs])*
            pub(super) const $upcase: DataTypeName = DataTypeName(arcstr::literal!($realname));
        )+

        impl From<&str> for DataTypeName {
            fn from(value: &str) -> Self {
                match value {
                    $(
                        $realname $(| $possiblenames)* => $upcase,
                    )+
                    _ => DataTypeName(arcstr::ArcStr::from(value)),
                }
            }
        }
    };
}

simple_data_types! {
    /// Uuid type.
    (UUID_DATA_TYPE, "UUID", "uuid",);
    /// [MySQL] blob with up to 2**8 bytes.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/9.1/en/blob.html
    (TINY_BLOB_DATA_TYPE, "TINYBLOB", "tinyblob",);
    /// [MySQL] blob with up to 2**24 bytes.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/9.1/en/blob.html
    (MEDIUM_BLOB_DATA_TYPE, "MEDIUMBLOB", "mediumblob",);
    /// [MySQL] blob with up to 2**32 bytes.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/9.1/en/blob.html
    (LONG_BLOB_DATA_TYPE, "LONGBLOB", "longblob",);
    /// Unsigned tiny integer, e.g. UTINYINT
    (U_TINY_INT_DATA_TYPE, "UTINYINT", "utinyint",);
    /// Unsigned tiny integer, e.g. UTINYINT
    (U_SMALL_INT_DATA_TYPE, "USMALLINT", "usmallint",);
    /// integer, e.g. INTEGER
    (INTEGER_DATA_TYPE, "INTEGER", "integer",);
    /// Integer type in [BigQuery], [ClickHouse].
    ///
    /// [BigQuery]: https://cloud.google.com/bigquery/docs/reference/standard-sql/data-types#integer_types
    /// [ClickHouse]: https://clickhouse.com/docs/en/sql-reference/data-types/int-uint
    (INT64_DATA_TYPE, "INT64", "int64",);
    /// 128-bit integer type, e.g. HUGEINT.
    (HUGE_INT_DATA_TYPE, "HUGEINT", "hugeint",);
    /// Unsigned 128-bit integer type, e.g. UHUGEINT.
    (U_HUGE_INT_DATA_TYPE, "UHUGEINT", "uhugeint",);
    /// Unsigned big integer, e.g. UBIGINT.
    (U_BIG_INT_DATA_TYPE, "UBIGINT", "ubigint",);
    /// Signed integer as used in [MySQL CAST] target types, without `INTEGER` suffix.
    ///
    /// [MySQL CAST]: https://dev.mysql.com/doc/refman/8.4/en/cast-functions.html
    (SIGNED_DATA_TYPE, "SIGNED", "signed",);
    /// Signed integer as used in [MySQL CAST] target types, with `INTEGER` suffix.
    ///
    /// [MySQL CAST]: https://dev.mysql.com/doc/refman/8.4/en/cast-functions.html
    (SIGNED_INTEGER_DATA_TYPE, "SIGNED INTEGER", "signed integer",);
    /// Unsigned integer as used in [MySQL CAST] target types, without `INTEGER` suffix.
    ///
    /// [MySQL CAST]: https://dev.mysql.com/doc/refman/8.4/en/cast-functions.html
    (UNSIGNED_DATA_TYPE, "UNSIGNED", "unsigned",);
    /// Unsigned integer as used in [MySQL CAST] target types, with `INTEGER` suffix.
    ///
    /// [MySQL CAST]: https://dev.mysql.com/doc/refman/8.4/en/cast-functions.html
    (UNSIGNED_INTEGER_DATA_TYPE, "UNSIGNED INTEGER", "unsigned integer",);
    /// Float4 is an alias for Real in [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype.html
    (FLOAT4_DATA_TYPE, "FLOAT4", "float4",);
    /// Floating point in [BigQuery] and [ClickHouse].
    ///
    /// [BigQuery]: https://cloud.google.com/bigquery/docs/reference/standard-sql/data-types#floating_point_types
    /// [ClickHouse]: https://clickhouse.com/docs/en/sql-reference/data-types/float
    (FLOAT64_DATA_TYPE, "Float64", "float64",);
    /// Floating point, e.g. REAL.
    (REAL_DATA_TYPE, "REAL", "real",);
    /// [MySQL] unsigned real, e.g. REAL UNSIGNED.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/8.4/en/numeric-type-syntax.html
    (REAL_UNSIGNED_DATA_TYPE, "REAL UNSIGNED", "real unsigned",);
    /// Float8 is an alias for Double in [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype.html
    (FLOAT8_DATA_TYPE, "FLOAT8", "float8",);
    /// Double Precision, see [SQL Standard], [PostgreSQL].
    ///
    /// [SQL Standard]: https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#approximate-numeric-type
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype-numeric.html
    (DOUBLE_PRECISION_DATA_TYPE, "DOUBLE PRECISION", "double precision",);
    /// [MySQL] unsigned double precision, e.g. DOUBLE PRECISION UNSIGNED.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/8.4/en/numeric-type-syntax.html
    (DOUBLE_PRECISION_UNSIGNED_DATA_TYPE, "DOUBLE PRECISION UNSIGNED", "double precision unsigned",);
    /// Bool is an alias for Boolean, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype.html
    (BOOL_DATA_TYPE, "BOOL", "bool",);
    /// Boolean type.
    (BOOLEAN_DATA_TYPE, "BOOLEAN", "boolean",);
    /// Date type.
    (DATE_DATA_TYPE, "DATE", "date",);
    /// Time type.
    (TIME_DATA_TYPE, "TIME", "time",);
    /// Time with time zone, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype-datetime.html
    (TIME_WITH_TIME_ZONE_DATA_TYPE, "TIME WITH TIME ZONE", "time with time zone",);
    /// Datetime type, see [MySQL].
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/8.4/en/datetime.html
    (DATETIME_DATA_TYPE, "DATETIME", "datetime",);
    /// Timestamp type, see [SQL Standard].
    ///
    /// [SQL Standard]: https://jakewheat.github.io/sql-overview/sql-2016-foundation-grammar.html#datetime-type
    (TIMESTAMP_DATA_TYPE, "TIMESTAMP", "timestamp",);
    /// Timestamp without time zone, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype-datetime.html
    (TIMESTAMP_NTZ_DATA_TYPE, "TIMESTAMP WITHOUT TIME ZONE", "timestamp without time zone", "TIMESTAMP_NTZ", "timestamp_ntz",);
    /// Timestamp with time zone, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype-datetime.html
    (TIMESTAMP_WITH_TZ_DATA_TYPE, "TIMESTAMP WITH TIME ZONE", "timestamp with time zone", "TIMESTAMPTZ", "timestamptz",);
    /// Regclass used in [PostgreSQL] serial.
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype.html
    (REGCLASS_DATA_TYPE, "REGCLASS", "regclass",);
    /// Text type.
    (TEXT_DATA_TYPE, "TEXT", "text",);
    /// [MySQL] text with up to 2**8 bytes.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/9.1/en/blob.html
    (TINY_TEXT_DATA_TYPE, "TINYTEXT", "tinytext",);
    /// [MySQL] text with up to 2**24 bytes.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/9.1/en/blob.html
    (MEDIUM_TEXT_DATA_TYPE, "MEDIUMTEXT", "mediumtext",);
    /// [MySQL] text with up to 2**32 bytes.
    ///
    /// [MySQL]: https://dev.mysql.com/doc/refman/9.1/en/blob.html
    (LONG_TEXT_DATA_TYPE, "LONGTEXT", "longtext",);
    /// Bytea type, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/datatype-bit.html
    (BYTEA_DATA_TYPE, "BYTEA", "bytea",);
    /// JSON type.
    (JSON_DATA_TYPE, "JSON", "json",);
    /// Binary JSON type.
    (JSONB_DATA_TYPE, "JSONB", "jsonb",);
    /// Interval type.
    (INTERVAL_DATA_TYPE, "INTERVAL", "interval",);
    /// No type specified — only used with SQLiteDialect, e.g. `CREATE TABLE t1 (a)`.
    (UNSPECIFIED_DATA_TYPE, "UNSPECIFIED", "unspecified",);
    /// Trigger data type, returned by functions associated with triggers, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/current/plpgsql-trigger.html
    (TRIGGER_DATA_TYPE, "TRIGGER", "trigger",);
    /// Any data type, used in BigQuery UDF definitions for templated parameters, see [BigQuery].
    ///
    /// [BigQuery]: https://cloud.google.com/bigquery/docs/user-defined-functions#templated-sql-udf-parameters
    (ANY_TYPE_DATA_TYPE, "ANY TYPE", "any type", "anytype", "ANYTYPE",);
    /// PostgreSQL text search vector, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/17/datatype-textsearch.html
    (TS_VECTOR_DATA_TYPE, "TSVECTOR", "tsvector",);
    /// PostgreSQL text search query, see [PostgreSQL].
    ///
    /// [PostgreSQL]: https://www.postgresql.org/docs/17/datatype-textsearch.html
    (TS_QUERY_DATA_TYPE, "TSQUERY", "tsquery",);
}

impl AsRef<str> for DataTypeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Into<String> for DataTypeName {
    fn into(self) -> String {
        self.0.to_string()
    }
}

impl std::fmt::Display for DataTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
