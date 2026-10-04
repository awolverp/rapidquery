use super::*;

#[test]
fn all_interned_names_canonicalize() {
    let cases = [
        // --- String ---
        ("character", "CHARACTER"),
        ("char", "CHAR"),
        ("character varying", "CHARACTER VARYING"),
        ("char varying", "CHAR VARYING"),
        ("varchar", "VARCHAR"),
        ("nvarchar", "NVARCHAR"),
        ("text", "TEXT"),
        // --- Uuid ---
        ("uuid", "UUID"),
        // --- Bytes ---
        ("binary", "BINARY"),
        ("varbinary", "VARBINARY"),
        ("blob", "BLOB"),
        ("bytea", "BYTEA"),
        // --- Bit ---
        ("bit", "BIT"),
        ("bit varying", "BIT VARYING"),
        // --- Float (includes decimal/numeric) ---
        ("numeric", "NUMERIC"),
        ("decimal", "DECIMAL"),
        ("dec", "DEC"),
        ("float", "FLOAT"),
        ("real", "REAL"),
        ("double", "DOUBLE"),
        ("double precision", "DOUBLE PRECISION"),
        // --- Integer ---
        ("tinyint", "TINYINT"),
        ("smallint", "SMALLINT"),
        ("int", "INT"),
        ("integer", "INTEGER"),
        ("bigint", "BIGINT"),
        // --- Boolean ---
        ("boolean", "BOOLEAN"),
        // --- Date ---
        ("date", "DATE"),
        // --- Time ---
        ("time", "TIME"),
        ("time with timezone", "TIME WITH TIMEZONE"),
        // --- Datetime ---
        ("datetime", "DATETIME"),
        ("timestamp", "TIMESTAMP"),
        ("timestamp without timezone", "TIMESTAMP"),
        ("timestamp with timezone", "TIMESTAMP WITH TIMEZONE"),
        ("timestamp with time zone", "TIMESTAMP WITH TIME ZONE"),
        ("timestamptz", "TIMESTAMPTZ"),
        // --- Json / Jsonb stay distinct spellings ---
        ("json", "JSON"),
        ("jsonb", "JSONB"),
        // --- Inet ---
        ("inet", "INET"),
        // --- MacAddress (alias collapses onto the interned spelling) ---
        ("macaddress", "MACADDRESS"),
        ("macaddr", "MACADDRESS"),
    ];

    for (input, canonical) in cases {
        let from_str = DataTypeName::from(input);
        assert_eq!(from_str.as_str(), canonical, "input {input:?}");

        let upper = input.to_ascii_uppercase();
        let from_upper = DataTypeName::from(upper.as_str());
        assert_eq!(from_upper.as_str(), canonical, "input {upper:?}");
        assert_eq!(from_str, from_upper, "case fold of {input:?}");

        let from_string = DataTypeName::from(input.to_string());
        assert_eq!(from_string, from_str, "From<String> of {input:?}");
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
    // Distinct interned spellings stay distinct.
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
        ("Int", "INT"),
        ("InTeGeR", "INTEGER"),
        ("VarChar", "VARCHAR"),
        ("tExT", "TEXT"),
        ("UuId", "UUID"),
        ("Boolean", "BOOLEAN"),
        ("TimeStampTz", "TIMESTAMPTZ"),
        ("Timestamp Without Timezone", "TIMESTAMP"),
        ("MacAddr", "MACADDRESS"),
        ("JsonB", "JSONB"),
        ("Double Precision", "DOUBLE PRECISION"),
        ("ByteA", "BYTEA"),
    ];
    for (input, canonical) in cases {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), canonical, "input {input:?}");
    }
}

#[test]
fn parameterized_name_is_custom_and_preserved() {
    for input in ["varchar(255)", "VARCHAR(255)", "decimal(10, 2)", "int(11)"] {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), input);
    }
}

#[test]
fn unknown_name_is_custom_and_preserved() {
    let dt = DataTypeName::from("some_totally_unknown_type");
    assert_eq!(dt.as_str(), "some_totally_unknown_type");
}

#[test]
fn from_string_reuses_unknown_allocation_path() {
    let dt = DataTypeName::from(String::from("not_a_real_type"));
    assert_eq!(dt.as_str(), "not_a_real_type");
}

/// Non-ASCII never hits the interned table: the lookup bails before matching,
/// and the caller's spelling (including unprintable scalars) is kept.
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

        let from_string = DataTypeName::from(input.to_string());
        assert_eq!(from_string.as_str(), input, "From<String> {input:?}");
        assert_eq!(from_str, from_string);
    }
}
