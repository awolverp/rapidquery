use super::*;

fn assert_copy<T: Copy>(_: T) {}

#[test]
fn data_type_group_is_copy() {
    assert_copy(DataTypeGroup::Integer);
}

#[test]
fn known_names_are_canonicalized() {
    let cases = [
        ("int", "INT", DataTypeGroup::Integer),
        ("INT", "INT", DataTypeGroup::Integer),
        ("Integer", "INTEGER", DataTypeGroup::Integer),
        ("integer", "INTEGER", DataTypeGroup::Integer),
        ("varchar", "VARCHAR", DataTypeGroup::String),
        ("text", "TEXT", DataTypeGroup::String),
        ("uuid", "UUID", DataTypeGroup::Uuid),
        ("bool", "BOOLEAN", DataTypeGroup::Boolean),
        ("boolean", "BOOLEAN", DataTypeGroup::Boolean),
        (
            "timestamptz",
            "TIMESTAMP WITH TIMEZONE",
            DataTypeGroup::Datetime,
        ),
        (
            "timestamp without timezone",
            "TIMESTAMP",
            DataTypeGroup::Datetime,
        ),
        ("macaddr", "MACADDRESS", DataTypeGroup::MacAddress),
        ("jsonb", "JSONB", DataTypeGroup::Jsonb),
        (
            "double precision unsigned",
            "DOUBLE PRECISION UNSIGNED",
            DataTypeGroup::Float,
        ),
    ];
    for (input, canonical, group) in cases {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), canonical, "input {input:?}");
        assert_eq!(dt.group(), group, "input {input:?}");
    }
}

#[test]
fn character_varying_typo_and_correct_spelling() {
    let correct = DataTypeName::from("character varying");
    let typo = DataTypeName::from("characater varying");
    assert_eq!(correct.as_str(), "CHARACTER VARYING");
    assert_eq!(typo.as_str(), "CHARACTER VARYING");
    assert_eq!(correct.group(), DataTypeGroup::String);
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
    assert_eq!(correct.group(), DataTypeGroup::Integer);
}

#[test]
fn parameterized_name_is_custom_and_preserved() {
    for input in ["varchar(255)", "VARCHAR(255)", "decimal(10, 2)"] {
        let dt = DataTypeName::from(input);
        assert_eq!(dt.as_str(), input);
        assert_eq!(dt.group(), DataTypeGroup::Custom);
    }
}

#[test]
fn unknown_name_is_custom_and_preserved() {
    let dt = DataTypeName::from("some_totally_unknown_type");
    assert_eq!(dt.as_str(), "some_totally_unknown_type");
    assert_eq!(dt.group(), DataTypeGroup::Custom);
}

#[test]
fn from_string_reuses_unknown_allocation_path() {
    let dt = DataTypeName::from(String::from("not_a_real_type"));
    assert_eq!(dt.as_str(), "not_a_real_type");
    assert_eq!(dt.group(), DataTypeGroup::Custom);
}

// #[test]
// fn inferred_serializer_matches_group() {
//     let cases = [
//         (DataTypeGroup::String, InferredSerializer::String),
//         (DataTypeGroup::Uuid, InferredSerializer::Uuid),
//         (DataTypeGroup::Bytes, InferredSerializer::Bytes),
//         (DataTypeGroup::Bit, InferredSerializer::Bit),
//         (DataTypeGroup::Integer, InferredSerializer::Integer),
//         (DataTypeGroup::Float, InferredSerializer::Float),
//         (DataTypeGroup::Boolean, InferredSerializer::Boolean),
//         (DataTypeGroup::Date, InferredSerializer::Date),
//         (DataTypeGroup::Time, InferredSerializer::Time),
//         (DataTypeGroup::Datetime, InferredSerializer::Datetime),
//         (DataTypeGroup::Json, InferredSerializer::Json),
//         (DataTypeGroup::Jsonb, InferredSerializer::Json),
//         (DataTypeGroup::Inet, InferredSerializer::Inet),
//         (DataTypeGroup::MacAddress, InferredSerializer::MacAddress),
//         (DataTypeGroup::Custom, InferredSerializer::Unspecified),
//     ];
//     for (group, expected) in cases {
//         assert_eq!(group.inferred_serializer(), expected);
//     }
// }

#[test]
fn non_ascii_name_is_custom() {
    let dt = DataTypeName::from("тип");
    assert_eq!(dt.as_str(), "тип");
    assert_eq!(dt.group(), DataTypeGroup::Custom);
}
