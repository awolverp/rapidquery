/// Specifies data type groups
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum DataTypeGroup {
    STRING,
    UUID,
    BYTES,
    BIT,
    INTEGER,
    FLOAT,
    BOOLEAN,
    DATE,
    TIME,
    DATETIME,
    JSON,
    JSONB,
    INET,
    MACADDRESS,
    CUSTOM,
}

/// SQL data types
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DataTypeName(arcstr::ArcStr, DataTypeGroup);

// Create all no-argument data types once. This reduces memory footprint, thanks to `arcstr::ArcStr`.
macro_rules! simple_data_types {
    (
        $(
            $(#[$docs:meta])*
            ($upcase:ident => ($realname:literal, $group:ident), $possible_name:literal, $($possible_names:literal),*);
        )+
    ) => {
        $(
            $(#[$docs])*
            pub(super) const $upcase: DataTypeName = DataTypeName(arcstr::literal!($realname), DataTypeGroup::$group);
        )+

        impl From<&str> for DataTypeName {
            fn from(value: &str) -> Self {
                if value.find('(').is_some() {
                    // For data types that has gotten parameters, we cannot use constant variables
                    DataTypeName(arcstr::ArcStr::from(value), DataTypeGroup::CUSTOM)
                } else {
                    let lowercase = value.to_ascii_lowercase();
                    match lowercase.as_str() {
                        $(
                            $possible_name $(| $possible_names)* => $upcase,
                        )+
                        _ => DataTypeName(arcstr::ArcStr::from(value), DataTypeGroup::CUSTOM),
                    }
                }
            }
        }
    };
}

simple_data_types! {
    // --- STRING group ---
    (CHARACTER_DATA_TYPE => ("CHARACTER", STRING), "character",);
    (CHAR_DATA_TYPE => ("CHAR", STRING), "char",);
    (CHARACTER_VARYING_DATA_TYPE => ("CHARACTER VARYING", STRING), "characater varying",);
    (CHAR_VARYING_DATA_TYPE => ("CHAR VARYING", STRING), "char varying",);
    (VARCHAR_DATA_TYPE => ("VARCHAR", STRING), "varchar",);
    (NVARCHAR_DATA_TYPE => ("NVARCHAR", STRING), "nvarchar",);
    (CHARACTER_LARGE_OBJECT_DATA_TYPE => ("CHARACTER LARGE OBJECT", STRING), "character large object",);
    (CHAR_LARGE_OBJECT_DATA_TYPE => ("CHAR LARGE OBJECT", STRING), "char large object",);
    (CLOB_DATA_TYPE => ("CLOB", STRING), "clob",);
    (REGCLASS_DATA_TYPE => ("REGCLASS", STRING), "regclass",);
    (TEXT_DATA_TYPE => ("TEXT", STRING), "text",);
    (TINYTEXT_DATA_TYPE => ("TINYTEXT", STRING), "tinytext",);
    (MEDIUMTEXT_DATA_TYPE => ("MEDIUMTEXT", STRING), "mediumtext",);
    (LONGTEXT_DATA_TYPE => ("LONGTEXT", STRING), "longtext",);
    (STRING_DATA_TYPE => ("STRING", STRING), "string",);

    // --- UUID group ---
    (UUID_DATA_TYPE => ("UUID", UUID), "uuid",);

    // --- BINARY group ---
    (BINARY_DATA_TYPE => ("BINARY", BYTES), "binary",);
    (VARBINARY_DATA_TYPE => ("VARBINARY", BYTES), "varbinary",);
    (BLOB_DATA_TYPE => ("BLOB", BYTES), "blob",);
    (TINYBLOB_DATA_TYPE => ("TINYBLOB", BYTES), "tinyblob",);
    (MEDIUMBLOB_DATA_TYPE => ("MEDIUMBLOB", BYTES), "mediumblob",);
    (LONGBLOB_DATA_TYPE => ("LONGBLOB", BYTES), "longblob",);
    (BYTES_DATA_TYPE => ("BYTES", BYTES), "bytes",);
    (BYTEA_DATA_TYPE => ("BYTEA", BYTES), "bytea",);

    // --- BIT group ---
    (BIT_DATA_TYPE => ("BIT", BIT), "bit",);
    (BIT_VARYING_DATA_TYPE => ("BIT VARYING", BIT), "bit varying",);
    (VARBIT_DATA_TYPE => ("VARBIT", BIT), "varbit",);

    // --- FLOAT group ---
    (NUMERIC_DATA_TYPE => ("NUMERIC", FLOAT), "numeric",);
    (DECIMAL_DATA_TYPE => ("DECIMAL", FLOAT), "decimal",);
    (BIGNUMERIC_DATA_TYPE => ("BIGNUMERIC", FLOAT), "bignumeric",);
    (BIGDECIMAL_DATA_TYPE => ("BIGDECIMAL", FLOAT), "bigdecimal",);
    (DEC_DATA_TYPE => ("DEC", FLOAT), "dec",);
    (FLOAT_DATA_TYPE => ("FLOAT", FLOAT), "float",);
    (FLOAT4_DATA_TYPE => ("FLOAT4", FLOAT), "float4",);
    (FLOAT8_DATA_TYPE => ("FLOAT8", FLOAT), "float8",);
    (FLOAT64_DATA_TYPE => ("FLOAT64", FLOAT), "float64",);
    (REAL_DATA_TYPE => ("REAL", FLOAT), "real",);
    (REAL_UNSIGNED_DATA_TYPE => ("REAL UNSIGNED", FLOAT), "real unsigned",);
    (DOUBLE_DATA_TYPE => ("DOUBLE", FLOAT), "double",);
    (DOUBLE_UNSIGNED_DATA_TYPE => ("DOUBLE UNSIGNED", FLOAT), "double unsigned",);
    (DOUBLE_PRECISION_DATA_TYPE => ("DOUBLE PRECISION", FLOAT), "double precision",);
    (DOUBLE_PRECISION_UNSIGNED_DATA_TYPE => ("DOUBLE PRECISION UNSIGNED", FLOAT), "double precision unsigned",);

    // --- INTEGER ---
    (TINYINT_DATA_TYPE => ("TINYINT", INTEGER), "tinyint",);
    (TINYINT_UNSIGNED_DATA_TYPE => ("TINYINT UNSIGNED", INTEGER), "tinyint unsigned", "utinyint");
    (INT2_DATA_TYPE => ("INT2", INTEGER), "int2",);
    (INT2_UNSIGNED_DATA_TYPE => ("INT2 UNSIGNED", INTEGER), "int2 unsigned",);
    (SMALLINT_DATA_TYPE => ("SMALLINT", INTEGER), "smallint",);
    (SMALLINT_UNSIGNED_DATA_TYPE => ("SMALLINT UNSIGNED", INTEGER), "smallint unsigned", "usmallint");
    (MEDIUMINT_DATA_TYPE => ("MEDIUMINT", INTEGER), "mediumint",);
    (MEDIUMINT_UNSIGNED_DATA_TYPE => ("MEDIUMINT UNSIGNED", INTEGER), "mediumint unsigned",);
    (INT_DATA_TYPE => ("INT", INTEGER), "int",);
    (INT4_DATA_TYPE => ("INT4", INTEGER), "int4",);
    (INT8_DATA_TYPE => ("INT8", INTEGER), "int8",);
    (INT64_DATA_TYPE => ("INT64", INTEGER), "int64",);
    (INTEGER_DATA_TYPE => ("INTEGER", INTEGER), "integer",);
    (UNSIGNED_INTEGER_DATA_TYPE => ("UNSIGNED INTEGER", INTEGER), "unsigned integer", "integer unsigned", "int unsigned");
    (INT4_UNSIGNED_DATA_TYPE => ("INT4 UNSIGNED", INTEGER), "int4 unsigned",);
    (HUGEINT_DATA_TYPE => ("HUGEINT", INTEGER), "hugeint",);
    (UHUGEINT_DATA_TYPE => ("UHUGEINT", INTEGER), "uhugeint",);
    (BIGINT_DATA_TYPE => ("BIGINT", INTEGER), "bigint",);
    (BIGINT_UNSIGNED_DATA_TYPE => ("BIGINT UNSIGNED", INTEGER), "bigint unsigned", "unsigned bitint", "ubigint");
    (INT8_UNSIGNED_DATA_TYPE => ("INT8 UNSIGNED", INTEGER), "int8 unsigned",);
    (SIGNED_DATA_TYPE => ("SIGNED", INTEGER), "signed",);
    (SIGNED_INTEGER_DATA_TYPE => ("SIGNED INTEGER", INTEGER), "signed integer",);
    (UNSIGNED_DATA_TYPE => ("UNSIGNED", INTEGER), "unsigned",);

    // --- BOOLEAN group ---
    (BOOLEAN_DATA_TYPE => ("BOOLEAN", BOOLEAN), "boolean", "bool");

    // --- DATE group ---
    (DATE_DATA_TYPE => ("DATE", DATE), "date",);

    // --- TIME group ---
    (TIME_DATA_TYPE => ("TIME", TIME), "time", "time without timezone");
    (TIME_WITH_TIMEZONE_DATA_TYPE => ("TIME WITH TIMEZONE", TIME), "time with timezone",);

    // --- DATETIME group ---
    (DATETIME_DATA_TYPE => ("DATETIME", DATETIME), "datetime",);
    (TIMESTAMP_DATA_TYPE => ("TIMESTAMP", DATETIME), "timestamp", "timestamp without timezone");
    (TIMESTAMP_WITH_TIMEZONE_DATA_TYPE => ("TIMESTAMP WITH TIMEZONE", DATETIME), "timestamp with timezone", "timestamptz");
    (TIMESTAMPNTZ_DATA_TYPE => ("TIMESTAMPNTZ", DATETIME), "timestampntz",);

    // --- JSON group ---
    (JSON_DATA_TYPE => ("JSON", JSON), "json",);
    (JSONB_DATA_TYPE => ("JSONB", JSONB), "jsonb",);

    // --- INET group ---
    (INET_DATA_TYPE => ("INET", INET), "inet",);

    // --- MACADDR group ---
    (MACADDRESS_DATA_TYPE => ("MACADDRESS", MACADDRESS), "macaddress", "macaddr");

    // --- CUSTOM group ---
    (ANY_DATA_TYPE => ("ANY", CUSTOM), "any",);
    (TRIGGER_DATA_TYPE => ("TRIGGER", CUSTOM), "trigger",);
    (UNSPECIFIED_DATA_TYPE => ("UNSPECIFIED", CUSTOM), "unspecified",);
}

impl DataTypeName {
    pub fn group(&self) -> DataTypeGroup {
        self.1
    }
}

impl From<String> for DataTypeName {
    fn from(value: String) -> Self {
        DataTypeName::from(value.as_str())
    }
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
