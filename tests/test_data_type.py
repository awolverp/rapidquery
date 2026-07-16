import pytest

from rapidquery._native.data_type import (
    BitSerializer,
    BooleanSerializer,
    BytesSerializer,
    DataType,
    DateSerializer,
    DatetimeSerializer,
    FloatSerializer,
    InetSerializer,
    IntegerSerializer,
    JSONSerializer,
    MacAddressSerializer,
    Serializer,
    StringSerializer,
    TimeSerializer,
    UnspecifiedSerializer,
    UuidSerializer,
)
from rapidquery._native.dialect import PostgreSql


class TestDataTypeConstruction:
    def test_constructs_with_name_only(self):
        dt = DataType("integer")
        assert dt.name == "INTEGER" or dt.name.lower() == "integer"

    def test_constructs_with_explicit_serializer(self):
        dt = DataType("my_custom_type", StringSerializer())
        assert isinstance(dt.serializer, StringSerializer)

    def test_name_property_returns_str(self):
        dt = DataType("varchar")
        assert isinstance(dt.name, str)

    def test_serializer_property_returns_serializer_instance(self):
        dt = DataType("integer")
        assert isinstance(dt.serializer, Serializer)

    def test_explicit_serializer_overrides_inference(self):
        dt = DataType("integer", StringSerializer())
        assert isinstance(dt.serializer, StringSerializer)
        assert not isinstance(dt.serializer, IntegerSerializer)


class TestDataTypeNameNormalization:
    @pytest.mark.parametrize(
        "input_name,canonical",
        [
            ("int", "INT"),
            ("integer", "INTEGER"),
            ("varchar", "VARCHAR"),
            ("text", "TEXT"),
            ("uuid", "UUID"),
        ],
    )
    def test_known_names_are_uppercased_to_canonical_form(self, input_name, canonical):
        dt = DataType(input_name)
        assert dt.name == canonical

    def test_name_matching_is_case_insensitive(self):
        assert (
            DataType("INTEGER").serializer.__class__
            == DataType("integer").serializer.__class__
        )
        assert (
            DataType("Integer").serializer.__class__
            == DataType("integer").serializer.__class__
        )

    def test_unrecognized_name_is_preserved_as_is(self):
        dt = DataType("some_totally_unknown_type")
        assert dt.name == "some_totally_unknown_type"

    def test_parameterized_name_is_preserved_as_is(self):
        dt = DataType("VARCHAR(255)")
        assert dt.name == "VARCHAR(255)"

    def test_parameterized_name_is_not_alias_matched(self):
        dt = DataType("varchar(255)")
        assert dt.name == "varchar(255)"


class TestDataTypeSerializerInference:
    """`DataType(name).serializer` with no explicit serializer should infer
    the correct serializer type from the SQL type name's group.
    """

    @pytest.mark.parametrize(
        "name",
        [
            "tinyint",
            "smallint",
            "mediumint",
            "int",
            "integer",
            "bigint",
            "int8",
            "hugeint",
            "unsigned",
        ],
    )
    def test_integer_family_infers_integer_serializer(self, name):
        assert isinstance(DataType(name).serializer, IntegerSerializer)

    @pytest.mark.parametrize(
        "name",
        [
            "float",
            "double",
            "decimal",
            "numeric",
            "real",
            "double precision",
            "bignumeric",
        ],
    )
    def test_float_family_infers_float_serializer(self, name):
        assert isinstance(DataType(name).serializer, FloatSerializer)

    @pytest.mark.parametrize(
        "name",
        ["char", "varchar", "text", "clob", "nvarchar", "string", "longtext"],
    )
    def test_string_family_infers_string_serializer(self, name):
        assert isinstance(DataType(name).serializer, StringSerializer)

    def test_uuid_infers_uuid_serializer(self):
        assert isinstance(DataType("uuid").serializer, UuidSerializer)

    @pytest.mark.parametrize("name", ["binary", "varbinary", "blob", "bytes", "bytea"])
    def test_bytes_family_infers_bytes_serializer(self, name):
        assert isinstance(DataType(name).serializer, BytesSerializer)

    @pytest.mark.parametrize("name", ["bit", "bit varying", "varbit"])
    def test_bit_family_infers_bit_serializer(self, name):
        assert isinstance(DataType(name).serializer, BitSerializer)

    @pytest.mark.parametrize("name", ["date"])
    def test_date_infers_date_serializer(self, name):
        assert isinstance(DataType(name).serializer, DateSerializer)

    @pytest.mark.parametrize("name", ["time", "time with timezone"])
    def test_time_family_infers_time_serializer(self, name):
        assert isinstance(DataType(name).serializer, TimeSerializer)

    @pytest.mark.parametrize(
        "name", ["datetime", "timestamp", "timestamp with timezone", "timestampntz"]
    )
    def test_datetime_family_infers_datetime_serializer(self, name):
        assert isinstance(DataType(name).serializer, DatetimeSerializer)

    @pytest.mark.parametrize("name", ["json", "jsonb"])
    def test_json_family_infers_json_serializer(self, name):
        assert isinstance(DataType(name).serializer, JSONSerializer)

    def test_inet_infers_inet_serializer(self):
        assert isinstance(DataType("inet").serializer, InetSerializer)

    @pytest.mark.parametrize("name", ["macaddress", "macaddr"])
    def test_macaddress_family_infers_macaddress_serializer(self, name):
        assert isinstance(DataType(name).serializer, MacAddressSerializer)

    @pytest.mark.parametrize("name", ["any", "trigger", "unspecified"])
    def test_custom_group_names_infer_unspecified_serializer(self, name):
        assert isinstance(DataType(name).serializer, UnspecifiedSerializer)

    def test_unrecognized_name_infers_unspecified_serializer(self):
        assert isinstance(DataType("not_a_real_type").serializer, UnspecifiedSerializer)

    def test_parameterized_name_infers_unspecified_serializer(self):
        """Parameterized names skip the alias table entirely -> CUSTOM group
        -> UnspecifiedSerializer, even if the base name (e.g. 'varchar')
        would otherwise be recognized."""
        assert isinstance(DataType("varchar(255)").serializer, UnspecifiedSerializer)

    def test_boolean_infers_boolean_serializer(self):
        assert isinstance(DataType("boolean").serializer, BooleanSerializer)

    def test_bool_alias_infers_boolean_serializer(self):
        assert isinstance(DataType("bool").serializer, BooleanSerializer)

    def test_boolean_serializer_correctly_rejects_non_bool_int(self):
        dt = DataType("boolean")
        with pytest.raises(TypeError):
            dt.serialize(PostgreSql, 1)


class TestDataTypeSerializeDelegation:
    def test_serialize_delegates_to_serializer(self):
        dt = DataType("integer")
        assert dt.serialize(PostgreSql, 42) == "42"

    def test_serialize_with_explicit_custom_serializer(self):
        dt = DataType("weird_type", StringSerializer())
        result = dt.serialize(PostgreSql, "hi")
        assert "hi" in result

    def test_serialize_propagates_type_errors_from_serializer(self):
        dt = DataType("integer")
        with pytest.raises(TypeError):
            dt.serialize(PostgreSql, "not an int")

    def test_serialize_accepts_dialect_class_or_instance(self):
        dt = DataType("integer")
        assert dt.serialize(PostgreSql, 1) == dt.serialize(PostgreSql(), 1)
