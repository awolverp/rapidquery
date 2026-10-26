"""Serializer type checks and literal spelling for a configured dialect."""

import datetime
import enum
import ipaddress
import typing
import uuid
from dataclasses import dataclass
from decimal import Decimal

import pytest

from rapidquery.dialect import OPT_BOOL_FORMAT_INTEGER, Dialect, LiteralConfig, StringConfig, TemporalConfig
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


class InvalidStrEnum(enum.Enum):
    Variant = 1


@dataclass(frozen=True, slots=True)
class ExampleDataclass:
    x: int


def _literal(**kwargs: typing.Any) -> Dialect:
    return Dialect(literal=LiteralConfig(**kwargs))


def _temporal(**kwargs: str) -> Dialect:
    return Dialect(temporal=TemporalConfig(**kwargs))


class TestIntegerSerializer:
    def test_values(self):
        serializer = IntegerSerializer()
        dialect = Dialect()
        assert serializer.serialize(dialect, 1) == "1"
        assert serializer.serialize(dialect, 18_446_744_073_709_551_615) == "18446744073709551615"
        assert serializer.serialize(dialect, -9_223_372_036_854_775_808) == "-9223372036854775808"
        assert serializer.serialize(dialect, True) == "1"
        assert serializer.serialize(dialect, False) == "0"

    @pytest.mark.parametrize("value", [2.5, "invalid"])
    def test_rejects_non_int(self, value: typing.Any):
        with pytest.raises(TypeError):
            IntegerSerializer().serialize(Dialect(), value)


class TestFloatSerializer:
    def test_values(self):
        serializer = FloatSerializer()
        dialect = Dialect()
        assert serializer.serialize(dialect, 1.1) == "1.1"
        assert serializer.serialize(dialect, 1) == "1"
        assert serializer.serialize(dialect, 1.0) == "1.0"
        assert serializer.serialize(dialect, Decimal("1.3435")) == "1.3435"

    def test_rejects_non_numeric(self):
        with pytest.raises(TypeError):
            FloatSerializer().serialize(Dialect(), "invalid")

    @pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
    def test_rejects_non_finite(self, value: float):
        with pytest.raises(ValueError):
            FloatSerializer().serialize(Dialect(), value)


class TestBooleanSerializer:
    def test_keyword(self):
        serializer = BooleanSerializer()
        dialect = Dialect()
        assert serializer.serialize(dialect, True) == "TRUE"
        assert serializer.serialize(dialect, False) == "FALSE"

    def test_integer(self):
        serializer = BooleanSerializer()
        dialect = _literal(boolean=OPT_BOOL_FORMAT_INTEGER)
        assert serializer.serialize(dialect, True) == "1"
        assert serializer.serialize(dialect, False) == "0"

    @pytest.mark.parametrize("value", [1, 0, object()])
    def test_rejects_non_bool(self, value: typing.Any):
        with pytest.raises(TypeError):
            BooleanSerializer().serialize(Dialect(), value)


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

    @pytest.mark.parametrize("template", ["'{single}'", "E'{hex}'"])
    def test_rejects_nul(self, template: str):
        dialect = Dialect(string=StringConfig(template=template))
        with pytest.raises(ValueError, match="NUL"):
            StringSerializer().serialize(dialect, "a\0b")

    @pytest.mark.parametrize("value", [InvalidStrEnum.Variant, 1, None, b"bytes"])
    def test_rejects_non_str(self, value: typing.Any):
        with pytest.raises(TypeError):
            StringSerializer().serialize(Dialect(), value)


class TestUuidSerializer:
    @pytest.mark.parametrize("template", ["'{}'", "E'{}'::uuid", "UUID '{}'", "N'{}'"])
    def test_str_and_uuid(self, template: str):
        dialect = _literal(uuid=template)
        expected = template.replace("{}", _UUID)
        serializer = UuidSerializer()
        assert serializer.serialize(dialect, _UUID) == expected
        assert serializer.serialize(dialect, uuid.UUID(_UUID)) == expected

    def test_unsupported(self):
        with pytest.raises(RuntimeError, match="does not support UuidSerializer"):
            UuidSerializer().serialize(Dialect(), _UUID)

    def test_invalid_str(self):
        dialect = _literal(uuid="'{}'")
        with pytest.raises(ValueError):
            UuidSerializer().serialize(dialect, "103596-db75-4ae0-907a-a4a9bc4d85")

    def test_rejects_non_uuid(self):
        dialect = _literal(uuid="'{}'")
        with pytest.raises(TypeError):
            UuidSerializer().serialize(dialect, 6428739)


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


