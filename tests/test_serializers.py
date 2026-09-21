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
    Serializer,
    StringSerializer,
    TimeSerializer,
    UnspecifiedSerializer,
    UuidSerializer,
)
from rapidquery.dialect import (
    MSSQL,
    BigQuery,
    Databricks,
    DuckDB,
    Hive,
    MySql,
    Oracle,
    PostgreSQL,
    Redshift,
    Snowflake,
    Spark,
    SqlDialect,
    SQLite,
    Teradata,
)

from .conftest import ALL_DIALECTS, USING_ORJSON

# Quoted-literal wrap groups (payloads with no quotes/backslashes).
_PG = (PostgreSQL,)
_MSSQL = (MSSQL,)
_PLAIN_QUOTE = (
    BigQuery,
    Databricks,
    DuckDB,
    Hive,
    MySql,
    Oracle,
    Redshift,
    Snowflake,
    Spark,
    SQLite,
    Teradata,
)

# String-escape families — same partition as src/dialect/tests.rs.
_BACKSLASH_ESCAPING = (
    BigQuery,
    Databricks,
    Hive,
    MySql,
    PostgreSQL,
    Snowflake,
    Spark,
)
_QUOTE_ONLY = (
    DuckDB,
    MSSQL,
    Oracle,
    Redshift,
    SQLite,
    Teradata,
)
_NUL_STRIPPING = (MSSQL, Oracle, Redshift, SQLite, Teradata)
_NUL_X00 = (BigQuery, PostgreSQL)
_NUL_BACKSLASH_ZERO = (Databricks, Hive, MySql, Snowflake, Spark)
_DOUBLE_QUOTE_ESCAPING = (Databricks, MySql, Spark)
_HEX_CTRL = (BigQuery, PostgreSQL, Snowflake)
_UNICODE_CTRL = (Databricks, Hive, Spark)

_BIT_OK = (DuckDB, MySql, PostgreSQL, Teradata)
_BIT_NO = tuple(d for d in ALL_DIALECTS if d not in _BIT_OK)
_ARRAY_OK = (BigQuery, DuckDB, PostgreSQL)
_ARRAY_NO = tuple(d for d in ALL_DIALECTS if d not in _ARRAY_OK)
_INET_NO = tuple(d for d in ALL_DIALECTS if d not in (DuckDB, PostgreSQL))
_MAC_NO = tuple(d for d in ALL_DIALECTS if d is not PostgreSQL)


class ValidStrEnum(enum.StrEnum):
    HelloWorld = "Hello World"
    Sticker = "😀"
    WithQuote = "'OK'"
    Hex = "\x0d"


class InvalidStrEnum(enum.Enum):
    Variant = 1


@dataclass(frozen=True, slots=True)
class ExampleDataclass:
    x: int


def _custom_ok(dialect_name: str, obj: typing.Any) -> str:
    return f"CUSTOM:{dialect_name}:{obj}"


def _custom_not_str(dialect_name: str, obj: typing.Any) -> int:
    return 12345


def _quoted(dialect: type[SqlDialect] | SqlDialect, body: str) -> str:
    name = dialect if isinstance(dialect, type) else type(dialect)
    if name is PostgreSQL:
        return f"E'{body}'"
    if name is MSSQL:
        return f"N'{body}'"
    return f"'{body}'"


def _bytes_literal(dialect: type[SqlDialect] | SqlDialect, data: bytes) -> str:
    hx = data.hex().upper()
    name = dialect if isinstance(dialect, type) else type(dialect)
    if name is BigQuery:
        return "B'" + "".join(f"\\x{b:02X}" for b in data) + "'"
    if name is Hive:
        return f"UNHEX('{hx}')"
    if name is MSSQL:
        return f"0x{hx}"
    if name is Oracle:
        return f"HEXTORAW('{hx}')"
    if name is PostgreSQL:
        return f"'\\x{hx}'::bytea"
    if name is Redshift:
        return f"TO_VARBYTE('{hx}', 'hex')"
    if name is Teradata:
        return f"'{hx}'XBV"
    return f"X'{hx}'"


