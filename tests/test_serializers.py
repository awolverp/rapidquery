import datetime
import ipaddress
import uuid
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
from rapidquery.dialect import PostgreSQL, SqlDialect

# ---------------------------------------------------------------------------
# IntegerSerializer
# ---------------------------------------------------------------------------


class TestIntegerSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(IntegerSerializer, Serializer)

    @pytest.mark.parametrize("value", [0, 1, -1, 42, -42, 2**63, -(2**63), 10**30])
    def test_serializes_ints_as_bare_decimal_literal(
        self, value, dialect_cls: type[SqlDialect]
    ):
        result = IntegerSerializer().serialize(dialect_cls, value)
        assert result == str(value)

    def test_accepts_dialect_instance_as_well_as_class(self):
        cls_result = IntegerSerializer().serialize(PostgreSQL, 5)
        instance_result = IntegerSerializer().serialize(PostgreSQL(), 5)
        assert cls_result == instance_result == "5"

    def test_arbitrary_precision_is_preserved(self, dialect_cls: type[SqlDialect]):
        big = 10**40
        assert IntegerSerializer().serialize(dialect_cls, big) == str(big)

    def test_rejects_bool_despite_bool_being_a_python_int_subclass(
        self, dialect_cls: type[SqlDialect]
    ):
        """bool is technically an int in Python, but must not be accepted here."""
        with pytest.raises(TypeError):
            IntegerSerializer().serialize(dialect_cls, True)

    @pytest.mark.parametrize("value", [1.5, "5", None, [1], Decimal("5")])
    def test_rejects_non_int_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            IntegerSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# FloatSerializer
# ---------------------------------------------------------------------------


class TestFloatSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(FloatSerializer, Serializer)

    def test_serializes_float_as_numeric_literal(self, dialect_cls: type[SqlDialect]):
        result = FloatSerializer().serialize(dialect_cls, 3.14)
        assert result == "3.14"
        # Must round-trip back to the same float (no precision loss).
        assert float(result) == 3.14

    def test_serializes_int_via_its_exact_text_form(
        self, dialect_cls: type[SqlDialect]
    ):
        assert FloatSerializer().serialize(dialect_cls, 42) == "42"

    def test_serializes_decimal_via_its_exact_text_form(
        self, dialect_cls: type[SqlDialect]
    ):
        result = FloatSerializer().serialize(dialect_cls, Decimal("1.23456789012345"))
        assert result == "1.23456789012345"

    @pytest.mark.parametrize("bad", ["abc", "", "  ", "1.2.3-oops", "3.14"])
    def test_rejects_non_numeric_strings(self, bad, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            FloatSerializer().serialize(dialect_cls, bad)

    @pytest.mark.parametrize("value", [float("nan"), float("inf"), float("-inf")])
    def test_rejects_non_finite_floats(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            FloatSerializer().serialize(dialect_cls, value)

    def test_float_round_trips_without_ambiguous_int_form(
        self, dialect_cls: type[SqlDialect]
    ):
        """1.0 must serialize distinguishably from the integer 1 (per doc comment)."""
        result = FloatSerializer().serialize(dialect_cls, 1.0)
        assert result != "1"

    @pytest.mark.parametrize("value", [None, [1.0], b"1.0"])
    def test_rejects_unsupported_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            FloatSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# BooleanSerializer
# ---------------------------------------------------------------------------


class TestBooleanSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(BooleanSerializer, Serializer)

    def test_serializes_true(self, dialect_cls: type[SqlDialect]):
        result = BooleanSerializer().serialize(dialect_cls, True)
        assert isinstance(result, str)
        assert result != ""

    def test_serializes_false(self, dialect_cls: type[SqlDialect]):
        result = BooleanSerializer().serialize(dialect_cls, False)
        assert isinstance(result, str)
        assert result != ""

    def test_true_and_false_serialize_differently(self, dialect_cls: type[SqlDialect]):
        s = BooleanSerializer()
        assert s.serialize(dialect_cls, True) != s.serialize(dialect_cls, False)

    @pytest.mark.parametrize("value", [1, 0, "true", "false", None, 1.0])
    def test_rejects_non_bool_types(self, value, dialect_cls: type[SqlDialect]):
        """Only actual Python bool is accepted, not truthy/falsy look-alikes."""
        with pytest.raises(TypeError):
            BooleanSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# StringSerializer
# ---------------------------------------------------------------------------


class TestStringSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(StringSerializer, Serializer)

    def test_serializes_str_as_quoted_literal(self, dialect_cls: type[SqlDialect]):
        result = StringSerializer().serialize(dialect_cls, "hello")
        assert isinstance(result, str)
        assert "hello" in result
        # A string literal must be visually distinguishable/quoted, not bare.
        assert result != "hello"

    def test_empty_string_still_quoted(self, dialect_cls: type[SqlDialect]):
        result = StringSerializer().serialize(dialect_cls, "")
        assert result != ""

    def test_single_quote_is_escaped(self, dialect_cls: type[SqlDialect]):
        """A literal single quote inside the value must not break out of the
        surrounding quotes (classic SQL-injection-shaped correctness bug)."""
        result = StringSerializer().serialize(dialect_cls, "O'Brien")
        # The escaped form must contain a doubled/escaped quote, not a bare
        # unescaped one that would prematurely close the string literal.
        assert "O'Brien" not in result or "O''Brien" in result or "O\\'Brien" in result

    def test_serializes_str_enum_member_via_its_value(
        self, dialect_cls: type[SqlDialect]
    ):
        import enum

        class Color(str, enum.Enum):
            RED = "red"

        result = StringSerializer().serialize(dialect_cls, Color.RED)
        assert "red" in result

    @pytest.mark.parametrize("value", [123, None, [1, 2], b"bytes"])
    def test_rejects_non_str_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            StringSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# UuidSerializer
# ---------------------------------------------------------------------------


class TestUuidSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(UuidSerializer, Serializer)

    def test_serializes_uuid_object(self, dialect_cls: type[SqlDialect]):
        value = uuid.uuid4()
        result = UuidSerializer().serialize(dialect_cls, value)
        assert str(value) in result

    def test_serializes_uuid_string(self, dialect_cls: type[SqlDialect]):
        value = str(uuid.uuid4())
        result = UuidSerializer().serialize(dialect_cls, value)
        assert value in result

    def test_result_is_quoted(self, dialect_cls: type[SqlDialect]):
        value = uuid.uuid4()
        result = UuidSerializer().serialize(dialect_cls, value)
        assert result != str(value)

    @pytest.mark.parametrize("value", [123, None, [1, 2], b"bytes"])
    def test_rejects_unsupported_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            UuidSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# BytesSerializer
# ---------------------------------------------------------------------------


class TestBytesSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(BytesSerializer, Serializer)

    def test_serializes_bytes(self, dialect_cls: type[SqlDialect]):
        result = BytesSerializer().serialize(dialect_cls, b"\x00\x01\xff")
        assert isinstance(result, str)

    def test_empty_bytes(self, dialect_cls: type[SqlDialect]):
        result = BytesSerializer().serialize(dialect_cls, b"")
        assert isinstance(result, str)

    def test_different_bytes_serialize_differently(self, dialect_cls: type[SqlDialect]):
        s = BytesSerializer()
        assert s.serialize(dialect_cls, b"abc") != s.serialize(dialect_cls, b"xyz")

    @pytest.mark.parametrize("value", ["not bytes", 123, None, bytearray(b"abc")])
    def test_rejects_non_exact_bytes_types(self, value, dialect_cls: type[SqlDialect]):
        """Only `bytes`, not `str`/`bytearray`/other buffer-like types."""
        with pytest.raises(TypeError):
            BytesSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# BitSerializer
# ---------------------------------------------------------------------------


class TestBitSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(BitSerializer, Serializer)

    @pytest.mark.parametrize("value", ["0", "1", "1010", "000", "111111"])
    def test_serializes_valid_bit_strings(self, value, dialect_cls: type[SqlDialect]):
        result = BitSerializer().serialize(dialect_cls, value)
        assert isinstance(result, str)

        as_int = int(value, base=2)

        assert value in result or str(as_int) in result

    def test_rejects_empty_string(self, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            BitSerializer().serialize(dialect_cls, "")

    @pytest.mark.parametrize("bad", ["102", "abc", "1 0", "2", "-1"])
    def test_rejects_non_binary_content(self, bad, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            BitSerializer().serialize(dialect_cls, bad)

    @pytest.mark.parametrize("value", [10, None, [1, 0]])
    def test_rejects_non_str_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            BitSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# JSONSerializer
# ---------------------------------------------------------------------------


class TestJSONSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(JSONSerializer, Serializer)

    @pytest.mark.parametrize(
        "value",
        [
            {"a": 1, "b": [1, 2, 3]},
            [1, 2, 3],
            "plain string",
            123,
            123.45,
            True,
            None,
        ],
    )
    def test_serializes_json_encodable_values(
        self, value, dialect_cls: type[SqlDialect]
    ):
        result = JSONSerializer().serialize(dialect_cls, value)
        assert isinstance(result, str)
        assert result != ""

    def test_result_is_quoted_as_a_string_literal(self, dialect_cls: type[SqlDialect]):
        result = JSONSerializer().serialize(dialect_cls, {"a": 1})
        # The JSON text itself must be wrapped as a SQL string literal.
        assert result != '{"a": 1}'

    def test_dict_contents_are_present_in_output(self, dialect_cls: type[SqlDialect]):
        result = JSONSerializer().serialize(dialect_cls, {"key": "value"})
        assert "key" in result and "value" in result

    def test_rejects_non_json_encodable_values(self, dialect_cls: type[SqlDialect]):
        class NotEncodable:
            pass

        with pytest.raises(TypeError):
            JSONSerializer().serialize(dialect_cls, NotEncodable())


# ---------------------------------------------------------------------------
# DateSerializer
# ---------------------------------------------------------------------------


class TestDateSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(DateSerializer, Serializer)

    def test_serializes_date_as_iso_literal(self, dialect_cls: type[SqlDialect]):
        result = DateSerializer().serialize(dialect_cls, datetime.date(2024, 1, 15))
        assert "2024-01-15" in result

    def test_result_is_quoted(self, dialect_cls: type[SqlDialect]):
        result = DateSerializer().serialize(dialect_cls, datetime.date(2024, 1, 15))
        assert result != "2024-01-15"

    def test_rejects_datetime_as_not_exactly_date(self, dialect_cls: type[SqlDialect]):
        """A `datetime.datetime` is technically a `date` subclass in Python,
        but the serializer should require an exact `date` (see e.g. the
        exact-type checks used throughout the other serializers)."""
        with pytest.raises(TypeError):
            DateSerializer().serialize(
                dialect_cls, datetime.datetime(2024, 1, 15, 10, 0)
            )

    @pytest.mark.parametrize("value", ["2024-01-15", None, 20240115])
    def test_rejects_non_date_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            DateSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# TimeSerializer
# ---------------------------------------------------------------------------


class TestTimeSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(TimeSerializer, Serializer)

    def test_serializes_time_as_hms_literal(self, dialect_cls: type[SqlDialect]):
        result = TimeSerializer().serialize(dialect_cls, datetime.time(13, 45, 30))
        assert "13:45:30" in result

    def test_serializes_time_with_microseconds(self, dialect_cls: type[SqlDialect]):
        result = TimeSerializer().serialize(dialect_cls, datetime.time(1, 2, 3, 456789))
        assert "01:02:03" in result

    def test_result_is_quoted(self, dialect_cls: type[SqlDialect]):
        result = TimeSerializer().serialize(dialect_cls, datetime.time(13, 45, 30))
        assert result != "13:45:30"

    @pytest.mark.parametrize("value", ["13:45:30", None, datetime.date(2024, 1, 1)])
    def test_rejects_non_time_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            TimeSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# DatetimeSerializer
# ---------------------------------------------------------------------------


class TestDatetimeSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(DatetimeSerializer, Serializer)

    def test_serializes_naive_datetime(self, dialect_cls: type[SqlDialect]):
        result = DatetimeSerializer().serialize(
            dialect_cls, datetime.datetime(2024, 1, 15, 10, 30, 0)
        )
        assert "2024-01-15" in result and "10:30:00" in result

    def test_serializes_aware_datetime(self, dialect_cls: type[SqlDialect]):
        value = datetime.datetime(2024, 1, 15, 10, 30, 0, tzinfo=datetime.timezone.utc)
        result = DatetimeSerializer().serialize(dialect_cls, value)
        assert "2024-01-15" in result and "10:30:00" in result

    def test_accepts_int_as_utc_epoch(self, dialect_cls: type[SqlDialect]):
        result = DatetimeSerializer().serialize(dialect_cls, 0)
        assert "1970-01-01" in result

    def test_accepts_float_as_utc_epoch(self, dialect_cls: type[SqlDialect]):
        result = DatetimeSerializer().serialize(dialect_cls, 0.0)
        assert "1970-01-01" in result

    def test_result_is_quoted(self, dialect_cls: type[SqlDialect]):
        result = DatetimeSerializer().serialize(
            dialect_cls, datetime.datetime(2024, 1, 15, 10, 30, 0)
        )
        assert result != "2024-01-15 10:30:00"

    @pytest.mark.parametrize("value", ["2024-01-15T10:30:00", None, [2024, 1, 15]])
    def test_rejects_unsupported_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(TypeError):
            DatetimeSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# InetSerializer
# ---------------------------------------------------------------------------


class TestInetSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(InetSerializer, Serializer)

    @pytest.mark.parametrize(
        "value",
        [
            ipaddress.ip_address("192.168.1.1"),
            "192.168.1.1",
            "192.168.1.0/24",
            "::1",
            "2001:db8::/32",
        ],
    )
    def test_serializes_valid_ip_networks(self, value, dialect_cls: type[SqlDialect]):
        result = InetSerializer().serialize(dialect_cls, value)
        assert isinstance(result, str)
        assert result != ""

    def test_result_is_quoted(self, dialect_cls: type[SqlDialect]):
        result = InetSerializer().serialize(dialect_cls, "192.168.1.1")
        assert result != "192.168.1.1"

    @pytest.mark.parametrize("bad", ["not-an-ip", "999.999.999.999", ""])
    def test_rejects_invalid_ip_strings(self, bad, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            InetSerializer().serialize(dialect_cls, bad)

    @pytest.mark.parametrize("value", [None, 12345])
    def test_rejects_non_str_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            InetSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# MacAddressSerializer
# ---------------------------------------------------------------------------


class TestMacAddressSerializer:
    def test_is_serializer_subclass(self):
        assert issubclass(MacAddressSerializer, Serializer)

    @pytest.mark.parametrize("value", ["01:23:45:67:89:ab", "01-23-45-67-89-AB"])
    def test_serializes_valid_mac_addresses(self, value, dialect_cls: type[SqlDialect]):
        result = MacAddressSerializer().serialize(dialect_cls, value)
        assert isinstance(result, str)
        assert result != ""

    def test_result_is_quoted(self, dialect_cls: type[SqlDialect]):
        result = MacAddressSerializer().serialize(dialect_cls, "01:23:45:67:89:ab")
        assert result != "01:23:45:67:89:ab"

    @pytest.mark.parametrize("bad", ["not-a-mac", "gg:hh:ii:jj:kk:ll", ""])
    def test_rejects_invalid_mac_strings(self, bad, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            MacAddressSerializer().serialize(dialect_cls, bad)

    @pytest.mark.parametrize("value", [None, 12345])
    def test_rejects_non_str_types(self, value, dialect_cls: type[SqlDialect]):
        with pytest.raises(ValueError):
            MacAddressSerializer().serialize(dialect_cls, value)


# ---------------------------------------------------------------------------
# UnspecifiedSerializer
# ---------------------------------------------------------------------------


class TestUnspecifiedSerializer:
    """The fallback serializer: `str(obj)`, no type validation, no quoting.

    This is intentionally permissive per its doc comment ("useful when user
    doesn't specify the serializer, and also we couldn't detect the type
    from datatype").
    """

    def test_is_serializer_subclass(self):
        assert issubclass(UnspecifiedSerializer, Serializer)

    @pytest.mark.parametrize(
        "value,expected",
        [
            (123, "123"),
            ("hello", "hello"),
            (3.14, "3.14"),
            (None, "None"),
            (True, "True"),
        ],
    )
    def test_falls_back_to_str_of_the_object(
        self, value, expected, dialect_cls: type[SqlDialect]
    ):
        result = UnspecifiedSerializer().serialize(dialect_cls, value)
        assert result == expected

    def test_accepts_any_object_without_raising(self, dialect_cls: type[SqlDialect]):
        class Anything:
            def __str__(self):
                return "custom-repr"

        result = UnspecifiedSerializer().serialize(dialect_cls, Anything())
        assert result == "custom-repr"


# ---------------------------------------------------------------------------
# Cross-cutting: dialect argument acceptance
# ---------------------------------------------------------------------------


class TestSerializeAcceptsClassOrInstance:
    """Per data_type.pyi: `serialize(dialect: Type[SqlDialect] | SqlDialect, ...)`."""

    def test_integer_serializer_accepts_class(self):
        assert IntegerSerializer().serialize(PostgreSQL, 1) == "1"

    def test_integer_serializer_accepts_instance(self):
        assert IntegerSerializer().serialize(PostgreSQL(), 1) == "1"

    def test_class_and_instance_give_identical_results(
        self, dialect_cls: type[SqlDialect]
    ):
        s = StringSerializer()
        assert s.serialize(dialect_cls, "x") == s.serialize(dialect_cls(), "x")

    def test_rejects_non_dialect_argument(self):
        with pytest.raises(TypeError):
            IntegerSerializer().serialize(
                "not-a-dialect",  # type: ignore
                1,
            )

    def test_rejects_none_as_dialect(self):
        with pytest.raises(TypeError):
            IntegerSerializer().serialize(
                None,  # type: ignore
                1,
            )


class TestSerializerMarkers:
    def test_unit_instances_compare_equal(self):
        assert IntegerSerializer() == IntegerSerializer()
        assert IntegerSerializer() != FloatSerializer()

    def test_unit_instances_are_hashable(self):
        assert hash(IntegerSerializer()) == hash(IntegerSerializer())
        assert len({IntegerSerializer(), IntegerSerializer(), FloatSerializer()}) == 2

    def test_unit_repr_is_constructor_call(self):
        assert repr(IntegerSerializer()) == "IntegerSerializer()"
        assert repr(JSONSerializer()) == "JSONSerializer()"

    def test_array_equals_by_element_tree(self):
        a = ArraySerializer(IntegerSerializer())
        b = ArraySerializer(IntegerSerializer())
        c = ArraySerializer(StringSerializer())
        assert a == b
        assert a != c
        assert repr(a) == "ArraySerializer(IntegerSerializer())"

    def test_custom_equals_by_function_identity(self):
        def fn(dialect, obj):
            return "x"

        assert CustomSerializer(fn) == CustomSerializer(fn)
        assert CustomSerializer(fn) != CustomSerializer(lambda d, o: "x")


class TestDatetimeOffsetPreserved:
    def test_aware_datetime_keeps_offset(self, dialect_cls: type[SqlDialect]):
        value = datetime.datetime(
            2024,
            1,
            15,
            10,
            30,
            0,
            tzinfo=datetime.timezone(datetime.timedelta(hours=5)),
        )
        result = DatetimeSerializer().serialize(dialect_cls, value)
        assert "10:30:00" in result
        assert "+05:00" in result
        assert "05:30:00" not in result
