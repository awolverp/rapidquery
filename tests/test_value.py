import pytest

from rapidquery import Value
from rapidquery.data_type import (
    DataType,
    IntegerSerializer,
    JSONSerializer,
    StringSerializer,
)
from rapidquery.dialect import PostgreSQL


class TestValueConstruction:
    def test_infers_integer_serializer(self):
        value = Value(42)
        assert value.object == 42
        assert isinstance(value.serializer, IntegerSerializer)

    def test_infers_string_serializer(self):
        value = Value("hi")
        assert value.object == "hi"
        assert isinstance(value.serializer, StringSerializer)

    def test_infers_boolean_for_bool_and_none(self):
        assert type(Value(True).serializer).__name__ == "BooleanSerializer"
        assert type(Value(None).serializer).__name__ == "BooleanSerializer"

    def test_infers_json_for_list_and_dict(self):
        assert isinstance(Value([1, 2]).serializer, JSONSerializer)
        assert isinstance(Value({"a": 1}).serializer, JSONSerializer)

    def test_infers_bytes_serializer(self):
        value = Value(b"abc")
        assert value.object == b"abc"
        assert type(value.serializer).__name__ == "BytesSerializer"

    def test_explicit_serializer_overrides_inference(self):
        value = Value(1, StringSerializer())
        assert isinstance(value.serializer, StringSerializer)

    def test_wraps_existing_value_and_keeps_serializer(self):
        inner = Value(1, StringSerializer())
        outer = Value(inner)
        assert outer.object == 1
        assert isinstance(outer.serializer, StringSerializer)

    def test_wraps_existing_value_with_new_serializer(self):
        inner = Value(1)
        outer = Value(inner, StringSerializer())
        assert outer.object == 1
        assert isinstance(outer.serializer, StringSerializer)

    def test_data_type_supplies_serializer_only(self):
        value = Value(1, DataType("int"))
        assert isinstance(value.serializer, IntegerSerializer)
        assert not hasattr(value, "data_type")

    def test_data_type_serializer_override(self):
        value = Value("x", DataType("integer", StringSerializer()))
        assert isinstance(value.serializer, StringSerializer)

    def test_rejects_non_serializer_second_arg(self):
        with pytest.raises(TypeError, match="Serializer or DataType"):
            _ = Value(1, "integer")


class TestValueToSql:
    def test_integer_literal(self):
        assert Value(42).to_sql(PostgreSQL) == "42"

    def test_string_literal_matches_string_serializer(self):
        assert Value("hi").to_sql(PostgreSQL) == StringSerializer().serialize(
            PostgreSQL, "hi"
        )

    def test_none_is_null(self):
        assert Value(None).to_sql(PostgreSQL) == "NULL"

    def test_accepts_dialect_class_or_instance(self):
        value = Value(1)
        assert value.to_sql(PostgreSQL) == value.to_sql(PostgreSQL())

    def test_propagates_serializer_type_error(self):
        with pytest.raises(TypeError):
            Value("not an int", IntegerSerializer()).to_sql(PostgreSQL)


class TestValueEquality:
    def test_equal_when_object_and_serializer_match(self):
        assert Value(1) == Value(1)
        assert hash(Value(1)) == hash(Value(1))

    def test_unequal_when_serializer_differs(self):
        assert Value(1) != Value(1, StringSerializer())

    def test_unequal_when_object_differs(self):
        assert Value(1) != Value(2)

    def test_not_equal_to_bare_object(self):
        assert Value(1) != 1

    def test_unhashable_object_raises(self):
        with pytest.raises(TypeError):
            hash(Value({"a": 1}))

    def test_repr_includes_object_and_serializer(self):
        assert repr(Value(42)) == "Value(42, IntegerSerializer())"