@dataclass(frozen=True, slots=True)
class SerializeTestCase:
    serializer: Serializer
    dialects: typing.Iterable[type[SqlDialect] | SqlDialect]
    input: typing.Any
    output: str | type[Exception]


SERIALIZE_TEST_CASES: list[SerializeTestCase] = [
    # --- Integer ---
    SerializeTestCase(IntegerSerializer(), ALL_DIALECTS, 1, "1"),
    SerializeTestCase(
        IntegerSerializer(),
        ALL_DIALECTS,
        18_446_744_073_709_551_615,
        "18446744073709551615",
    ),
    SerializeTestCase(
        IntegerSerializer(),
        ALL_DIALECTS,
        -9_223_372_036_854_775_808,
        "-9223372036854775808",
    ),
    SerializeTestCase(IntegerSerializer(), ALL_DIALECTS, 2.5, TypeError),
    SerializeTestCase(IntegerSerializer(), ALL_DIALECTS, True, TypeError),
    SerializeTestCase(IntegerSerializer(), ALL_DIALECTS, "invalid", TypeError),
    # --- Float ---
    SerializeTestCase(FloatSerializer(), ALL_DIALECTS, 1.1, "1.1"),
    SerializeTestCase(FloatSerializer(), ALL_DIALECTS, 1, "1"),
    SerializeTestCase(FloatSerializer(), ALL_DIALECTS, 1.0, "1.0"),
    SerializeTestCase(FloatSerializer(), ALL_DIALECTS, Decimal("1.3435"), "1.3435"),
    SerializeTestCase(FloatSerializer(), ALL_DIALECTS, "invalid", TypeError),
    # --- Boolean ---
    SerializeTestCase(
        BooleanSerializer(),
        (
            BigQuery,
            Databricks,
            DuckDB,
            Hive,
            PostgreSQL,
            Redshift,
            Snowflake,
            Spark,
            Teradata,
        ),
        True,
        "TRUE",
    ),
    SerializeTestCase(
        BooleanSerializer(),
        (MSSQL, MySql, Oracle, SQLite),
        True,
        "1",
    ),
    SerializeTestCase(
        BooleanSerializer(),
        (
            BigQuery,
            Databricks,
            DuckDB,
            Hive,
            PostgreSQL,
            Redshift,
            Snowflake,
            Spark,
            Teradata,
        ),
        False,
        "FALSE",
    ),
    SerializeTestCase(
        BooleanSerializer(),
        (MSSQL, MySql, Oracle, SQLite),
        False,
        "0",
    ),
    SerializeTestCase(BooleanSerializer(), ALL_DIALECTS, 1, TypeError),
    SerializeTestCase(BooleanSerializer(), ALL_DIALECTS, object(), TypeError),
    SerializeTestCase(BooleanSerializer(), ALL_DIALECTS, 0, TypeError),
    # --- String | PostgreSQL ---
    SerializeTestCase(
        StringSerializer(), [PostgreSQL], "Hello World", "E'Hello World'"
    ),
    SerializeTestCase(StringSerializer(), [PostgreSQL], "😀", "E'😀'"),
    SerializeTestCase(StringSerializer(), [PostgreSQL], "'OK'", "E'\\'OK\\''"),
    SerializeTestCase(StringSerializer(), [PostgreSQL], "\x0d", "E'\\r'"),
    # --- String | MSSQL ---
    SerializeTestCase(StringSerializer(), [MSSQL], "Hello World", "N'Hello World'"),
    SerializeTestCase(StringSerializer(), [MSSQL], "😀", "N'😀'"),
    SerializeTestCase(StringSerializer(), [MSSQL], "'OK'", "N'''OK'''"),
    SerializeTestCase(StringSerializer(), [MSSQL], "\x0d", "N'\r'"),
    # --- String | DuckDB - Oracle - Redshift - SQLite - Teradata ---
    SerializeTestCase(
        StringSerializer(),
        [DuckDB, Oracle, Redshift, SQLite, Teradata],
        "Hello World",
        "'Hello World'",
    ),
    SerializeTestCase(
        StringSerializer(), [DuckDB, Oracle, Redshift, SQLite, Teradata], "😀", "'😀'"
    ),
    SerializeTestCase(
        StringSerializer(),
        [DuckDB, Oracle, Redshift, SQLite, Teradata],
        "'OK'",
        "'''OK'''",
    ),
    SerializeTestCase(
        StringSerializer(), [DuckDB, Oracle, Redshift, SQLite, Teradata], "\x0d", "'\r'"
    ),
    # --- String ---
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        "Hello World",
        "'Hello World'",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        "😀",
        "'😀'",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        "'OK'",
        "'\\'OK\\''",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        "\x0d",
        "'\\r'",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        ValidStrEnum.HelloWorld,
        "'Hello World'",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        ValidStrEnum.Sticker,
        "'😀'",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        ValidStrEnum.WithQuote,
        "'\\'OK\\''",
    ),
    SerializeTestCase(
        StringSerializer(),
        (
            BigQuery,
            Databricks,
            Hive,
            MySql,
            Snowflake,
            Spark,
        ),
        ValidStrEnum.Hex,
        "'\\r'",
    ),
    SerializeTestCase(
        StringSerializer(),
        ALL_DIALECTS,
        InvalidStrEnum.Variant,
        TypeError,
    ),
    SerializeTestCase(
        StringSerializer(),
        ALL_DIALECTS,
        1,
        TypeError,
    ),
    # --- Uuid | PostgreSQL ---
    SerializeTestCase(
        UuidSerializer(),
        [PostgreSQL],
        "103596b1-db75-4ae0-907a-a4a9bc4dab85",
        "E'103596b1-db75-4ae0-907a-a4a9bc4dab85'",
    ),
    SerializeTestCase(
        UuidSerializer(),
        [PostgreSQL],
        uuid.UUID("103596b1-db75-4ae0-907a-a4a9bc4dab85"),
        "E'103596b1-db75-4ae0-907a-a4a9bc4dab85'",
    ),
    SerializeTestCase(
        UuidSerializer(),
        [PostgreSQL],
        "103596-db75-4ae0-907a-a4a9bc4d85",
        ValueError,
    ),
    SerializeTestCase(
        UuidSerializer(),
        [PostgreSQL],
        6428739,
        TypeError,
    ),
    # --- Uuid | MSSQL ---
    SerializeTestCase(
        UuidSerializer(),
        [MSSQL],
        "103596b1-db75-4ae0-907a-a4a9bc4dab85",
        "N'103596b1-db75-4ae0-907a-a4a9bc4dab85'",
    ),
    SerializeTestCase(
        UuidSerializer(),
        [MSSQL],
        uuid.UUID("103596b1-db75-4ae0-907a-a4a9bc4dab85"),
        "N'103596b1-db75-4ae0-907a-a4a9bc4dab85'",
    ),
    # --- Uuid ---
    SerializeTestCase(
        UuidSerializer(),
        [i for i in ALL_DIALECTS if i not in (MSSQL, PostgreSQL)],
        "103596b1-db75-4ae0-907a-a4a9bc4dab85",
        "'103596b1-db75-4ae0-907a-a4a9bc4dab85'",
    ),
    SerializeTestCase(
        UuidSerializer(),
        [i for i in ALL_DIALECTS if i not in (MSSQL, PostgreSQL)],
        uuid.UUID("103596b1-db75-4ae0-907a-a4a9bc4dab85"),
        "'103596b1-db75-4ae0-907a-a4a9bc4dab85'",
    ),
    # --- Bytes | BigQuery ---
    SerializeTestCase(
        BytesSerializer(), [BigQuery], b"simple", "B'\\x73\\x69\\x6D\\x70\\x6C\\x65'"
    ),
    # --- Bytes | X'...' format ---
    SerializeTestCase(
        BytesSerializer(),
        [Databricks, DuckDB, MySql, Snowflake, Spark, SQLite],
        b"simple",
        "X'73696D706C65'",
    ),
    # --- Bytes | Hive ---
    SerializeTestCase(BytesSerializer(), [Hive], b"simple", "UNHEX('73696D706C65')"),
    # --- Bytes | MSSQL ---
    SerializeTestCase(BytesSerializer(), [MSSQL], b"simple", "0x73696D706C65"),
    # --- Bytes | Oracle ---
    SerializeTestCase(
        BytesSerializer(), [Oracle], b"simple", "HEXTORAW('73696D706C65')"
    ),
    # --- Bytes | PostgreSQL ---
    SerializeTestCase(
        BytesSerializer(), [PostgreSQL], b"simple", "'\\x73696D706C65'::bytea"
    ),
    # --- Bytes | Redshift ---
    SerializeTestCase(
        BytesSerializer(), [Redshift], b"simple", "TO_VARBYTE('73696D706C65', 'hex')"
    ),
    # --- Bytes | Teradata ---
    SerializeTestCase(BytesSerializer(), [Teradata], b"simple", "'73696D706C65'XBV"),
    # --- Bit | MySql ---
    SerializeTestCase(BitSerializer(), [MySql], "10", "b'10'"),
    # --- Bit | Teradata ---
    SerializeTestCase(BitSerializer(), [Teradata], "10", "'10'B"),
    # --- Bit ---
    SerializeTestCase(BitSerializer(), [DuckDB, PostgreSQL], "10", "B'10'"),
    SerializeTestCase(BitSerializer(), _BIT_NO, "10", NotImplementedError),
    SerializeTestCase(BitSerializer(), _BIT_NO, 123, NotImplementedError),
    # --- JSON | PostgreSQL ---
    SerializeTestCase(JSONSerializer(), [PostgreSQL], 10, "E'10'"),
    SerializeTestCase(JSONSerializer(), [PostgreSQL], "10", """E'"10"'"""),
    SerializeTestCase(
        JSONSerializer(), [PostgreSQL], [1, "Ali", 1.0, {}], """E'[1,"Ali",1.0,{}]'"""
    ),
    SerializeTestCase(
        JSONSerializer(),
        [PostgreSQL],
        datetime.datetime(2024, 10, 1, tzinfo=datetime.timezone.utc),
        """E'"2024-10-01T00:00:00+00:00"'""" if USING_ORJSON else TypeError,
    ),
    SerializeTestCase(
        JSONSerializer(),
        [PostgreSQL],
        uuid.UUID("103596b1-db75-4ae0-907a-a4a9bc4dab85"),
        """E'"103596b1-db75-4ae0-907a-a4a9bc4dab85"'""" if USING_ORJSON else TypeError,
    ),
    SerializeTestCase(
        JSONSerializer(),
        [PostgreSQL],
        ExampleDataclass(x=1),
        """E'{"x":1}'""" if USING_ORJSON else TypeError,
    ),
    SerializeTestCase(
        JSONSerializer(),
        [PostgreSQL],
        ValidStrEnum.HelloWorld,
        """E'"Hello World"'""" if USING_ORJSON else TypeError,
    ),
    # --- Date ---
    SerializeTestCase(
        DateSerializer(), _PG, datetime.date(2024, 1, 15), "E'2024-01-15'"
    ),
    SerializeTestCase(
        DateSerializer(), _MSSQL, datetime.date(2024, 1, 15), "N'2024-01-15'"
    ),
    SerializeTestCase(
        DateSerializer(), _PLAIN_QUOTE, datetime.date(2024, 1, 15), "'2024-01-15'"
    ),
    SerializeTestCase(
        DateSerializer(),
        ALL_DIALECTS,
        datetime.datetime(2024, 1, 15, 10, 0),
        TypeError,
    ),
    SerializeTestCase(DateSerializer(), ALL_DIALECTS, "2024-01-15", TypeError),
    # --- Time ---
    SerializeTestCase(TimeSerializer(), _PG, datetime.time(13, 45, 30), "E'13:45:30'"),
    SerializeTestCase(
        TimeSerializer(), _MSSQL, datetime.time(13, 45, 30), "N'13:45:30'"
    ),
    SerializeTestCase(
        TimeSerializer(), _PLAIN_QUOTE, datetime.time(13, 45, 30), "'13:45:30'"
    ),
    SerializeTestCase(
        TimeSerializer(),
        _PG,
        datetime.time(1, 2, 3, 456789),
        "E'01:02:03.456789'",
    ),
    SerializeTestCase(
        TimeSerializer(),
        _MSSQL,
        datetime.time(1, 2, 3, 456789),
        "N'01:02:03.456789'",
    ),
    SerializeTestCase(
        TimeSerializer(),
        _PLAIN_QUOTE,
        datetime.time(1, 2, 3, 456789),
        "'01:02:03.456789'",
    ),
    SerializeTestCase(
        TimeSerializer(), ALL_DIALECTS, datetime.date(2024, 1, 1), TypeError
    ),
    # --- Datetime ---
    SerializeTestCase(
        DatetimeSerializer(),
        _PG,
        datetime.datetime(2024, 1, 15, 10, 30, 0),
        "E'2024-01-15 10:30:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _MSSQL,
        datetime.datetime(2024, 1, 15, 10, 30, 0),
        "N'2024-01-15 10:30:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _PLAIN_QUOTE,
        datetime.datetime(2024, 1, 15, 10, 30, 0),
        "'2024-01-15 10:30:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _PG,
        datetime.datetime(2024, 1, 15, 10, 30, 0, tzinfo=datetime.timezone.utc),
        "E'2024-01-15 10:30:00+00:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _MSSQL,
        datetime.datetime(2024, 1, 15, 10, 30, 0, tzinfo=datetime.timezone.utc),
        "N'2024-01-15 10:30:00+00:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _PLAIN_QUOTE,
        datetime.datetime(2024, 1, 15, 10, 30, 0, tzinfo=datetime.timezone.utc),
        "'2024-01-15 10:30:00+00:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _PG,
        datetime.datetime(
            2024,
            1,
            15,
            10,
            30,
            0,
            tzinfo=datetime.timezone(datetime.timedelta(hours=-5)),
        ),
        "E'2024-01-15 10:30:00-05:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _MSSQL,
        datetime.datetime(
            2024,
            1,
            15,
            10,
            30,
            0,
            tzinfo=datetime.timezone(datetime.timedelta(hours=-5)),
        ),
        "N'2024-01-15 10:30:00-05:00'",
    ),
    SerializeTestCase(
        DatetimeSerializer(),
        _PLAIN_QUOTE,
        datetime.datetime(
            2024,
            1,
            15,
            10,
            30,
            0,
            tzinfo=datetime.timezone(datetime.timedelta(hours=-5)),
        ),
        "'2024-01-15 10:30:00-05:00'",
    ),
    SerializeTestCase(DatetimeSerializer(), _PG, 0, "E'1970-01-01 00:00:00+00:00'"),
    SerializeTestCase(DatetimeSerializer(), _MSSQL, 0, "N'1970-01-01 00:00:00+00:00'"),
    SerializeTestCase(
        DatetimeSerializer(), _PLAIN_QUOTE, 0, "'1970-01-01 00:00:00+00:00'"
    ),
    SerializeTestCase(
        DatetimeSerializer(), _PG, 0.5, "E'1970-01-01 00:00:00.500+00:00'"
    ),
    SerializeTestCase(
        DatetimeSerializer(), _MSSQL, 0.5, "N'1970-01-01 00:00:00.500+00:00'"
    ),
    SerializeTestCase(
        DatetimeSerializer(), _PLAIN_QUOTE, 0.5, "'1970-01-01 00:00:00.500+00:00'"
    ),
    SerializeTestCase(
        DatetimeSerializer(), ALL_DIALECTS, datetime.date(2024, 1, 15), TypeError
    ),
    SerializeTestCase(
        DatetimeSerializer(), ALL_DIALECTS, "2024-01-15T10:30:00", TypeError
    ),
    SerializeTestCase(DatetimeSerializer(), ALL_DIALECTS, 10**20, OverflowError),
    # --- Inet ---
    SerializeTestCase(InetSerializer(), _PG, "192.168.1.1", "E'192.168.1.1/32'"),
    SerializeTestCase(InetSerializer(), [DuckDB], "192.168.1.1", "'192.168.1.1/32'"),
    SerializeTestCase(
        InetSerializer(),
        _PG,
        ipaddress.ip_address("192.168.1.1"),
        "E'192.168.1.1/32'",
    ),
    SerializeTestCase(
        InetSerializer(),
        [DuckDB],
        ipaddress.ip_address("192.168.1.1"),
        "'192.168.1.1/32'",
    ),
    SerializeTestCase(InetSerializer(), _PG, "192.168.1.0/24", "E'192.168.1.0/24'"),
    SerializeTestCase(InetSerializer(), [DuckDB], "192.168.1.0/24", "'192.168.1.0/24'"),
    SerializeTestCase(InetSerializer(), _PG, "::1", "E'::1/128'"),
    SerializeTestCase(InetSerializer(), [DuckDB], "::1", "'::1/128'"),
    SerializeTestCase(InetSerializer(), _PG, "2001:db8::/32", "E'2001:db8::/32'"),
    SerializeTestCase(InetSerializer(), [DuckDB], "2001:db8::/32", "'2001:db8::/32'"),
    SerializeTestCase(InetSerializer(), [PostgreSQL, DuckDB], "not-an-ip", ValueError),
    SerializeTestCase(InetSerializer(), _INET_NO, "192.168.1.1", NotImplementedError),
    # --- MacAddress ---
    SerializeTestCase(
        MacAddressSerializer(), _PG, "01:23:45:67:89:ab", "E'01:23:45:67:89:AB'"
    ),
    SerializeTestCase(
        MacAddressSerializer(), _PG, "01-23-45-67-89-AB", "E'01:23:45:67:89:AB'"
    ),
    SerializeTestCase(MacAddressSerializer(), [PostgreSQL], "not-a-mac", ValueError),
    SerializeTestCase(
        MacAddressSerializer(), _MAC_NO, "01:23:45:67:89:ab", NotImplementedError
    ),
    # --- Unspecified ---
    SerializeTestCase(UnspecifiedSerializer(), ALL_DIALECTS, 123, "123"),
    SerializeTestCase(UnspecifiedSerializer(), ALL_DIALECTS, "hello", "hello"),
    SerializeTestCase(UnspecifiedSerializer(), ALL_DIALECTS, None, "None"),
    # --- Array ---
    SerializeTestCase(
        ArraySerializer(IntegerSerializer()), _ARRAY_OK, [1, 2, 3], "ARRAY[1, 2, 3]"
    ),
    SerializeTestCase(
        ArraySerializer(IntegerSerializer()), _ARRAY_OK, (1, 2, 3), "ARRAY[1, 2, 3]"
    ),
    SerializeTestCase(ArraySerializer(IntegerSerializer()), _ARRAY_OK, [], "ARRAY[]"),
    SerializeTestCase(
        ArraySerializer(ArraySerializer(IntegerSerializer())),
        _ARRAY_OK,
        [[1, 2], [3]],
        "ARRAY[ARRAY[1, 2], ARRAY[3]]",
    ),
    SerializeTestCase(
        ArraySerializer(IntegerSerializer()), _ARRAY_OK, "not a list", TypeError
    ),
    SerializeTestCase(
        ArraySerializer(IntegerSerializer()), _ARRAY_OK, [1, "x"], TypeError
    ),
    SerializeTestCase(
        ArraySerializer(IntegerSerializer()),
        _ARRAY_NO,
        [1, 2, 3],
        NotImplementedError,
    ),
    # --- Custom ---
    SerializeTestCase(
        CustomSerializer(_custom_ok), [PostgreSQL], 1, "CUSTOM:PostgreSQL:1"
    ),
    SerializeTestCase(
        CustomSerializer(_custom_not_str),  # pyright: ignore
        ALL_DIALECTS,
        "x",
        TypeError,
    ),
]


