"""Serializer type checks and literal spelling for a configured dialect."""

import datetime
import ipaddress
import json
import typing
import uuid
from dataclasses import dataclass
from decimal import Decimal

import pytest
from hypothesis import given, reproduce_failure
from hypothesis import strategies as st

from rapidquery.dialect import OPT_BOOL_FORMAT_INTEGER, Dialect, LiteralConfig, StringConfig, TemporalConfig
from rapidquery.exc import DialectError, SerializeError
from rapidquery.serializers import (
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

from .conftest import USING_ORJSON

_UUID = "103596b1-db75-4ae0-907a-a4a9bc4dab85"
_DATE = datetime.date(2024, 1, 15)
_TIME = datetime.time(13, 45, 30)
_TIME_FRAC = datetime.time(1, 2, 3, 456789)
_NAIVE = datetime.datetime(2024, 1, 15, 10, 30, 0)
_OFFSET = datetime.datetime(2024, 1, 15, 10, 30, 0, tzinfo=datetime.timezone(datetime.timedelta(hours=-5)))


@dataclass(frozen=True, slots=True)
class ExampleDataclass:
    x: int


def _literal(**kwargs: typing.Any) -> Dialect:
    return Dialect(literal=LiteralConfig(**kwargs))


def _temporal(**kwargs: str) -> Dialect:
    return Dialect(temporal=TemporalConfig(**kwargs))


_DIALECT = Dialect()
_BOOL_INT = _literal(boolean=OPT_BOOL_FORMAT_INTEGER)
_SINGLE = Dialect(string=StringConfig(template="'{single}'"))
_UUID_DIALECT = _literal(uuid="{}")
_BYTES = _literal(bytes="X'{hex}'")
_BIT = _literal(bit="B'{}'")
_DATE_TMPL = _temporal(date="{}")
_TIME_TMPL = _temporal(time="{}")
_NAIVE_TMPL = _temporal(datetime_naive="{}")
_UNIX_TMPL = _temporal(datetime_aware="{}")
_INET = _literal(inet="'{}'::inet")
_MAC = _literal(macaddr="E'{}'::macaddr")
_ARRAY = _literal(array="ARRAY[{}]")
_JSON = Dialect(string=StringConfig(template="'{single}'"), literal=LiteralConfig(json="{}"))

_NOT_INT = st.one_of(st.floats(allow_nan=False), st.text(), st.none(), st.binary())
_NOT_STR = st.one_of(st.integers(), st.none(), st.binary())
_NOT_BYTES = st.one_of(st.text(), st.integers(), st.none(), st.binary().map(bytearray))
_BAD_BIT = st.one_of(st.just(""), st.text(alphabet="23456789", min_size=1))
_PRINTABLE_KEY = st.text(alphabet=st.characters(min_codepoint=32, max_codepoint=126), min_size=1)


class TestIntegerSerializer:
    def test_values(self):
        serializer = IntegerSerializer()
        dialect = Dialect()
        assert serializer.serialize(dialect, 1) == "1"
        assert serializer.serialize(dialect, 18_446_744_073_709_551_615) == "18446744073709551615"
        assert serializer.serialize(dialect, -9_223_372_036_854_775_808) == "-9223372036854775808"
        assert serializer.serialize(dialect, True) == "1"
        assert serializer.serialize(dialect, False) == "0"

    @given(st.integers(min_value=-(10**30), max_value=10**30))
    def test_integers_match_str(self, value: int):
        assert IntegerSerializer().serialize(_DIALECT, value) == str(value)

    @given(st.booleans())
    def test_bools_are_digits(self, value: bool):
        assert IntegerSerializer().serialize(_DIALECT, value) == ("1" if value else "0")

    @given(_NOT_INT)
    def test_rejects_non_int(self, value: typing.Any):
        with pytest.raises(SerializeError):
            IntegerSerializer().serialize(_DIALECT, value)


class TestFloatSerializer:
    def test_values(self):
        serializer = FloatSerializer()
        dialect = Dialect()
        assert serializer.serialize(dialect, 1.1) == "1.1"
        assert serializer.serialize(dialect, 1) == "1"
        assert serializer.serialize(dialect, 1.0) == "1.0"
        assert serializer.serialize(dialect, Decimal("1.3435")) == "1.3435"

    @given(st.floats(allow_nan=False, allow_infinity=False))
    def test_finite_floats_render(self, value: float):
        text = FloatSerializer().serialize(_DIALECT, value)
        assert "nan" not in text.lower()
        assert "inf" not in text.lower()

    @given(st.sampled_from([float("nan"), float("inf"), float("-inf")]))
    def test_rejects_non_finite(self, value: float):
        with pytest.raises(SerializeError):
            FloatSerializer().serialize(_DIALECT, value)

    @given(st.integers(min_value=-(10**20), max_value=10**20))
    def test_integers_match_str(self, value: int):
        assert FloatSerializer().serialize(_DIALECT, value) == str(value)

    @given(st.decimals(min_value=-(10**20), max_value=10**20, allow_nan=False, allow_infinity=False))
    def test_decimals_match_str(self, value: Decimal):
        assert FloatSerializer().serialize(_DIALECT, value) == str(value)

    @given(st.text())
    def test_rejects_non_numeric(self, value: str):
        with pytest.raises(SerializeError):
            FloatSerializer().serialize(_DIALECT, value)


class TestBooleanSerializer:
    @given(st.booleans())
    def test_keyword(self, value: bool):
        expected = "TRUE" if value else "FALSE"
        assert BooleanSerializer().serialize(_DIALECT, value) == expected

    @given(st.booleans())
    def test_integer(self, value: bool):
        assert BooleanSerializer().serialize(_BOOL_INT, value) == ("1" if value else "0")

    @given(st.one_of(st.integers(), st.text()))
    def test_rejects_non_bool(self, value: typing.Any):
        with pytest.raises(SerializeError):
            BooleanSerializer().serialize(_DIALECT, value)


# (template, value, sql). One template per escape key. `E` prefixes the hex key.
_STRING_CASES: list[tuple[str, str, str]] = [
    ("'{single}'", "hello", "'hello'"),
    ("'{single}'", "O'Brien", "'O''Brien'"),
    ("'{single}'", "a\\b", r"'a\b'"),
    ("'{single}'", "a\nb", "'a\nb'"),
    ("'{single}'", 'say "hi"', "'say \"hi\"'"),
    ("'{single}'", "\t", "'\t'"),
    ("'{single}'", "\x01", "'\x01'"),
    ("'{single}'", "\x1a", "'\x1a'"),
    ('"{double}"', "hello", '"hello"'),
    ('"{double}"', "O'Brien", '"O\'Brien"'),
    ('"{double}"', "a\\b", r'"a\b"'),
    ('"{double}"', "a\nb", '"a\nb"'),
    ('"{double}"', 'say "hi"', '"say ""hi"""'),
    ('"{double}"', "\t", '"\t"'),
    ('"{double}"', "\x01", '"\x01"'),
    ('"{double}"', "\x1a", '"\x1a"'),
    ("E'{hex}'", "hello", "E'hello'"),
    ("E'{hex}'", "O'Brien", r"E'O\'Brien'"),
    ("E'{hex}'", "a\\b", r"E'a\\b'"),
    ("E'{hex}'", "a\nb", r"E'a\nb'"),
    ("E'{hex}'", 'say "hi"', "E'say \"hi\"'"),
    ("E'{hex}'", "\t", r"E'\t'"),
    ("E'{hex}'", "\x01", r"E'\x01'"),
    ("E'{hex}'", "\x1a", r"E'\x1A'"),
    ('"{hex_quote}"', "hello", '"hello"'),
    ('"{hex_quote}"', "O'Brien", r'"O\'Brien"'),
    ('"{hex_quote}"', "a\\b", r'"a\\b"'),
    ('"{hex_quote}"', "a\nb", r'"a\nb"'),
    ('"{hex_quote}"', 'say "hi"', r'"say \"hi\""'),
    ('"{hex_quote}"', "\t", r'"\t"'),
    ('"{hex_quote}"', "\x01", r'"\x01"'),
    ('"{hex_quote}"', "\x1a", r'"\x1A"'),
    ("'{mysql}'", "hello", "'hello'"),
    ("'{mysql}'", "O'Brien", r"'O\'Brien'"),
    ("'{mysql}'", "a\\b", r"'a\\b'"),
    ("'{mysql}'", "a\nb", r"'a\nb'"),
    ("'{mysql}'", 'say "hi"', r"'say \"hi\"'"),
    ("'{mysql}'", "\t", "'\t'"),
    ("'{mysql}'", "\x01", "'\x01'"),
    ("'{mysql}'", "\x1a", r"'\Z'"),
    ("'{unicode}'", "hello", "'hello'"),
    ("'{unicode}'", "O'Brien", r"'O\'Brien'"),
    ("'{unicode}'", "a\\b", r"'a\\b'"),
    ("'{unicode}'", "a\nb", r"'a\nb'"),
    ("'{unicode}'", 'say "hi"', "'say \"hi\"'"),
    ("'{unicode}'", "\t", r"'\t'"),
    ("'{unicode}'", "\x01", r"'\u0001'"),
    ("'{unicode}'", "\x1a", r"'\u001A'"),
    ("'{unicode_quote}'", "hello", "'hello'"),
    ("'{unicode_quote}'", "O'Brien", r"'O\'Brien'"),
    ("'{unicode_quote}'", "a\\b", r"'a\\b'"),
    ("'{unicode_quote}'", "a\nb", r"'a\nb'"),
    ("'{unicode_quote}'", 'say "hi"', r"'say \"hi\"'"),
    ("'{unicode_quote}'", "\t", r"'\t'"),
    ("'{unicode_quote}'", "\x01", r"'\u0001'"),
    ("'{unicode_quote}'", "\x1a", r"'\u001A'"),
]


class TestStringSerializer:
    @pytest.mark.parametrize(("template", "value", "expected"), _STRING_CASES)
    def test_escape_key(self, template: str, value: str, expected: str):
        dialect = Dialect(string=StringConfig(template=template))
        assert StringSerializer().serialize(dialect, value) == expected

    @given(st.text().filter(lambda value: "\0" not in value))
    def test_single_quotes_round_trip(self, value: str):
        rendered = StringSerializer().serialize(_SINGLE, value)
        assert rendered[0] == rendered[-1] == "'"
        assert rendered[1:-1].replace("''", "'") == value

    @given(st.text(), st.text())
    def test_rejects_nul(self, left: str, right: str):
        with pytest.raises(SerializeError, match="NUL"):
            StringSerializer().serialize(_SINGLE, left + "\0" + right)

    @given(_NOT_STR)
    def test_rejects_non_str(self, value: typing.Any):
        with pytest.raises(SerializeError):
            StringSerializer().serialize(_DIALECT, value)


class TestUuidSerializer:
    @pytest.mark.parametrize("template", ["'{}'", "E'{}'::uuid", "UUID '{}'", "N'{}'"])
    def test_str_and_uuid(self, template: str):
        dialect = _literal(uuid=template)
        expected = template.replace("{}", _UUID)
        serializer = UuidSerializer()
        assert serializer.serialize(dialect, _UUID) == expected
        assert serializer.serialize(dialect, uuid.UUID(_UUID)) == expected

    @given(st.uuids())
    def test_uuids_match_str(self, value: uuid.UUID):
        assert UuidSerializer().serialize(_UUID_DIALECT, value) == str(value)

    @given(st.uuids())
    def test_unsupported(self, value: uuid.UUID):
        with pytest.raises(DialectError, match="UUID literal"):
            UuidSerializer().serialize(_DIALECT, value)

    @given(st.text(alphabet="xyz", min_size=1))
    def test_invalid_str(self, value: str):
        with pytest.raises(SerializeError):
            UuidSerializer().serialize(_UUID_DIALECT, value)

    @given(st.integers())
    def test_rejects_non_uuid(self, value: int):
        with pytest.raises(SerializeError):
            UuidSerializer().serialize(_UUID_DIALECT, value)


@pytest.mark.parametrize(
    ("template", "dead", "empty"),
    [
        ("X'{hex}'", "X'DEAD'", "X''"),
        (r"'\x{hex}'::bytea", r"'\xDEAD'::bytea", r"'\x'::bytea"),
        ("0x{hex}", "0xDEAD", "0x"),
        ("UNHEX('{hex}')", "UNHEX('DEAD')", "UNHEX('')"),
        ("HEXTORAW('{hex}')", "HEXTORAW('DEAD')", "HEXTORAW('')"),
        ("TO_VARBYTE('{hex}', 'hex')", "TO_VARBYTE('DEAD', 'hex')", "TO_VARBYTE('', 'hex')"),
        ("TO_BINARY('{hex}', 'HEX')", "TO_BINARY('DEAD', 'HEX')", "TO_BINARY('', 'HEX')"),
        ("'{hex}'XBV", "'DEAD'XBV", "''XBV"),
        ("B'{slashx}'", r"B'\xDE\xAD'", "B''"),
        ("'{slashx}'::BLOB", r"'\xDE\xAD'::BLOB", "''::BLOB"),
    ],
)
def test_bytes_template(template: str, dead: str, empty: str):
    dialect = _literal(bytes=template)
    serializer = BytesSerializer()
    assert serializer.serialize(dialect, b"\xde\xad") == dead
    assert serializer.serialize(dialect, b"") == empty


@given(st.binary())
def test_bytes_are_uppercase_hex(value: bytes):
    assert BytesSerializer().serialize(_BYTES, value) == "X'" + value.hex().upper() + "'"


@given(_NOT_BYTES)
def test_bytes_rejects_non_exact_bytes(value: typing.Any):
    with pytest.raises(SerializeError):
        BytesSerializer().serialize(_DIALECT, value)


class TestBitSerializer:
    @pytest.mark.parametrize(
        ("template", "expected"),
        [("B'{}'", "B'10'"), ("b'{}'", "b'10'"), ("'{}'B", "'10'B")],
    )
    def test_embeds_bits(self, template: str, expected: str):
        assert BitSerializer().serialize(_literal(bit=template), "10") == expected

    @given(st.text(alphabet="01", min_size=1))
    def test_embeds_generated_bits(self, value: str):
        assert BitSerializer().serialize(_BIT, value) == f"B'{value}'"

    @given(st.text(alphabet="01", min_size=1))
    def test_unsupported(self, value: str):
        with pytest.raises(DialectError, match="bit string literal"):
            BitSerializer().serialize(_DIALECT, value)

    @given(_BAD_BIT)
    def test_rejects_non_bit_string(self, value: str):
        with pytest.raises(SerializeError):
            BitSerializer().serialize(_BIT, value)

    @given(st.integers())
    def test_rejects_non_str(self, value: int):
        with pytest.raises(SerializeError):
            BitSerializer().serialize(_BIT, value)


class TestJSONSerializer:
    @pytest.mark.parametrize(
        ("string_template", "json_template", "obj", "expected"),
        [
            ("E'{hex}'", "{}", 10, "E'10'"),
            ("E'{hex}'", "{}", "10", "E'\"10\"'"),
            ("'{single}'", "{}", {"a": 1}, "'{\"a\":1}'"),
            ("N'{single}'", "{}", {"a": 1}, "N'{\"a\":1}'"),
            ("'{hex}'", "JSON {}", {"a": 1}, "JSON '{\"a\":1}'"),
            ("'{hex}'", "PARSE_JSON({})", {"a": 1}, "PARSE_JSON('{\"a\":1}')"),
            ("'{mysql}'", "{}", {"a": 1}, r"'{\"a\":1}'"),
            ("'{unicode_quote}'", "{}", {"a": 1}, r"'{\"a\":1}'"),
            ('"{hex_quote}"', "{}", {"a": 1}, r'"{\"a\":1}"'),
            ("'{unicode}'", "{}", {"a": 1}, "'{\"a\":1}'"),
        ],
    )
    def test_dumps_then_wraps(self, string_template: str, json_template: str, obj: typing.Any, expected: str):
        dialect = Dialect(string=StringConfig(template=string_template), literal=LiteralConfig(json=json_template))
        assert JSONSerializer().serialize(dialect, obj) == expected

    @given(st.dictionaries(_PRINTABLE_KEY, st.integers(), max_size=5))
    def test_printable_objects_match_compact_json(self, value: dict[str, int]):
        dumped = json.dumps(value, separators=(",", ":"), ensure_ascii=True).replace("'", "''")
        assert JSONSerializer().serialize(_JSON, value) == f"'{dumped}'"

    def test_orjson_only_values(self):
        dialect = Dialect(string=StringConfig(template="E'{hex}'"))
        serializer = JSONSerializer()
        extras = [
            (
                datetime.datetime(2024, 10, 1, tzinfo=datetime.timezone.utc),
                "E'\"2024-10-01T00:00:00+00:00\"'",
            ),
            (uuid.UUID(_UUID), f"E'\"{_UUID}\"'"),
            (ExampleDataclass(x=1), "E'{\"x\":1}'"),
        ]
        for obj, expected in extras:
            if USING_ORJSON:
                assert serializer.serialize(dialect, obj) == expected
            else:
                with pytest.raises(TypeError):
                    serializer.serialize(dialect, obj)


class TestDateTimeSerializers:
    @pytest.mark.parametrize(
        ("template", "expected"),
        [
            ("DATE '{}'", "DATE '2024-01-15'"),
            ("'{}'", "'2024-01-15'"),
            ("DATE E'{}'", "DATE E'2024-01-15'"),
            ("N'{}'", "N'2024-01-15'"),
        ],
    )
    def test_date(self, template: str, expected: str):
        assert DateSerializer().serialize(_temporal(date=template), _DATE) == expected

    @given(st.dates())
    def test_dates_match_isoformat(self, value: datetime.date):
        assert DateSerializer().serialize(_DATE_TMPL, value) == value.isoformat()

    @given(st.datetimes() | st.text())
    def test_date_rejects_datetime_and_str(self, value: datetime.datetime | str):
        with pytest.raises(SerializeError):
            DateSerializer().serialize(_DIALECT, value)

    @pytest.mark.parametrize(
        ("template", "expected"),
        [
            ("TIME '{}'", "TIME '13:45:30'"),
            ("'{}'", "'13:45:30'"),
            ("TIME E'{}'", "TIME E'13:45:30'"),
            ("N'{}'", "N'13:45:30'"),
        ],
    )
    def test_time(self, template: str, expected: str):
        assert TimeSerializer().serialize(_temporal(time=template), _TIME) == expected

    def test_time_fraction(self):
        dialect = _temporal(time="TIME E'{}'")
        assert TimeSerializer().serialize(dialect, _TIME_FRAC) == "TIME E'01:02:03.456789'"

    @given(st.times())
    def test_times_start_with_hms(self, value: datetime.time):
        rendered = TimeSerializer().serialize(_TIME_TMPL, value)
        assert rendered.startswith(value.strftime("%H:%M:%S"))

    @given(st.dates())
    def test_time_rejects_date(self, value: datetime.date):
        with pytest.raises(SerializeError):
            TimeSerializer().serialize(_DIALECT, value)

    @pytest.mark.parametrize(
        ("template", "expected"),
        [
            ("TIMESTAMP '{}'", "TIMESTAMP '2024-01-15 10:30:00'"),
            ("'{}'", "'2024-01-15 10:30:00'"),
            ("TIMESTAMP E'{}'", "TIMESTAMP E'2024-01-15 10:30:00'"),
            ("N'{}'", "N'2024-01-15 10:30:00'"),
        ],
    )
    def test_datetime_naive(self, template: str, expected: str):
        assert DatetimeSerializer().serialize(_temporal(datetime_naive=template), _NAIVE) == expected

    @pytest.mark.parametrize(
        ("template", "expected"),
        [
            ("TIMESTAMPTZ '{}'", "TIMESTAMPTZ '2024-01-15 10:30:00-05:00'"),
            ("TIMESTAMP WITH TIMEZONE '{}'", "TIMESTAMP WITH TIMEZONE '2024-01-15 10:30:00-05:00'"),
            ("TIMESTAMPTZ E'{}'", "TIMESTAMPTZ E'2024-01-15 10:30:00-05:00'"),
            ("N'{}'", "N'2024-01-15 10:30:00-05:00'"),
        ],
    )
    def test_datetime_aware(self, template: str, expected: str):
        assert DatetimeSerializer().serialize(_temporal(datetime_aware=template), _OFFSET) == expected

    def test_datetime_unix_timestamp(self):
        dialect = _temporal(datetime_aware="TIMESTAMPTZ E'{}'")
        serializer = DatetimeSerializer()
        assert serializer.serialize(dialect, 0) == "TIMESTAMPTZ E'1970-01-01 00:00:00+00:00'"
        assert serializer.serialize(dialect, 0.5) == "TIMESTAMPTZ E'1970-01-01 00:00:00.500+00:00'"

    @given(st.datetimes(timezones=st.none()))
    def test_naive_datetimes_start_with_timestamp(self, value: datetime.datetime):
        rendered = DatetimeSerializer().serialize(_NAIVE_TMPL, value)

        strftime = value.strftime("%Y-%m-%d %H:%M:%S")

        # - On Windows / macOS .strftime(...) usually zero-pads years → 0001-01-01.
        # - On Linux (glibc) .strftime(...) does not pad years → 1-01-01
        strftime = strftime.lstrip("0")

        assert rendered.startswith(strftime)

    @given(st.integers(min_value=-2_000_000_000, max_value=2_000_000_000))
    def test_unix_timestamps_in_range(self, value: int):
        rendered = DatetimeSerializer().serialize(_UNIX_TMPL, value)
        assert len(rendered) >= 19

    @given(st.integers(min_value=10**18, max_value=10**30))
    def test_datetime_overflow(self, value: int):
        with pytest.raises(SerializeError, match="out of range"):
            DatetimeSerializer().serialize(_UNIX_TMPL, value)

    @given(st.dates() | st.text())
    def test_datetime_rejects_date_and_str(self, value: datetime.date | str):
        with pytest.raises(SerializeError):
            DatetimeSerializer().serialize(_DIALECT, value)


class TestInetSerializer:
    @pytest.mark.parametrize("template", ["E'{}'::inet", "'{}'::inet"])
    @pytest.mark.parametrize(
        ("obj", "inner"),
        [
            ("192.168.1.1", "192.168.1.1/32"),
            (ipaddress.ip_address("192.168.1.1"), "192.168.1.1/32"),
            ("192.168.1.0/24", "192.168.1.0/24"),
            ("::1", "::1/128"),
            ("2001:db8::/32", "2001:db8::/32"),
        ],
    )
    def test_canonical_cidr(self, template: str, obj: typing.Any, inner: str):
        assert InetSerializer().serialize(_literal(inet=template), obj) == template.replace("{}", inner)

    @given(st.ip_addresses())
    def test_addresses_use_host_prefix(self, value: ipaddress.IPv4Address | ipaddress.IPv6Address):
        prefix = 32 if isinstance(value, ipaddress.IPv4Address) else 128
        assert InetSerializer().serialize(_INET, value) == f"'{value}/{prefix}'::inet"

    @given(st.ip_addresses())
    def test_unsupported(self, value: ipaddress.IPv4Address | ipaddress.IPv6Address):
        with pytest.raises(DialectError, match="INET literal"):
            InetSerializer().serialize(_DIALECT, str(value))

    @given(st.text(alphabet="xyz", min_size=1))
    def test_invalid_address(self, value: str):
        with pytest.raises(SerializeError):
            InetSerializer().serialize(_INET, value)


class TestMacAddressSerializer:
    def test_canonical_form(self):
        dialect = _literal(macaddr="E'{}'::macaddr")
        serializer = MacAddressSerializer()
        expected = "E'01:23:45:67:89:AB'::macaddr"
        assert serializer.serialize(dialect, "01:23:45:67:89:ab") == expected
        assert serializer.serialize(dialect, "01-23-45-67-89-AB") == expected

    @given(st.binary(min_size=6, max_size=6))
    def test_six_bytes_are_uppercase(self, value: bytes):
        body = ":".join(f"{byte:02X}" for byte in value)
        assert MacAddressSerializer().serialize(_MAC, body) == f"E'{body}'::macaddr"

    @given(st.binary(min_size=6, max_size=6))
    def test_unsupported(self, value: bytes):
        body = ":".join(f"{byte:02X}" for byte in value)
        with pytest.raises(DialectError, match="MACADDR literal"):
            MacAddressSerializer().serialize(_DIALECT, body)

    @given(st.text(alphabet="ghijklmnopqrstuvwxyz", min_size=1))
    def test_invalid_address(self, value: str):
        with pytest.raises(SerializeError):
            MacAddressSerializer().serialize(_MAC, value)


class TestUnspecifiedSerializer:
    @given(st.one_of(st.integers(), st.text(), st.none()))
    def test_uses_str(self, value: int | str | None):
        assert UnspecifiedSerializer().serialize(_DIALECT, value) == str(value)


class TestArraySerializer:
    @pytest.mark.parametrize(
        ("template", "expected", "empty", "nested"),
        [
            ("ARRAY[{}]", "ARRAY[1, 2, 3]", "ARRAY[]", "ARRAY[ARRAY[1, 2], ARRAY[3]]"),
            ("[{}]", "[1, 2, 3]", "[]", "[[1, 2], [3]]"),
            ("array({})", "array(1, 2, 3)", "array()", "array(array(1, 2), array(3))"),
        ],
    )
    def test_renders_constructor(self, template: str, expected: str, empty: str, nested: str):
        dialect = _literal(array=template)
        serializer = ArraySerializer(IntegerSerializer())
        assert serializer.serialize(dialect, [1, 2, 3]) == expected
        assert serializer.serialize(dialect, (1, 2, 3)) == expected
        assert serializer.serialize(dialect, []) == empty
        nested_serializer = ArraySerializer(ArraySerializer(IntegerSerializer()))
        assert nested_serializer.serialize(dialect, [[1, 2], [3]]) == nested

    def test_unsupported(self):
        with pytest.raises(DialectError, match="ARRAY literal"):
            ArraySerializer(IntegerSerializer()).serialize(Dialect(), [1, 2])

    @given(st.lists(st.integers(min_value=-(10**20), max_value=10**20), max_size=20))
    def test_lists_of_integers(self, value: list[int]):
        body = ", ".join(str(item) for item in value)
        assert ArraySerializer(IntegerSerializer()).serialize(_ARRAY, value) == f"ARRAY[{body}]"

    @given(st.text())
    def test_rejects_non_sequence(self, value: str):
        with pytest.raises(SerializeError):
            ArraySerializer(IntegerSerializer()).serialize(_ARRAY, value)

    @given(st.lists(st.text(), min_size=1, max_size=5))
    def test_rejects_bad_element(self, value: list[str]):
        with pytest.raises(SerializeError):
            ArraySerializer(IntegerSerializer()).serialize(_ARRAY, value)

    def test_nested_unsupported_element(self):
        dialect = _literal(array="ARRAY[{}]")
        with pytest.raises(DialectError, match="bit string literal"):
            ArraySerializer(BitSerializer()).serialize(dialect, ["10"])


class TestCustomSerializer:
    def test_receives_dialect_instance(self):
        seen: list[Dialect] = []

        def render(dialect: Dialect, obj: typing.Any) -> str:
            seen.append(dialect)
            return f"CUSTOM:{obj}"

        dialect = Dialect()
        assert CustomSerializer(render).serialize(dialect, 1) == "CUSTOM:1"
        assert seen == [dialect]

    @given(st.integers())
    def test_returns_rendered_text(self, value: int):
        serializer = CustomSerializer(lambda _dialect, obj: f"N:{obj}")
        assert serializer.serialize(_DIALECT, value) == f"N:{value}"

    @given(st.integers())
    def test_must_return_str(self, value: int):
        serializer = CustomSerializer(lambda _dialect, _obj: value)
        with pytest.raises(SerializeError):
            serializer.serialize(_DIALECT, "x")
