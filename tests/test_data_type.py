import pytest

from rapidquery.data_type import (
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
    StringSerializer,
    TimeSerializer,
    UnspecifiedSerializer,
    UuidSerializer,
)
from rapidquery.dialect import PostgreSql


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
    @pytest.mark.parametrize(
        "name",
        [
            "tinyint",
            "tinyint unsigned",
            "utinyint",
            "int2",
            "int2 unsigned",
            "smallint",
            "smallint unsigned",
            "usmallint",
            "mediumint",
            "mediumint unsigned",
            "int",
            "int4",
            "int8",
            "int64",
            "integer",
            "unsigned integer",
            "integer unsigned",
            "int unsigned",
            "int4 unsigned",
            "hugeint",
            "uhugeint",
            "bigint",
            "bigint unsigned",
            "unsigned bigint",
            "unsigned bitint",
            "ubigint",
            "int8 unsigned",
            "signed",
            "signed integer",
            "unsigned",
        ],
    )
    def test_integer_family_infers_integer_serializer(self, name):
        assert isinstance(DataType(name).serializer, IntegerSerializer)

    @pytest.mark.parametrize(
        "name",
        [
            "numeric",
            "decimal",
            "bignumeric",
            "bigdecimal",
            "dec",
            "float",
            "float4",
            "float8",
            "float64",
            "real",
            "real unsigned",
            "double",
            "double unsigned",
            "double precision",
            "double precision unsigned",
        ],
    )
    def test_float_family_infers_float_serializer(self, name):
        assert isinstance(DataType(name).serializer, FloatSerializer)

    @pytest.mark.parametrize(
        "name",
        [
            "character",
            "char",
            "character varying",
            "characater varying",
            "char varying",
            "varchar",
            "nvarchar",
            "character large object",
            "char large object",
            "clob",
            "regclass",
            "text",
            "tinytext",
            "mediumtext",
            "longtext",
            "string",
        ],
    )
    def test_string_family_infers_string_serializer(self, name):
        assert isinstance(DataType(name).serializer, StringSerializer)

    def test_uuid_infers_uuid_serializer(self):
        assert isinstance(DataType("uuid").serializer, UuidSerializer)

    @pytest.mark.parametrize(
        "name",
        [
            "binary",
            "varbinary",
            "blob",
            "tinyblob",
            "mediumblob",
            "longblob",
            "bytes",
            "bytea",
        ],
    )
    def test_bytes_family_infers_bytes_serializer(self, name):
        assert isinstance(DataType(name).serializer, BytesSerializer)

    @pytest.mark.parametrize("name", ["bit", "bit varying", "varbit"])
    def test_bit_family_infers_bit_serializer(self, name):
        assert isinstance(DataType(name).serializer, BitSerializer)

    def test_date_infers_date_serializer(self):
        assert isinstance(DataType("date").serializer, DateSerializer)

    @pytest.mark.parametrize(
        "name", ["time", "time without timezone", "time with timezone"]
    )
    def test_time_family_infers_time_serializer(self, name):
        assert isinstance(DataType(name).serializer, TimeSerializer)

    @pytest.mark.parametrize(
        "name",
        [
            "datetime",
            "timestamp",
            "timestamp without timezone",
            "timestamp with timezone",
            "timestamptz",
            "timestampntz",
        ],
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
        assert isinstance(DataType("varchar(255)").serializer, UnspecifiedSerializer)

    @pytest.mark.parametrize("name", ["boolean", "bool"])
    def test_boolean_family_infers_boolean_serializer(self, name):
        assert isinstance(DataType(name).serializer, BooleanSerializer)

    def test_boolean_serializer_correctly_rejects_non_bool_int(self):
        dt = DataType("boolean")
        with pytest.raises(TypeError):
            dt.serialize(PostgreSql(), 1)


class TestDataTypeSerializeDelegation:
    def test_serialize_delegates_to_serializer(self):
        dt = DataType("integer")
        assert dt.serialize(PostgreSql(), 42) == "42"

    def test_serialize_with_explicit_custom_serializer(self):
        dt = DataType("weird_type", StringSerializer())
        result = dt.serialize(PostgreSql(), "hi")
        assert "hi" in result

    def test_serialize_propagates_type_errors_from_serializer(self):
        dt = DataType("integer")
        with pytest.raises(TypeError):
            dt.serialize(PostgreSql(), "not an int")


class TestDataTypeEquality:
    def test_interned_spellings_of_the_same_name_are_equal(self):
        assert DataType("int") == DataType("INT")
        assert hash(DataType("int")) == hash(DataType("INT"))

    def test_distinct_interned_names_are_not_equal(self):
        assert DataType("int") != DataType("integer")

    def test_explicit_serializer_participates_in_equality(self):
        assert DataType("int", StringSerializer()) != DataType("int")

    def test_not_equal_to_unrelated_types(self):
        assert DataType("int") != "INT"
        assert DataType("int") != IntegerSerializer()