@pytest.mark.parametrize("testcase", SERIALIZE_TEST_CASES)
def test_serialize(testcase: SerializeTestCase):
    for dialect in testcase.dialects:
        try:
            output = testcase.serializer.serialize(dialect, testcase.input)
        except Exception as error:
            if isinstance(testcase.output, str):
                raise

            assert type(error) is testcase.output

        else:
            assert output == testcase.output, (
                f"{testcase.serializer!r}({dialect!r}, {testcase.input!r}) == {output!r} != {testcase.output!r}"
            )


def test_serialize_cases_cover_all_serializers():
    assert {type(c.serializer) for c in SERIALIZE_TEST_CASES} == {
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
    }


def test_unsupported_serializer_error_message():
    with pytest.raises(
        NotImplementedError, match="SQLite does not support BitSerializer"
    ):
        BitSerializer().serialize(SQLite, "10")
    with pytest.raises(
        NotImplementedError, match="MySql does not support ArraySerializer"
    ):
        ArraySerializer(IntegerSerializer()).serialize(MySql, [1, 2])
    with pytest.raises(
        NotImplementedError, match="SQLite does not support InetSerializer"
    ):
        InetSerializer().serialize(SQLite, "192.168.1.1")
    with pytest.raises(
        NotImplementedError, match="DuckDB does not support MacAddressSerializer"
    ):
        MacAddressSerializer().serialize(DuckDB, "01:23:45:67:89:ab")
    with pytest.raises(
        NotImplementedError, match="BigQuery does not support BitSerializer"
    ):
        ArraySerializer(BitSerializer()).serialize(BigQuery, ["10"])


