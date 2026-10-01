use super::*;

fn assert_copy<T: Copy>(_: T) {}

#[test]
fn data_type_group_is_copy() {
    assert_copy(DataTypeKind::Integer);
}

#[test]
fn all_interned_names_canonicalize() {
    let cases = [
        // --- String ---
        ("character", "CHARACTER", DataTypeKind::String),
        ("char", "CHAR", DataTypeKind::String),
        (
            "character varying",
            "CHARACTER VARYING",
            DataTypeKind::String,
        ),
        ("char varying", "CHAR VARYING", DataTypeKind::String),
        ("varchar", "VARCHAR", DataTypeKind::String),
        ("nvarchar", "NVARCHAR", DataTypeKind::String),
        ("text", "TEXT", DataTypeKind::String),
        // --- Uuid ---
        ("uuid", "UUID", DataTypeKind::Uuid),
        // --- Bytes ---
        ("binary", "BINARY", DataTypeKind::Bytes),
        ("varbinary", "VARBINARY", DataTypeKind::Bytes),
        ("blob", "BLOB", DataTypeKind::Bytes),
        ("bytea", "BYTEA", DataTypeKind::Bytes),
        // --- Bit ---
        ("bit", "BIT", DataTypeKind::Bit),
        ("bit varying", "BIT VARYING", DataTypeKind::Bit),
        // --- Float (includes decimal/numeric) ---
        ("numeric", "NUMERIC", DataTypeKind::Float),
        ("decimal", "DECIMAL", DataTypeKind::Float),
        ("dec", "DEC", DataTypeKind::Float),
        ("float", "FLOAT", DataTypeKind::Float),
        ("real", "REAL", DataTypeKind::Float),
        ("double", "DOUBLE", DataTypeKind::Float),
        ("double precision", "DOUBLE PRECISION", DataTypeKind::Float),
        // --- Integer ---
        ("tinyint", "TINYINT", DataTypeKind::Integer),
        ("smallint", "SMALLINT", DataTypeKind::Integer),
        ("int", "INT", DataTypeKind::Integer),
        ("integer", "INTEGER", DataTypeKind::Integer),
        ("bigint", "BIGINT", DataTypeKind::Integer),
        // --- Boolean ---
        ("boolean", "BOOLEAN", DataTypeKind::Boolean),
        // --- Date ---
        ("date", "DATE", DataTypeKind::Date),
        // --- Time ---
        ("time", "TIME", DataTypeKind::Time),
        (
            "time with timezone",
            "TIME WITH TIMEZONE",
            DataTypeKind::Time,
        ),
        // --- Datetime ---
        ("datetime", "DATETIME", DataTypeKind::Datetime),
        ("timestamp", "TIMESTAMP", DataTypeKind::Datetime),
        (
            "timestamp without timezone",
            "TIMESTAMP",
            DataTypeKind::Datetime,
        ),
        (
            "timestamp with timezone",
            "TIMESTAMP WITH TIMEZONE",
            DataTypeKind::Datetime,
        ),
        (
            "timestamp with time zone",
            "TIMESTAMP WITH TIME ZONE",
            DataTypeKind::Datetime,
        ),
        ("timestamptz", "TIMESTAMPTZ", DataTypeKind::Datetime),
        // --- Json / Jsonb stay distinct groups ---
        ("json", "JSON", DataTypeKind::Json),
        ("jsonb", "JSONB", DataTypeKind::Jsonb),
        // --- Inet ---
        ("inet", "INET", DataTypeKind::Inet),
        // --- MacAddress (alias collapses onto the interned spelling) ---
        ("macaddress", "MACADDRESS", DataTypeKind::MacAddress),
        ("macaddr", "MACADDRESS", DataTypeKind::MacAddress),
    ];

    for (input, canonical, kind) in cases {
        let from_str = DataTypeName::from(input);
        assert_eq!(from_str.as_str(), canonical, "input {input:?}");
        assert_eq!(from_str.kind(), kind, "input {input:?}");

        let upper = input.to_ascii_uppercase();
        let from_upper = DataTypeName::from(upper.as_str());
        assert_eq!(from_upper.as_str(), canonical, "input {upper:?}");
        assert_eq!(from_upper.kind(), kind, "input {upper:?}");
        assert_eq!(from_str, from_upper, "case fold of {input:?}");

        let from_string = DataTypeName::from(input.to_string());
        assert_eq!(from_string, from_str, "From<String> of {input:?}");
        assert_eq!(from_string.kind(), kind);
    }
}

