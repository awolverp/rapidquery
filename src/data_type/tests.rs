use super::*;

fn assert_copy<T: Copy>(_: T) {}

#[test]
fn data_type_group_is_copy() {
    assert_copy(DataTypeKind::Integer);
}

#[test]
fn known_names_are_canonicalized() {
    let cases = [
        ("int", "INT", DataTypeKind::Integer),
        ("INT", "INT", DataTypeKind::Integer),
        ("Integer", "INTEGER", DataTypeKind::Integer),
        ("integer", "INTEGER", DataTypeKind::Integer),
        ("varchar", "VARCHAR", DataTypeKind::String),
        ("text", "TEXT", DataTypeKind::String),
        ("uuid", "UUID", DataTypeKind::Uuid),
        ("bool", "BOOLEAN", DataTypeKind::Boolean),
        ("boolean", "BOOLEAN", DataTypeKind::Boolean),
        (
            "timestamptz",
            "TIMESTAMP WITH TIMEZONE",
            DataTypeKind::Datetime,
        ),
        (
            "timestamp without timezone",
            "TIMESTAMP",
            DataTypeKind::Datetime,
        ),
        ("macaddr", "MACADDRESS", DataTypeKind::MacAddress),
        ("jsonb", "JSONB", DataTypeKind::Jsonb),
        (
            "double precision unsigned",
            "DOUBLE PRECISION UNSIGNED",
            DataTypeKind::Float,
        ),
    ];
    for (input, canonical, group) in cases {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), canonical, "input {input:?}");
        assert_eq!(dt.kind(), group, "input {input:?}");
    }
}

#[test]
fn character_varying_typo_and_correct_spelling() {
    let correct = DataTypeName::from("character varying");
    let typo = DataTypeName::from("characater varying");
    assert_eq!(correct.as_str(), "CHARACTER VARYING");
    assert_eq!(typo.as_str(), "CHARACTER VARYING");
    assert_eq!(correct.kind(), DataTypeKind::String);
    assert_eq!(correct, typo);
}

#[test]
fn unsigned_bigint_typo_and_correct_spelling() {
    let correct = DataTypeName::from("unsigned bigint");
    let typo = DataTypeName::from("unsigned bitint");
    let alias = DataTypeName::from("ubigint");
    assert_eq!(correct.as_str(), "BIGINT UNSIGNED");
    assert_eq!(typo.as_str(), "BIGINT UNSIGNED");
    assert_eq!(alias.as_str(), "BIGINT UNSIGNED");
    assert_eq!(correct.kind(), DataTypeKind::Integer);
}

#[test]
fn parameterized_name_is_custom_and_preserved() {
    for input in ["varchar(255)", "VARCHAR(255)", "decimal(10, 2)"] {
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

#[test]
fn non_ascii_name_is_custom() {
    let dt = DataTypeName::from("тип");
    assert_eq!(dt.as_str(), "тип");
    assert_eq!(dt.kind(), DataTypeKind::Custom);
}