# ---------------------------------------------------------------------------
# String serialization escapes / edge cases
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("dialect", ALL_DIALECTS)
def test_string_empty_is_quoted(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "") == _quoted(dialect, "")


@pytest.mark.parametrize("dialect", _QUOTE_ONLY)
def test_string_quote_only_doubles_single_quote(dialect: type[SqlDialect]):
    s = StringSerializer()
    assert s.serialize(dialect, "O'Brien") == _quoted(dialect, "O''Brien")
    assert s.serialize(dialect, "''''") == _quoted(dialect, "''''''''")


@pytest.mark.parametrize("dialect", _BACKSLASH_ESCAPING)
def test_string_backslash_family_escapes_single_quote(dialect: type[SqlDialect]):
    s = StringSerializer()
    assert s.serialize(dialect, "O'Brien") == _quoted(dialect, "O\\'Brien")
    assert s.serialize(dialect, "''''") == _quoted(dialect, "\\'\\'\\'\\'")


@pytest.mark.parametrize("dialect", _QUOTE_ONLY)
def test_string_quote_only_passes_backslash_through(dialect: type[SqlDialect]):
    s = StringSerializer()
    assert s.serialize(dialect, "\\") == _quoted(dialect, "\\")
    assert s.serialize(dialect, "\\'") == _quoted(dialect, "\\''")