#[test]
fn interned_synonyms_compare_equal() {
    assert_eq!(
        DataTypeName::from("timestamp"),
        DataTypeName::from("timestamp without timezone"),
    );
    assert_eq!(
        DataTypeName::from("macaddress"),
        DataTypeName::from("MACADDR"),
    );
    // Distinct interned spellings stay distinct even when the kind matches.
    assert_ne!(
        DataTypeName::from("timestamp with timezone"),
        DataTypeName::from("timestamp with time zone"),
    );
    assert_ne!(DataTypeName::from("json"), DataTypeName::from("jsonb"));
    assert_ne!(DataTypeName::from("int"), DataTypeName::from("integer"));
}

#[test]
fn mixed_case_known_names_are_canonicalized() {
    let cases = [
        ("Int", "INT", DataTypeKind::Integer),
        ("InTeGeR", "INTEGER", DataTypeKind::Integer),
        ("VarChar", "VARCHAR", DataTypeKind::String),
        ("tExT", "TEXT", DataTypeKind::String),
        ("UuId", "UUID", DataTypeKind::Uuid),
        ("Boolean", "BOOLEAN", DataTypeKind::Boolean),
        ("TimeStampTz", "TIMESTAMPTZ", DataTypeKind::Datetime),
        (
            "Timestamp Without Timezone",
            "TIMESTAMP",
            DataTypeKind::Datetime,
        ),
        ("MacAddr", "MACADDRESS", DataTypeKind::MacAddress),
        ("JsonB", "JSONB", DataTypeKind::Jsonb),
        ("Double Precision", "DOUBLE PRECISION", DataTypeKind::Float),
        ("ByteA", "BYTEA", DataTypeKind::Bytes),
    ];
    for (input, canonical, kind) in cases {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), canonical, "input {input:?}");
        assert_eq!(dt.kind(), kind, "input {input:?}");
    }
}

#[test]
fn parameterized_name_is_custom_and_preserved() {
    for input in ["varchar(255)", "VARCHAR(255)", "decimal(10, 2)", "int(11)"] {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), input);
        assert_eq!(dt.kind(), DataTypeKind::Custom);
    }
}

#[test]
fn unknown_name_is_custom_and_preserved() {
    let dt = DataTypeName::from("some_totally_unknown_type");
    assert_eq!(dt.as_str(), "some_totally_unknown_type");
    assert_eq!(dt.kind(), DataTypeKind::Custom);
}

#[test]
fn from_string_reuses_unknown_allocation_path() {
    let dt = DataTypeName::from(String::from("not_a_real_type"));
    assert_eq!(dt.as_str(), "not_a_real_type");
    assert_eq!(dt.kind(), DataTypeKind::Custom);
}

/// Non-ASCII never hits the interned table: the lookup bails before matching,
/// and the caller's spelling (including unprintable scalars) is kept as Custom.
#[test]
fn unicode_and_unprintable_names_are_custom() {
    let cases = [
        "тип",
        "文字",
        "int🔥",
        // Cyrillic 'е' (U+0435), so this is not the interned "integer".
        "intеger",
        "varchar\u{200b}",
        "\u{0000}",
        "int\u{0000}",
        "\t",
        "\n",
        "\r\n",
        "\u{0001}",
        "\u{0007}",
        "\u{001b}",
        "\u{007f}",
        "bit\u{0008}varying",
    ];
    for input in cases {
        let from_str = DataTypeName::from(input);
        assert_eq!(from_str.as_str(), input, "input {input:?}");
        assert_eq!(from_str.kind(), DataTypeKind::Custom, "input {input:?}");

        let from_string = DataTypeName::from(input.to_string());
        assert_eq!(from_string.as_str(), input, "From<String> {input:?}");
        assert_eq!(from_string.kind(), DataTypeKind::Custom);
        assert_eq!(from_str, from_string);
    }
}
