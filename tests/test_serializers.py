"""Serializer type checks, canonicalization, and literal spelling."""

import datetime
import enum
import ipaddress
import typing
import uuid
from dataclasses import dataclass
from decimal import Decimal

import pytest

from rapidquery.data_type import (
    ArraySerializer,
    BitSerializer,
    BooleanSerializer,
    BytesSerializer,
    CustomSerializer,
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
from rapidquery.dialect import (
    BigQuery,
    DuckDb,
    MsSql,
    MySql,
    Oracle,
    PostgreSql,
    Snowflake,
    SqlDialect,
    Sqlite,
)

from .conftest import ALL_DIALECT_NAMES, ALL_DIALECTS, USING_ORJSON

PG = PostgreSql()
BOOL_INT_NAMES = {"MsSql", "MySql", "Oracle", "Sqlite"}
BIT_SQL = {
    "PostgreSql": "B'10'",
    "DuckDb": "B'10'",
    "MySql": "b'10'",
    "Teradata": "'10'B",
}
ARRAY_SQL = {
    "PostgreSql": "ARRAY[1, 2, 3]",
    "BigQuery": "ARRAY[1, 2, 3]",
    "DuckDb": "[1, 2, 3]",
    "Spark": "array(1, 2, 3)",
    "Hive": "array(1, 2, 3)",
}


class InvalidStrEnum(enum.Enum):
    Variant = 1


@dataclass(frozen=True, slots=True)
class ExampleDataclass:
    x: int


def _custom_ok(dialect: SqlDialect, obj: typing.Any) -> str:
    assert isinstance(dialect, SqlDialect)
    return f"CUSTOM:{obj}"


def _custom_not_str(dialect: SqlDialect, obj: typing.Any) -> int:
    return 12345


class TestIntegerSerializer:
    def test_values(self):
        s = IntegerSerializer()
        assert s.serialize(PG, 1) == "1"
        assert s.serialize(PG, 18_446_744_073_709_551_615) == "18446744073709551615"
        assert s.serialize(PG, -9_223_372_036_854_775_808) == "-9223372036854775808"

    @pytest.mark.parametrize("value", [2.5, True, "invalid"])
    def test_rejects_non_int(self, value: typing.Any):
        with pytest.raises(TypeError):
            IntegerSerializer().serialize(PG, value)


class TestFloatSerializer:
    def test_values(self):
        s = FloatSerializer()
        assert s.serialize(PG, 1.1) == "1.1"
        assert s.serialize(PG, 1) == "1"
        assert s.serialize(PG, 1.0) == "1.0"
        assert s.serialize(PG, Decimal("1.3435")) == "1.3435"

    def test_rejects_non_numeric(self):
        with pytest.raises(TypeError):
            FloatSerializer().serialize(PG, "invalid")

    @pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
    def test_rejects_non_finite(self, value: float):
        with pytest.raises(ValueError):
            FloatSerializer().serialize(PG, value)


@pytest.mark.parametrize("dialect", ALL_DIALECTS, ids=ALL_DIALECT_NAMES)
def test_boolean_spelling(dialect: SqlDialect):
    s = BooleanSerializer()
    if type(dialect).__name__ in BOOL_INT_NAMES:
        assert s.serialize(dialect, True) == "1"
        assert s.serialize(dialect, False) == "0"
    else:
        assert s.serialize(dialect, True) == "TRUE"
        assert s.serialize(dialect, False) == "FALSE"


@pytest.mark.parametrize("value", [1, 0, object()])
def test_boolean_rejects_non_bool(value: typing.Any):
    with pytest.raises(TypeError):
        BooleanSerializer().serialize(PG, value)


@pytest.mark.parametrize("dialect", ALL_DIALECTS, ids=ALL_DIALECT_NAMES)
def test_string_obrien_and_nul(dialect: SqlDialect):
    s = StringSerializer()
    sql = s.serialize(dialect, "O'Brien")
    assert "O'Brien" not in sql
    assert "O''Brien" in sql or "O\\'Brien" in sql
    with pytest.raises(ValueError, match="NUL"):
        s.serialize(dialect, "a\0b")


@pytest.mark.parametrize("value", [InvalidStrEnum.Variant, 1, None, b"bytes"])
def test_string_rejects_non_str(value: typing.Any):
    with pytest.raises(TypeError):
        StringSerializer().serialize(PG, value)


class TestUuidSerializer:
    _TEXT = "103596b1-db75-4ae0-907a-a4a9bc4dab85"

    def test_uuid_and_str(self):
        s = UuidSerializer()
        expected = f"'{self._TEXT}'::uuid"
        assert s.serialize(PG, self._TEXT) == expected
        assert s.serialize(PG, uuid.UUID(self._TEXT)) == expected
        assert s.serialize(DuckDb(), self._TEXT) == f"UUID '{self._TEXT}'"
        assert s.serialize(Sqlite(), self._TEXT) == f"'{self._TEXT}'"

    def test_invalid_str(self):
        with pytest.raises(ValueError):
            UuidSerializer().serialize(PG, "103596-db75-4ae0-907a-a4a9bc4d85")

    def test_rejects_non_uuid(self):
        with pytest.raises(TypeError):
            UuidSerializer().serialize(PG, 6428739)


def test_bytes_pg_snowflake_duckdb():
    dead = b"\xde\xad"
    assert BytesSerializer().serialize(PG, dead) == "'\\xDEAD'::bytea"
    assert BytesSerializer().serialize(PG, b"") == "'\\x'::bytea"
    assert BytesSerializer().serialize(Snowflake(), dead) == "TO_BINARY('DEAD', 'HEX')"
    assert BytesSerializer().serialize(Snowflake(), b"") == "TO_BINARY('', 'HEX')"
    assert BytesSerializer().serialize(DuckDb(), dead) == "'\\xDE\\xAD'::BLOB"
    assert BytesSerializer().serialize(DuckDb(), b"") == "''::BLOB"


@pytest.mark.parametrize(
    "value", [bytearray(b"abc"), memoryview(b"abc"), "abc", 1, None]
)
def test_bytes_rejects_non_exact_bytes(value: typing.Any):
    with pytest.raises(TypeError):
        BytesSerializer().serialize(PG, value)


class TestBitSerializer:
    def test_follows_bit_format_when_supported(self, dialect_instance: SqlDialect):
        name = type(dialect_instance).__name__
        if name not in BIT_SQL:
            with pytest.raises(
                NotImplementedError, match="does not support BitSerializer"
            ):
                BitSerializer().serialize(dialect_instance, "10")
            return
        assert BitSerializer().serialize(dialect_instance, "10") == BIT_SQL[name]

    def test_rejects_non_bit_string(self):
        with pytest.raises(ValueError):
            BitSerializer().serialize(PG, "102")
        with pytest.raises(ValueError):
            BitSerializer().serialize(PG, "")
        with pytest.raises(TypeError):
            BitSerializer().serialize(PG, 10)


class TestJSONSerializer:
    def test_dumps_then_json_format(self):
        s = JSONSerializer()
        assert s.serialize(PG, 10) == "'10'::jsonb"
        assert s.serialize(PG, "10") == "'\"10\"'::jsonb"
        assert s.serialize(BigQuery(), {"a": 1}) == "JSON '{\"a\":1}'"
        assert s.serialize(Snowflake(), {"a": 1}) == "PARSE_JSON('{\"a\":1}')"
        assert s.serialize(Sqlite(), {"a": 1}) == "'{\"a\":1}'"

    def test_orjson_only_values(self):
        s = JSONSerializer()
        extras = [
            (
                datetime.datetime(2024, 10, 1, tzinfo=datetime.timezone.utc),
                "'\"2024-10-01T00:00:00+00:00\"'::jsonb",
            ),
            (
                uuid.UUID("103596b1-db75-4ae0-907a-a4a9bc4dab85"),
                "'\"103596b1-db75-4ae0-907a-a4a9bc4dab85\"'::jsonb",
            ),
            (ExampleDataclass(x=1), "'{\"x\":1}'::jsonb"),
        ]
        for obj, expected in extras:
            if USING_ORJSON:
                assert s.serialize(PG, obj) == expected
            else:
                with pytest.raises(TypeError):
                    s.serialize(PG, obj)


class TestDateTimeSerializers:
    def test_date(self):
        value = datetime.date(2024, 1, 15)
        assert DateSerializer().serialize(PG, value) == "DATE '2024-01-15'"
        assert DateSerializer().serialize(Oracle(), value) == "DATE '2024-01-15'"
        assert DateSerializer().serialize(Sqlite(), value) == "'2024-01-15'"

    def test_date_rejects_datetime_and_str(self):
        with pytest.raises(TypeError):
            DateSerializer().serialize(PG, datetime.datetime(2024, 1, 15, 10, 0))
        with pytest.raises(TypeError):
            DateSerializer().serialize(PG, "2024-01-15")

    def test_time(self):
        s = TimeSerializer()
        assert s.serialize(PG, datetime.time(13, 45, 30)) == "TIME '13:45:30'"
        assert (
            s.serialize(PG, datetime.time(1, 2, 3, 456789)) == "TIME '01:02:03.456789'"
        )
        assert s.serialize(Oracle(), datetime.time(13, 45, 30)) == "'13:45:30'"

    def test_time_rejects_date(self):
        with pytest.raises(TypeError):
            TimeSerializer().serialize(PG, datetime.date(2024, 1, 1))

    def test_datetime_naive_and_aware(self):
        s = DatetimeSerializer()
        naive = datetime.datetime(2024, 1, 15, 10, 30, 0)
        utc = datetime.datetime(2024, 1, 15, 10, 30, 0, tzinfo=datetime.timezone.utc)
        offset = datetime.datetime(
            2024,
            1,
            15,
            10,
            30,
            0,
            tzinfo=datetime.timezone(datetime.timedelta(hours=-5)),
        )
        assert s.serialize(PG, naive) == "TIMESTAMP '2024-01-15 10:30:00'"
        assert s.serialize(PG, utc) == "TIMESTAMP '2024-01-15 10:30:00+00:00'"
        assert s.serialize(PG, offset) == "TIMESTAMP '2024-01-15 10:30:00-05:00'"
        assert s.serialize(Oracle(), naive) == "TIMESTAMP '2024-01-15 10:30:00'"
        assert s.serialize(Sqlite(), naive) == "'2024-01-15 10:30:00'"

    def test_datetime_unix_timestamp(self):
        s = DatetimeSerializer()
        assert s.serialize(PG, 0) == "TIMESTAMP '1970-01-01 00:00:00+00:00'"
        assert s.serialize(PG, 0.5) == "TIMESTAMP '1970-01-01 00:00:00.500+00:00'"

    def test_datetime_rejects_date_and_str(self):
        with pytest.raises(TypeError):
            DatetimeSerializer().serialize(PG, datetime.date(2024, 1, 15))
        with pytest.raises(TypeError):
            DatetimeSerializer().serialize(PG, "2024-01-15T10:30:00")

    def test_datetime_overflow(self):
        with pytest.raises(OverflowError):
            DatetimeSerializer().serialize(PG, 10**20)


class TestInetSerializer:
    def test_canonical_cidr(self, dialect_instance: SqlDialect):
        name = type(dialect_instance).__name__
        if name not in {"PostgreSql", "DuckDb"}:
            with pytest.raises(
                NotImplementedError, match="does not support InetSerializer"
            ):
                InetSerializer().serialize(dialect_instance, "192.168.1.1")
            return
        s = InetSerializer()
        cases = [
            ("192.168.1.1", "192.168.1.1/32"),
            (ipaddress.ip_address("192.168.1.1"), "192.168.1.1/32"),
            ("192.168.1.0/24", "192.168.1.0/24"),
            ("::1", "::1/128"),
            ("2001:db8::/32", "2001:db8::/32"),
        ]
        for obj, inner in cases:
            assert s.serialize(dialect_instance, obj) == f"'{inner}'::inet"

    def test_invalid_address(self):
        with pytest.raises(ValueError):
            InetSerializer().serialize(PG, "not-an-ip")


class TestMacAddressSerializer:
    def test_canonical_form(self, dialect_instance: SqlDialect):
        if type(dialect_instance).__name__ != "PostgreSql":
            with pytest.raises(
                NotImplementedError, match="does not support MacAddressSerializer"
            ):
                MacAddressSerializer().serialize(dialect_instance, "01:23:45:67:89:ab")
            return
        s = MacAddressSerializer()
        expected = "'01:23:45:67:89:AB'::macaddr"
        assert s.serialize(dialect_instance, "01:23:45:67:89:ab") == expected
        assert s.serialize(dialect_instance, "01-23-45-67-89-AB") == expected

    def test_invalid_address(self):
        with pytest.raises(ValueError):
            MacAddressSerializer().serialize(PG, "not-a-mac")


class TestUnspecifiedSerializer:
    def test_uses_str(self):
        s = UnspecifiedSerializer()
        assert s.serialize(PG, 123) == "123"
        assert s.serialize(PG, "hello") == "hello"
        assert s.serialize(PG, None) == "None"


class TestArraySerializer:
    def test_renders_array_constructor(self, dialect_instance: SqlDialect):
        s = ArraySerializer(IntegerSerializer())
        name = type(dialect_instance).__name__
        expected = ARRAY_SQL.get(name)
        if expected is None:
            with pytest.raises(
                NotImplementedError, match="does not support ArraySerializer"
            ):
                s.serialize(dialect_instance, [1, 2, 3])
            return
        assert s.serialize(dialect_instance, [1, 2, 3]) == expected
        assert s.serialize(dialect_instance, (1, 2, 3)) == expected
        empty = (
            expected.replace("1, 2, 3", "")
            if "array(" in expected
            else expected.replace("1, 2, 3", "")
        )
        assert s.serialize(dialect_instance, []) == empty
        nested = ArraySerializer(ArraySerializer(IntegerSerializer()))
        inner = expected
        if name in {"PostgreSql", "BigQuery"}:
            assert nested.serialize(dialect_instance, [[1, 2], [3]]) == (
                "ARRAY[ARRAY[1, 2], ARRAY[3]]"
            )
        elif name == "DuckDb":
            assert nested.serialize(dialect_instance, [[1, 2], [3]]) == "[[1, 2], [3]]"
        elif name in {"Spark", "Hive"}:
            assert nested.serialize(dialect_instance, [[1, 2], [3]]) == (
                "array(array(1, 2), array(3))"
            )
        else:
            raise AssertionError(inner)

    def test_rejects_non_sequence_and_bad_element(self):
        s = ArraySerializer(IntegerSerializer())
        with pytest.raises(TypeError):
            s.serialize(PG, "not a list")
        with pytest.raises(TypeError):
            s.serialize(PG, [1, "x"])

    def test_nested_unsupported_element(self):
        with pytest.raises(NotImplementedError, match="does not support BitSerializer"):
            ArraySerializer(BitSerializer()).serialize(BigQuery(), ["10"])


class TestCustomSerializer:
    def test_receives_dialect_instance(self):
        dialect = DuckDb()
        assert CustomSerializer(_custom_ok).serialize(dialect, 1) == "CUSTOM:1"

    def test_must_return_str(self):
        with pytest.raises(TypeError):
            CustomSerializer(_custom_not_str).serialize(PG, "x")  # pyright: ignore


def test_unsupported_serializer_error_message():
    with pytest.raises(NotImplementedError, match="does not support BitSerializer"):
        BitSerializer().serialize(Sqlite(), "10")
    with pytest.raises(NotImplementedError, match="does not support ArraySerializer"):
        ArraySerializer(IntegerSerializer()).serialize(MySql(), [1, 2])
    with pytest.raises(NotImplementedError, match="does not support InetSerializer"):
        InetSerializer().serialize(Sqlite(), "192.168.1.1")
    with pytest.raises(
        NotImplementedError, match="does not support MacAddressSerializer"
    ):
        MacAddressSerializer().serialize(DuckDb(), "01:23:45:67:89:ab")


class TestLiteralGoldens:
    def test_pg_plain_string_skips_e_prefix(self):
        assert StringSerializer().serialize(PG, "hello") == "'hello'"
        assert StringSerializer().serialize(PG, "a\nb") == "E'a\\nb'"

    def test_mssql_nvarchar_prefix(self):
        assert StringSerializer().serialize(MsSql(), "hi") == "N'hi'"

    def test_pg_e_needed_for_backslash(self):
        assert StringSerializer().serialize(PG, "a\\b") == "E'a\\\\b'"