@pytest.mark.parametrize("dialect", _BACKSLASH_ESCAPING)
def test_string_backslash_family_doubles_backslash(dialect: type[SqlDialect]):
    s = StringSerializer()
    assert s.serialize(dialect, "\\") == _quoted(dialect, "\\\\")
    assert s.serialize(dialect, "\\'") == _quoted(dialect, "\\\\\\'")


@pytest.mark.parametrize("dialect", _NUL_STRIPPING)
def test_string_nul_is_stripped(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "a\0b") == _quoted(dialect, "ab")


@pytest.mark.parametrize("dialect", _NUL_X00)
def test_string_nul_is_hex_escaped(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "a\0b") == _quoted(dialect, "a\\x00b")


@pytest.mark.parametrize("dialect", _NUL_BACKSLASH_ZERO)
def test_string_nul_is_backslash_zero(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "a\0b") == _quoted(dialect, "a\\0b")


def test_string_nul_duckdb_passthrough():
    assert StringSerializer().serialize(DuckDB, "a\0b") == "'a\0b'"


@pytest.mark.parametrize("dialect", _BACKSLASH_ESCAPING)
def test_string_backslash_family_escapes_newline_and_cr(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "a\nb\rc") == _quoted(
        dialect, "a\\nb\\rc"
    )


