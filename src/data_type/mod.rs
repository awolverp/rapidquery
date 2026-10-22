use crate::internal::sql;

#[cfg(test)]
mod tests;

/// SQL type name, interned when it is one of the known spellings.
///
/// Known names fold to their traditional uppercase form. Unknown names and
/// parameterized names such as `varchar(16)` keep the caller's spelling.
///
/// ```
/// use rapidquery::data_type::DataTypeName;
///
/// assert_eq!(DataTypeName::from("varchar").as_str(), "VARCHAR");
/// assert_eq!(DataTypeName::from("varchar(16)").as_str(), "varchar(16)");
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(transparent)]
pub struct DataTypeName(arcstr::ArcStr);

impl DataTypeName {
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Longest interned name is `"timestamp without time zone"` (27 bytes).
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
    DataTypeName(value.into())
}

// Interned no-arg types. `arcstr::literal!` shares one allocation per spelling.
macro_rules! simple_data_types {
    (
        $(
            $(#[$docs:meta])*
            ($upcase:ident => $realname:literal, $($possible_names:literal)|+);
        )+
    ) => {
        $(
            $(#[$docs])*
            pub(super) const $upcase: DataTypeName = DataTypeName(arcstr::literal!($realname));
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
    (CHARACTER_DATA_TYPE => "CHARACTER", "character");
    (CHAR_DATA_TYPE => "CHAR", "char");
    (CHARACTER_VARYING_DATA_TYPE => "CHARACTER VARYING", "character varying");
    (CHAR_VARYING_DATA_TYPE => "CHAR VARYING", "char varying");
    (VARCHAR_DATA_TYPE => "VARCHAR", "varchar");
    (NVARCHAR_DATA_TYPE => "NVARCHAR", "nvarchar");
    (TEXT_DATA_TYPE => "TEXT", "text");

    // --- Uuid ---
    (UUID_DATA_TYPE => "UUID", "uuid");

    // --- Bytes ---
    (BYTES_DATA_TYPE => "BYTES", "bytes");
    (BINARY_DATA_TYPE => "BINARY", "binary");
    (VARBINARY_DATA_TYPE => "VARBINARY", "varbinary");
    (BLOB_DATA_TYPE => "BLOB", "blob");
    (BYTEA_DATA_TYPE => "BYTEA", "bytea");

    // --- Bit ---
    (BIT_DATA_TYPE => "BIT", "bit");
    (VARBIT_DATA_TYPE => "VARBIT", "varbit");
    (BIT_VARYING_DATA_TYPE => "BIT VARYING", "bit varying");

    // --- Float (includes decimal/numeric) ---
    (NUMERIC_DATA_TYPE => "NUMERIC", "numeric");
    (BIGNUMERIC_DATA_TYPE => "BIGNUMERIC", "bignumeric");
    (DECIMAL_DATA_TYPE => "DECIMAL", "decimal");
    (BIGDECIMAL_DATA_TYPE => "BIGDECIMAL", "bigdecimal");
    (FLOAT_DATA_TYPE => "FLOAT", "float");
    (REAL_DATA_TYPE => "REAL", "real");
    (DOUBLE_DATA_TYPE => "DOUBLE", "double");
    (DOUBLE_PRECISION_DATA_TYPE => "DOUBLE PRECISION", "double precision");

    // --- Integer ---
    (TINYINT_DATA_TYPE => "TINYINT", "tinyint");
    (SMALLINT_DATA_TYPE => "SMALLINT", "smallint");
    (INT_DATA_TYPE => "INT", "int");
    (INTEGER_DATA_TYPE => "INTEGER", "integer");
    (BIGINT_DATA_TYPE => "BIGINT", "bigint");

    // --- Boolean ---
    (BOOLEAN_DATA_TYPE => "BOOLEAN", "boolean" | "bool");

    // --- Date ---
    (DATE_DATA_TYPE => "DATE", "date");

    // --- Time ---
    (TIME_DATA_TYPE => "TIME", "time" | "time without timezone" | "time without time zone");
    (TIME_WITH_TIMEZONE_DATA_TYPE => "TIME WITH TIMEZONE", "time with timezone");
    (TIME_WITH_TIME_ZONE_DATA_TYPE => "TIME WITH TIME ZONE", "time with time zone");

    // --- Datetime ---
    (DATETIME_DATA_TYPE => "DATETIME", "datetime");
    (TIMESTAMP_DATA_TYPE => "TIMESTAMP", "timestamp" | "timestamp without timezone" | "timestamp without time zone");
    (TIMESTAMP_WITH_TIMEZONE_DATA_TYPE => "TIMESTAMP WITH TIMEZONE", "timestamp with timezone");
    (TIMESTAMP_WITH_TIME_ZONE_DATA_TYPE => "TIMESTAMP WITH TIME ZONE", "timestamp with time zone");
    (TIMESTAMPTZ_DATA_TYPE => "TIMESTAMPTZ", "timestamptz");

    // --- Json ---
    (JSON_DATA_TYPE => "JSON", "json");
    (JSONB_DATA_TYPE => "JSONB", "jsonb");

    // --- Inet ---
    (INET_DATA_TYPE => "INET", "inet");

    // --- MacAddress ---
    (MACADDRESS_DATA_TYPE => "MACADDRESS", "macaddress" | "macaddr");
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
        val.0.to_string()
    }
}

impl std::fmt::Display for DataTypeName {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl sql::WriteSql for DataTypeName {
    fn write_sql<W: sql::SqlWriter>(
        &self,
        _dialect: &crate::dialect::PyDialect,
        out: &mut W,
    ) -> sql::PrepareResult<()> {
        out.push_str(self.as_str());
        Ok(())
    }
}
