use crate::native::sql::{PrepareResult, SqlWriter, WriteSql};

/// Category used to infer a built-in serializer from a type name.
///
/// Distinct SQL spellings collapse onto one serializer (every integer width
/// is [`crate::native::Serializer::Integer`]). [`Json`] and [`Jsonb`] stay
/// distinct as groups so a later layer can treat storage differently; both
/// currently infer [`crate::native::Serializer::Json`].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum DataTypeGroup {
    String,
    Uuid,
    Bytes,
    Bit,
    Integer,
    Float,
    Boolean,
    Date,
    Time,
    Datetime,
    Json,
    Jsonb,
    Inet,
    MacAddress,
    Custom,
}

/// Canonical SQL type spelling plus its [`DataTypeGroup`].
///
/// Known names are interned (`arcstr::literal!`). Unknown and parameterized
/// names keep the caller's spelling and are [`DataTypeGroup::Custom`].
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DataTypeName {
    name: arcstr::ArcStr,
    group: DataTypeGroup,
}

impl DataTypeName {
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn group(&self) -> DataTypeGroup {
        self.group
    }
}

/// Longest interned alias is `"timestamp without timezone"` (26 bytes).
const LOOKUP_BUF: usize = 32;

fn interned(value: &str) -> Option<DataTypeName> {
    if value.len() > LOOKUP_BUF || !value.is_ascii() {
        return None;
    }
    let mut buf = [0u8; LOOKUP_BUF];
    for (i, b) in value.bytes().enumerate() {
        buf[i] = b.to_ascii_lowercase();
    }
    // SAFETY: `value` is ASCII, so the lowercased copy is valid UTF-8.
    let lower = unsafe { std::str::from_utf8_unchecked(&buf[..value.len()]) };
    lookup_alias(lower)
}

fn custom_from_str(value: &str) -> DataTypeName {
    DataTypeName {
        name: arcstr::ArcStr::from(value),
        group: DataTypeGroup::Custom,
    }
}

fn custom_from_string(value: String) -> DataTypeName {
    DataTypeName {
        name: arcstr::ArcStr::from(value),
        group: DataTypeGroup::Custom,
    }
}

fn is_parameterized(value: &str) -> bool {
    value.as_bytes().contains(&b'(')
}