@pytest.mark.parametrize("dialect", _QUOTE_ONLY)
def test_string_quote_only_passes_newline_cr_tab_through(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "a\nb\rc\td") == _quoted(
        dialect, "a\nb\rc\td"
    )


@pytest.mark.parametrize(
    "dialect",
    [d for d in _BACKSLASH_ESCAPING if d is not MySql],
)
def test_string_backslash_family_escapes_tab(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "a\tb") == _quoted(dialect, "a\\tb")


def test_string_mysql_passes_tab_through():
    assert StringSerializer().serialize(MySql, "a\tb") == "'a\tb'"


@pytest.mark.parametrize("dialect", _HEX_CTRL)
def test_string_other_controls_hex_escape(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "\x01\x1a\x7f") == _quoted(
        dialect, "\\x01\\x1A\\x7F"
    )


@pytest.mark.parametrize("dialect", _UNICODE_CTRL)
def test_string_other_controls_unicode_escape(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "\x01\x1a\x7f") == _quoted(
        dialect, "\\u0001\\u001A\\u007F"
    )


def test_string_mysql_ctrl_z_and_raw_other_controls():
    assert StringSerializer().serialize(MySql, "\x01\x1a\x7f") == "'\x01\\Z\x7f'"


@pytest.mark.parametrize("dialect", _QUOTE_ONLY)
def test_string_quote_only_passes_other_controls_through(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "\x01\x1a\x7f") == _quoted(
        dialect, "\x01\x1a\x7f"
    )


