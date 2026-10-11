import datetime
import decimal
import enum
import uuid

import pytest

from rapidquery.dialect import PostgreSql
from rapidquery.expr import Expr, Value
from rapidquery.serializers import (
    ArraySerializer,
    BitSerializer,
    BooleanSerializer,
    BytesSerializer,
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
from tests.conftest import USING_ORJSON


class ValidEnum(enum.StrEnum):
    TEST = "test"


class InvalidEnum(enum.Enum):
    TEST = 1


INFER_TEST_CASES: dict[type[Serializer], list[object]] = {
    IntegerSerializer: [1, 18_446_744_073_709_551_615, -1],
    FloatSerializer: [1.0, decimal.Decimal("1.0"), decimal.Decimal(1)],
    BooleanSerializer: [True, False],
    StringSerializer: ["string", ValidEnum.TEST],
    UuidSerializer: [uuid.UUID("58073209-2238-4d99-9e74-1bc21110393b")],
    BytesSerializer: [b"bytes"],
    BitSerializer: [],
    JSONSerializer: [[], {}, {"OK": 1}],
    DateSerializer: [datetime.date(2026, 10, 3)],
    TimeSerializer: [datetime.time(10, 23)],
    DatetimeSerializer: [datetime.datetime(2026, 10, 3, 23, 42, 34)],
    InetSerializer: [],
    MacAddressSerializer: [],
    UnspecifiedSerializer: [datetime.timezone.utc, InvalidEnum.TEST],
    ArraySerializer: [],
}

if USING_ORJSON:
    INFER_TEST_CASES[JSONSerializer].append({"OK": datetime.datetime(2026, 10, 3, 23, 42, 34)})
    INFER_TEST_CASES[JSONSerializer].append({"OK": uuid.UUID("58073209-2238-4d99-9e74-1bc21110393b")})


class TestValueConstruction:
    def test_new_overload_1(self):
        val = Value(1)
        assert isinstance(val, Expr)
        assert val.get_object() == 1
        assert isinstance(val.get_serializer(), IntegerSerializer)

    def test_new_overload_2(self):
        val = Value(1, InetSerializer())
        assert isinstance(val, Expr)
        assert val.get_object() == 1
        assert isinstance(val.get_serializer(), InetSerializer)

    @pytest.mark.parametrize(
        "serializer_cls, value",
        [
            pytest.param(cls, value, id=f"{cls.__name__}-{value!r}")
            for cls, values in INFER_TEST_CASES.items()
            for value in values
        ],
    )
    def test_infers(self, serializer_cls: type[Serializer], value: object):
        val = Value(value)
        assert val.get_object() == value
        assert isinstance(val.get_serializer(), serializer_cls)

    def test_wrap_inner(self):
        inner = Value(object(), UnspecifiedSerializer())
        outer = Value(inner)

        assert inner.get_object() is outer.get_object()
        assert inner.get_serializer() == outer.get_serializer()

    def test_wrap_trick_to_change_serializer(self):
        inner = Value(object(), InetSerializer())
        outer = Value(inner, UnspecifiedSerializer())

        assert inner.get_object() is outer.get_object()
        assert isinstance(inner.get_serializer(), InetSerializer)
        assert isinstance(outer.get_serializer(), UnspecifiedSerializer)


class TestValueToSql:
    def test_integer_literal(self):
        assert Value(42).to_sql(PostgreSql) == "42"

    def test_string_literal_matches_string_serializer(self):
        assert Value("hi").to_sql(PostgreSql) == StringSerializer().serialize(PostgreSql, "hi")

    def test_none_is_null(self):
        assert Value(None).to_sql(PostgreSql) == "NULL"

    def test_propagates_serializer_type_error(self):
        with pytest.raises(TypeError):
            Value("not an int", IntegerSerializer()).to_sql(PostgreSql)