// Interned no-arg types. `arcstr::literal!` shares one allocation per spelling.
macro_rules! simple_data_types {
    (
        $(
            $(#[$docs:meta])*
            ($upcase:ident => ($realname:literal, $group:ident), $($possible_names:literal)|+);
        )+
    ) => {
        $(
            $(#[$docs])*
            pub(super) const $upcase: DataTypeName = DataTypeName {
                name: arcstr::literal!($realname),
                group: DataTypeGroup::$group,
            };
        )+

        fn lookup_alias(lowercase: &str) -> Option<DataTypeName> {
            match lowercase {
                $(
                    $($possible_names)|+ => Some($upcase),
                )+
                _ => None,
            }
        }
    };
}

simple_data_types! {
    // --- String ---
    (CHARACTER_DATA_TYPE => ("CHARACTER", String), "character");
    (CHAR_DATA_TYPE => ("CHAR", String), "char");
    (CHARACTER_VARYING_DATA_TYPE => ("CHARACTER VARYING", String), "character varying" | "characater varying");
    (CHAR_VARYING_DATA_TYPE => ("CHAR VARYING", String), "char varying");
    (VARCHAR_DATA_TYPE => ("VARCHAR", String), "varchar");
    (NVARCHAR_DATA_TYPE => ("NVARCHAR", String), "nvarchar");
    (CHARACTER_LARGE_OBJECT_DATA_TYPE => ("CHARACTER LARGE OBJECT", String), "character large object");
    (CHAR_LARGE_OBJECT_DATA_TYPE => ("CHAR LARGE OBJECT", String), "char large object");
    (CLOB_DATA_TYPE => ("CLOB", String), "clob");
    (REGCLASS_DATA_TYPE => ("REGCLASS", String), "regclass");
    (TEXT_DATA_TYPE => ("TEXT", String), "text");
    (TINYTEXT_DATA_TYPE => ("TINYTEXT", String), "tinytext");
    (MEDIUMTEXT_DATA_TYPE => ("MEDIUMTEXT", String), "mediumtext");
    (LONGTEXT_DATA_TYPE => ("LONGTEXT", String), "longtext");
    (STRING_DATA_TYPE => ("STRING", String), "string");

    // --- Uuid ---
    (UUID_DATA_TYPE => ("UUID", Uuid), "uuid");

    // --- Bytes ---
    (BINARY_DATA_TYPE => ("BINARY", Bytes), "binary");
    (VARBINARY_DATA_TYPE => ("VARBINARY", Bytes), "varbinary");
    (BLOB_DATA_TYPE => ("BLOB", Bytes), "blob");
    (TINYBLOB_DATA_TYPE => ("TINYBLOB", Bytes), "tinyblob");
    (MEDIUMBLOB_DATA_TYPE => ("MEDIUMBLOB", Bytes), "mediumblob");
    (LONGBLOB_DATA_TYPE => ("LONGBLOB", Bytes), "longblob");
    (BYTES_DATA_TYPE => ("BYTES", Bytes), "bytes");
    (BYTEA_DATA_TYPE => ("BYTEA", Bytes), "bytea");

    // --- Bit ---
    (BIT_DATA_TYPE => ("BIT", Bit), "bit");
    (BIT_VARYING_DATA_TYPE => ("BIT VARYING", Bit), "bit varying");
    (VARBIT_DATA_TYPE => ("VARBIT", Bit), "varbit");

    // --- Float (includes decimal/numeric) ---
    (NUMERIC_DATA_TYPE => ("NUMERIC", Float), "numeric");
    (DECIMAL_DATA_TYPE => ("DECIMAL", Float), "decimal");
    (BIGNUMERIC_DATA_TYPE => ("BIGNUMERIC", Float), "bignumeric");
    (BIGDECIMAL_DATA_TYPE => ("BIGDECIMAL", Float), "bigdecimal");
    (DEC_DATA_TYPE => ("DEC", Float), "dec");
    (FLOAT_DATA_TYPE => ("FLOAT", Float), "float");
    (FLOAT4_DATA_TYPE => ("FLOAT4", Float), "float4");
    (FLOAT8_DATA_TYPE => ("FLOAT8", Float), "float8");
    (FLOAT64_DATA_TYPE => ("FLOAT64", Float), "float64");
    (REAL_DATA_TYPE => ("REAL", Float), "real");
    (REAL_UNSIGNED_DATA_TYPE => ("REAL UNSIGNED", Float), "real unsigned");
    (DOUBLE_DATA_TYPE => ("DOUBLE", Float), "double");
    (DOUBLE_UNSIGNED_DATA_TYPE => ("DOUBLE UNSIGNED", Float), "double unsigned");
    (DOUBLE_PRECISION_DATA_TYPE => ("DOUBLE PRECISION", Float), "double precision");
    (DOUBLE_PRECISION_UNSIGNED_DATA_TYPE => ("DOUBLE PRECISION UNSIGNED", Float), "double precision unsigned");

    // --- Integer ---
    (TINYINT_DATA_TYPE => ("TINYINT", Integer), "tinyint");
    (TINYINT_UNSIGNED_DATA_TYPE => ("TINYINT UNSIGNED", Integer), "tinyint unsigned" | "utinyint");
    (INT2_DATA_TYPE => ("INT2", Integer), "int2");
    (INT2_UNSIGNED_DATA_TYPE => ("INT2 UNSIGNED", Integer), "int2 unsigned");
    (SMALLINT_DATA_TYPE => ("SMALLINT", Integer), "smallint");
    (SMALLINT_UNSIGNED_DATA_TYPE => ("SMALLINT UNSIGNED", Integer), "smallint unsigned" | "usmallint");
    (MEDIUMINT_DATA_TYPE => ("MEDIUMINT", Integer), "mediumint");
    (MEDIUMINT_UNSIGNED_DATA_TYPE => ("MEDIUMINT UNSIGNED", Integer), "mediumint unsigned");
    (INT_DATA_TYPE => ("INT", Integer), "int");
    (INT4_DATA_TYPE => ("INT4", Integer), "int4");
    (INT8_DATA_TYPE => ("INT8", Integer), "int8");
    (INT64_DATA_TYPE => ("INT64", Integer), "int64");
    (INTEGER_DATA_TYPE => ("INTEGER", Integer), "integer");
    (UNSIGNED_INTEGER_DATA_TYPE => ("UNSIGNED INTEGER", Integer), "unsigned integer" | "integer unsigned" | "int unsigned");
    (INT4_UNSIGNED_DATA_TYPE => ("INT4 UNSIGNED", Integer), "int4 unsigned");
    (HUGEINT_DATA_TYPE => ("HUGEINT", Integer), "hugeint");
    (UHUGEINT_DATA_TYPE => ("UHUGEINT", Integer), "uhugeint");
    (BIGINT_DATA_TYPE => ("BIGINT", Integer), "bigint");
    (BIGINT_UNSIGNED_DATA_TYPE => ("BIGINT UNSIGNED", Integer), "bigint unsigned" | "unsigned bigint" | "unsigned bitint" | "ubigint");
    (INT8_UNSIGNED_DATA_TYPE => ("INT8 UNSIGNED", Integer), "int8 unsigned");
    (SIGNED_DATA_TYPE => ("SIGNED", Integer), "signed");
    (SIGNED_INTEGER_DATA_TYPE => ("SIGNED INTEGER", Integer), "signed integer");
    (UNSIGNED_DATA_TYPE => ("UNSIGNED", Integer), "unsigned");

    // --- Boolean ---
    (BOOLEAN_DATA_TYPE => ("BOOLEAN", Boolean), "boolean" | "bool");

    // --- Date ---
    (DATE_DATA_TYPE => ("DATE", Date), "date");

    // --- Time ---
    (TIME_DATA_TYPE => ("TIME", Time), "time" | "time without timezone");
    (TIME_WITH_TIMEZONE_DATA_TYPE => ("TIME WITH TIMEZONE", Time), "time with timezone");

    // --- Datetime ---
    (DATETIME_DATA_TYPE => ("DATETIME", Datetime), "datetime");
    (TIMESTAMP_DATA_TYPE => ("TIMESTAMP", Datetime), "timestamp" | "timestamp without timezone");
    (TIMESTAMP_WITH_TIMEZONE_DATA_TYPE => ("TIMESTAMP WITH TIMEZONE", Datetime), "timestamp with timezone" | "timestamptz");
    (TIMESTAMPNTZ_DATA_TYPE => ("TIMESTAMPNTZ", Datetime), "timestampntz");

    // --- Json ---
    (JSON_DATA_TYPE => ("JSON", Json), "json");
    (JSONB_DATA_TYPE => ("JSONB", Jsonb), "jsonb");

    // --- Inet ---
    (INET_DATA_TYPE => ("INET", Inet), "inet");

    // --- MacAddress ---
    (MACADDRESS_DATA_TYPE => ("MACADDRESS", MacAddress), "macaddress" | "macaddr");

    // --- Custom ---
    (ANY_DATA_TYPE => ("ANY", Custom), "any");
    (TRIGGER_DATA_TYPE => ("TRIGGER", Custom), "trigger");
    (UNSPECIFIED_DATA_TYPE => ("UNSPECIFIED", Custom), "unspecified");
}

impl From<&str> for DataTypeName {
    fn from(value: &str) -> Self {
        if is_parameterized(value) {
            return custom_from_str(value);
        }
        interned(value).unwrap_or_else(|| custom_from_str(value))
    }
}

impl From<String> for DataTypeName {
    fn from(value: String) -> Self {
        if is_parameterized(&value) {
            return custom_from_string(value);
        }
        if let Some(dt) = interned(&value) {
            return dt;
        }
        custom_from_string(value)
    }
}

impl AsRef<str> for DataTypeName {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<DataTypeName> for String {
    #[inline]
    fn from(val: DataTypeName) -> Self {
        val.name.to_string()
    }
}

impl std::fmt::Display for DataTypeName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl WriteSql for DataTypeName {
    fn write_sql<W: SqlWriter>(
        &self,
        _dialect: &crate::native::dialect::SqlDialect,
        out: &mut W,
    ) -> PrepareResult<()> {
        out.push_str(self.as_str());
        Ok(())
    }
}