@pytest.mark.parametrize("dialect", _DOUBLE_QUOTE_ESCAPING)
def test_string_double_quote_is_escaped(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, 'say "hi"') == _quoted(
        dialect, 'say \\"hi\\"'
    )


@pytest.mark.parametrize(
    "dialect",
    [d for d in ALL_DIALECTS if d not in _DOUBLE_QUOTE_ESCAPING],
)
def test_string_double_quote_is_literal(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, 'say "hi"') == _quoted(
        dialect, 'say "hi"'
    )


@pytest.mark.parametrize("dialect", _QUOTE_ONLY)
def test_string_injection_quote_only(dialect: type[SqlDialect]):
    out = StringSerializer().serialize(dialect, "' OR 1=1 --")
    assert out == _quoted(dialect, "'' OR 1=1 --")
    assert " OR 1=1 --" in out


@pytest.mark.parametrize("dialect", _BACKSLASH_ESCAPING)
def test_string_injection_backslash_family(dialect: type[SqlDialect]):
    out = StringSerializer().serialize(dialect, "' OR 1=1 --")
    assert out == _quoted(dialect, "\\' OR 1=1 --")


@pytest.mark.parametrize("dialect", ALL_DIALECTS)
def test_string_unicode_passthrough(dialect: type[SqlDialect]):
    assert StringSerializer().serialize(dialect, "日本語😀") == _quoted(
        dialect, "日本語😀"
    )


