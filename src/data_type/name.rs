use crate::sql::{PrepareResult, SqlWriter, WriteSql};

/// Category used to infer a built-in serializer from a type name.
///
/// Distinct SQL spellings collapse onto one serializer (every integer width
/// is [`crate::data_type::Serializer::Integer`]). [`Json`] and [`Jsonb`] stay
/// distinct as groups so a later layer can treat storage differently; both
/// currently infer [`crate::data_type::Serializer::Json`].
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

/// SQL type spelling plus its [`DataTypeGroup`].
///
/// Known names are interned (`arcstr::literal!`) under that same spelling
/// (case-folded to the traditional uppercase form). Unknown and parameterized
/// names keep the caller's spelling and are [`DataTypeGroup::Custom`].
/// Synonyms are not rewritten to a different name.
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

/// Longest interned name is `"timestamp with timezone"` (23 bytes).
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

fn custom_name<T: Into<arcstr::ArcStr>>(value: T) -> DataTypeName {
    DataTypeName {
        name: value.into(),
        group: DataTypeGroup::Custom,
    }
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
    (CHARACTER_VARYING_DATA_TYPE => ("CHARACTER VARYING", String), "character varying");
    (CHAR_VARYING_DATA_TYPE => ("CHAR VARYING", String), "char varying");
    (VARCHAR_DATA_TYPE => ("VARCHAR", String), "varchar");
    (NVARCHAR_DATA_TYPE => ("NVARCHAR", String), "nvarchar");
    (TEXT_DATA_TYPE => ("TEXT", String), "text");

    // --- Uuid ---
    (UUID_DATA_TYPE => ("UUID", Uuid), "uuid");

    // --- Bytes ---
    (BINARY_DATA_TYPE => ("BINARY", Bytes), "binary");
    (VARBINARY_DATA_TYPE => ("VARBINARY", Bytes), "varbinary");
    (BLOB_DATA_TYPE => ("BLOB", Bytes), "blob");
    (BYTEA_DATA_TYPE => ("BYTEA", Bytes), "bytea");

    // --- Bit ---
    (BIT_DATA_TYPE => ("BIT", Bit), "bit");
    (BIT_VARYING_DATA_TYPE => ("BIT VARYING", Bit), "bit varying");

    // --- Float (includes decimal/numeric) ---
    (NUMERIC_DATA_TYPE => ("NUMERIC", Float), "numeric");
    (DECIMAL_DATA_TYPE => ("DECIMAL", Float), "decimal");
    (DEC_DATA_TYPE => ("DEC", Float), "dec");
    (FLOAT_DATA_TYPE => ("FLOAT", Float), "float");
    (REAL_DATA_TYPE => ("REAL", Float), "real");
    (DOUBLE_DATA_TYPE => ("DOUBLE", Float), "double");
    (DOUBLE_PRECISION_DATA_TYPE => ("DOUBLE PRECISION", Float), "double precision");

    // --- Integer ---
    (TINYINT_DATA_TYPE => ("TINYINT", Integer), "tinyint");
    (SMALLINT_DATA_TYPE => ("SMALLINT", Integer), "smallint");
    (INT_DATA_TYPE => ("INT", Integer), "int");
    (INTEGER_DATA_TYPE => ("INTEGER", Integer), "integer");
    (BIGINT_DATA_TYPE => ("BIGINT", Integer), "bigint");

    // --- Boolean ---
    (BOOLEAN_DATA_TYPE => ("BOOLEAN", Boolean), "boolean");

    // --- Date ---
    (DATE_DATA_TYPE => ("DATE", Date), "date");

    // --- Time ---
    (TIME_DATA_TYPE => ("TIME", Time), "time");
    (TIME_WITH_TIMEZONE_DATA_TYPE => ("TIME WITH TIMEZONE", Time), "time with timezone");

    // --- Datetime ---
    (DATETIME_DATA_TYPE => ("DATETIME", Datetime), "datetime");
    (TIMESTAMP_DATA_TYPE => ("TIMESTAMP", Datetime), "timestamp");
    (TIMESTAMP_WITH_TIMEZONE_DATA_TYPE => ("TIMESTAMP WITH TIMEZONE", Datetime), "timestamp with timezone");

    // --- Json ---
    (JSON_DATA_TYPE => ("JSON", Json), "json");
    (JSONB_DATA_TYPE => ("JSONB", Jsonb), "jsonb");

    // --- Inet ---
    (INET_DATA_TYPE => ("INET", Inet), "inet");

    // --- MacAddress ---
    (MACADDRESS_DATA_TYPE => ("MACADDRESS", MacAddress), "macaddress" | "macaddr");
}

fn is_parameterized(value: &str) -> bool {
    value.as_bytes().contains(&b'(')
}

impl From<&str> for DataTypeName {
    fn from(value: &str) -> Self {
        if is_parameterized(value) {
            return custom_name(value);
        }
        interned(value).unwrap_or_else(|| custom_name(value))
    }
}

impl From<String> for DataTypeName {
    fn from(value: String) -> Self {
        if is_parameterized(&value) {
            return custom_name(value);
        }
        if let Some(dt) = interned(&value) {
            return dt;
        }
        custom_name(value)
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
        _dialect: &crate::dialect::PySqlDialect,
        out: &mut W,
    ) -> PrepareResult<()> {
        out.push_str(self.as_str());
        Ok(())
    }
}