@pytest.mark.parametrize("value", [bytearray(b"abc"), memoryview(b"abc"), "abc", 1, None])
def test_bytes_rejects_non_exact_bytes(value: typing.Any):
    with pytest.raises(TypeError):
        BytesSerializer().serialize(Dialect(), value)


class TestBitSerializer:
    @pytest.mark.parametrize(
        ("template", "expected"),
        [("B'{}'", "B'10'"), ("b'{}'", "b'10'"), ("'{}'B", "'10'B")],
    )
    def test_embeds_bits(self, template: str, expected: str):
        assert BitSerializer().serialize(_literal(bit=template), "10") == expected

    def test_unsupported(self):
        with pytest.raises(RuntimeError, match="does not support BitSerializer"):
            BitSerializer().serialize(Dialect(), "10")

    def test_rejects_non_bit_string(self):
        dialect = _literal(bit="B'{}'")
        with pytest.raises(ValueError):
            BitSerializer().serialize(dialect, "102")
        with pytest.raises(ValueError):
            BitSerializer().serialize(dialect, "")
        with pytest.raises(TypeError):
            BitSerializer().serialize(dialect, 10)


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

    def test_date_rejects_datetime_and_str(self):
        with pytest.raises(TypeError):
            DateSerializer().serialize(Dialect(), datetime.datetime(2024, 1, 15, 10, 0))
        with pytest.raises(TypeError):
            DateSerializer().serialize(Dialect(), "2024-01-15")

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

    def test_time_rejects_date(self):
        with pytest.raises(TypeError):
            TimeSerializer().serialize(Dialect(), _DATE)

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

    def test_datetime_rejects_date_and_str(self):
        with pytest.raises(TypeError):
            DatetimeSerializer().serialize(Dialect(), _DATE)
        with pytest.raises(TypeError):
            DatetimeSerializer().serialize(Dialect(), "2024-01-15T10:30:00")

    def test_datetime_overflow(self):
        with pytest.raises(OverflowError):
            DatetimeSerializer().serialize(Dialect(), 10**20)


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

    def test_unsupported(self):
        with pytest.raises(RuntimeError, match="does not support InetSerializer"):
            InetSerializer().serialize(Dialect(), "192.168.1.1")

    def test_invalid_address(self):
        with pytest.raises(ValueError):
            InetSerializer().serialize(_literal(inet="'{}'::inet"), "not-an-ip")


class TestMacAddressSerializer:
    def test_canonical_form(self):
        dialect = _literal(macaddr="E'{}'::macaddr")
        serializer = MacAddressSerializer()
        expected = "E'01:23:45:67:89:AB'::macaddr"
        assert serializer.serialize(dialect, "01:23:45:67:89:ab") == expected
        assert serializer.serialize(dialect, "01-23-45-67-89-AB") == expected

    def test_unsupported(self):
        with pytest.raises(RuntimeError, match="does not support MacAddressSerializer"):
            MacAddressSerializer().serialize(Dialect(), "01:23:45:67:89:ab")

    def test_invalid_address(self):
        with pytest.raises(ValueError):
            MacAddressSerializer().serialize(_literal(macaddr="E'{}'::macaddr"), "not-a-mac")


class TestUnspecifiedSerializer:
    def test_uses_str(self):
        serializer = UnspecifiedSerializer()
        dialect = Dialect()
        assert serializer.serialize(dialect, 123) == "123"
        assert serializer.serialize(dialect, "hello") == "hello"
        assert serializer.serialize(dialect, None) == "None"


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
        with pytest.raises(RuntimeError, match="does not support ArraySerializer"):
            ArraySerializer(IntegerSerializer()).serialize(Dialect(), [1, 2])

    def test_rejects_non_sequence_and_bad_element(self):
        dialect = _literal(array="ARRAY[{}]")
        serializer = ArraySerializer(IntegerSerializer())
        with pytest.raises(TypeError):
            serializer.serialize(dialect, "not a list")
        with pytest.raises(TypeError):
            serializer.serialize(dialect, [1, "x"])

    def test_nested_unsupported_element(self):
        dialect = _literal(array="ARRAY[{}]")
        with pytest.raises(RuntimeError, match="does not support BitSerializer"):
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

    def test_must_return_str(self):
        def render(dialect: Dialect, obj: typing.Any) -> int:
            return 12345

        with pytest.raises(TypeError):
            CustomSerializer(render).serialize(Dialect(), "x")