@pytest.mark.parametrize("value", [123, b"bytes", None])
@pytest.mark.parametrize("dialect", ALL_DIALECTS)
def test_string_rejects_non_str(dialect: type[SqlDialect], value: typing.Any):
    with pytest.raises(TypeError):
        StringSerializer().serialize(dialect, value)


# ---------------------------------------------------------------------------
# Bytes serialization escapes / edge cases
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("dialect", ALL_DIALECTS)
@pytest.mark.parametrize(
    "data",
    [b"", b"\x00", b"\xff", b"\x00\x01\xff"],
)
def test_bytes_hex_wrappers(dialect: type[SqlDialect], data: bytes):
    assert BytesSerializer().serialize(dialect, data) == _bytes_literal(dialect, data)


@pytest.mark.parametrize("dialect", ALL_DIALECTS)
def test_bytes_all_256_values_uppercase_hex(dialect: type[SqlDialect]):
    data = bytes(range(256))
    out = BytesSerializer().serialize(dialect, data)
    assert out == _bytes_literal(dialect, data)
    if dialect is BigQuery:
        expected_run = "".join(f"\\x{b:02X}" for b in data)
    else:
        expected_run = "".join(f"{b:02X}" for b in data)

    assert expected_run in out


@pytest.mark.parametrize(
    "value",
    [bytearray(b"abc"), memoryview(b"abc"), "abc", 1, None],
)
@pytest.mark.parametrize("dialect", ALL_DIALECTS)
def test_bytes_rejects_non_exact_bytes(dialect: type[SqlDialect], value: typing.Any):
    with pytest.raises(TypeError):
        BytesSerializer().serialize(dialect, value)
