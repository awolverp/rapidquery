import datetime
import decimal
import enum
import typing
import uuid
from collections.abc import Callable

from rapidquery.dialect import Dialect

_T = typing.TypeVar("_T")

class Serializer(typing.Generic[_T]):
    """
    Turns one Python object into a SQL literal for a dialect.

    Use a subclass when `Value` should not infer the spelling, or call
    `serialize` to render a literal without building an expression. Subclasses
    are frozen singletons except `ArraySerializer` and `CustomSerializer`.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import StringSerializer
        >>> StringSerializer().serialize(PostgreSql, "hi")
        "E'hi'"
    """

    def serialize(self, dialect: Dialect, object: _T) -> str:
        """
        Render `object` as a SQL literal.

        Args:
            dialect: Dialect that chooses quotes, keywords, and hex form.
            object: Value this serializer accepts.

        Returns:
            The SQL literal, with no trailing separator.

        Raises:
            TypeError: `object` is not a type this serializer accepts.
            ValueError: `object` is the right type but not a legal value.
            NotImplementedError: The dialect has no literal of this kind.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@typing.final
class IntegerSerializer(Serializer[int]):
    """
    Bare SQL integer for a Python `int`.

    Use this for integers of any width, including values outside 64-bit.
    `bool` is rejected so `True` is not written as `1` by this serializer.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import IntegerSerializer
        >>> IntegerSerializer().serialize(Sqlite, 7)
        '7'
    """

    def __new__(cls) -> typing.Self:
        """Return the integer serializer."""

@typing.final
class FloatSerializer(Serializer[int | float | decimal.Decimal]):
    """
    Bare SQL number for `float`, `int`, or `decimal.Decimal`.

    Use this when a whole number must stay a decimal (`1.0`, not `1`).
    Non-finite floats are rejected. Strings are not accepted.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import FloatSerializer
        >>> FloatSerializer().serialize(Sqlite, 1.0)
        '1.0'
    """

    def __new__(cls) -> typing.Self:
        """Return the float serializer."""

@typing.final
class BooleanSerializer(Serializer[bool]):
    """
    Dialect boolean literal for a Python `bool`.

    Use this for `TRUE` / `FALSE` or `1` / `0`, whichever the dialect writes.
    Only `bool` is accepted.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import BooleanSerializer
        >>> BooleanSerializer().serialize(PostgreSql, True)
        'TRUE'
    """

    def __new__(cls) -> typing.Self:
        """Return the boolean serializer."""

@typing.final
class StringSerializer(Serializer[str | enum.Enum]):
    """
    Quoted, escaped SQL string.

    Use this for text. An `enum.Enum` member is accepted when its value is a
    `str`. The dialect chooses the quotes and the escape rules.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import StringSerializer
        >>> StringSerializer().serialize(Sqlite, "hi")
        "'hi'"
    """

    def __new__(cls) -> typing.Self:
        """Return the string serializer."""

@typing.final
class UuidSerializer(Serializer[uuid.UUID | str]):
    """
    UUID literal, spelled with the dialect's `uuid_format`.

    Use this for `uuid.UUID` or a UUID string. The result is a quoted literal,
    `UUID '…'`, or `'…'::uuid`.

    Examples:
        >>> import uuid
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import UuidSerializer
        >>> UuidSerializer().serialize(PostgreSql, uuid.UUID(int=0))
        "E'00000000-0000-0000-0000-000000000000'::uuid"
    """

    def __new__(cls) -> typing.Self:
        """Return the UUID serializer."""

@typing.final
class BytesSerializer(Serializer[bytes]):
    """
    Dialect blob or bytea literal for exact `bytes`.

    Use this for binary payloads. `bytearray` and `memoryview` are rejected.
    The dialect chooses `X'…'`, `\\x…::bytea`, or a conversion function.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import BytesSerializer
        >>> BytesSerializer().serialize(PostgreSql, b"A")
        "'\\\\x41'::bytea"
    """

    def __new__(cls) -> typing.Self:
        """Return the bytes serializer."""

@typing.final
class BitSerializer(Serializer[str]):
    """
    Bit-string literal for a string of `0` and `1` characters.

    Use this on PostgreSQL, DuckDB, MySQL, and Teradata. Other dialects raise.
    The string must be non-empty.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import BitSerializer
        >>> BitSerializer().serialize(PostgreSql, "101")
        "B'101'"
    """

    def __new__(cls) -> typing.Self:
        """Return the bit-string serializer."""

@typing.final
class JSONSerializer(Serializer[typing.Any]):
    """
    JSON literal, spelled with the dialect's `json_format`.

    Use this for objects the JSON encoder can dump. `orjson` is used when it
    is installed; otherwise the stdlib `json` module is used. The JSON text
    is then wrapped as a quoted literal, `JSON '…'`, or `PARSE_JSON('…')`.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import JSONSerializer
        >>> JSONSerializer().serialize(Sqlite, {"a": 1})
        '\\'{"a":1}\\''
    """

    def __new__(cls) -> typing.Self:
        """Return the JSON serializer."""

@typing.final
class DateSerializer(Serializer[datetime.date]):
    """
    Date literal `YYYY-MM-DD`, quoted or after the `DATE` keyword.

    Use this for `datetime.date`. A `datetime.datetime` is rejected so a
    timestamp is not silently truncated.

    Examples:
        >>> import datetime
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import DateSerializer
        >>> DateSerializer().serialize(Sqlite, datetime.date(2020, 1, 2))
        "'2020-01-02'"
    """

    def __new__(cls) -> typing.Self:
        """Return the date serializer."""

@typing.final
class TimeSerializer(Serializer[datetime.time]):
    """
    Time literal `HH:MM:SS[.ffffff]`, quoted or after the `TIME` keyword.

    Use this for `datetime.time`.

    Examples:
        >>> import datetime
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import TimeSerializer
        >>> TimeSerializer().serialize(Sqlite, datetime.time(1, 2, 3))
        "'01:02:03'"
    """

    def __new__(cls) -> typing.Self:
        """Return the time serializer."""

@typing.final
class DatetimeSerializer(Serializer[datetime.datetime | int | float]):
    """
    Timestamp literal for a `datetime` or a UTC epoch.

    Use this for naive or aware datetimes, or for `int` / `float` seconds
    since the Unix epoch. Naive values keep their wall time. Aware values
    keep their offset. Epoch numbers are treated as UTC. The dialect chooses
    a quoted literal or a `TIMESTAMP` / `TIMESTAMPTZ` keyword.

    Examples:
        >>> import datetime
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import DatetimeSerializer
        >>> value = datetime.datetime(2020, 1, 2, 3, 4, 5)
        >>> DatetimeSerializer().serialize(Sqlite, value)
        "'2020-01-02 03:04:05'"
    """

    def __new__(cls) -> typing.Self:
        """Return the timestamp serializer."""

@typing.final
class InetSerializer(Serializer[typing.Any]):
    """
    IP address or network literal for PostgreSQL and DuckDB.

    Use this for `ipaddress` objects or for a string that is an address or
    CIDR. Other dialects raise. The result is a quoted literal or `'…'::inet`.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import InetSerializer
        >>> InetSerializer().serialize(PostgreSql, "127.0.0.1")
        "E'127.0.0.1/32'::inet"
    """

    def __new__(cls) -> typing.Self:
        """Return the inet serializer."""

@typing.final
class MacAddressSerializer(Serializer[str]):
    """
    MAC address literal.

    Use this for a `str` that uses `:` or `-` as the separator. PostgreSQL
    writes a native MAC literal. Other dialects raise.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import MacAddressSerializer
        >>> MacAddressSerializer().serialize(PostgreSql, "00:11:22:33:44:55")
        "E'00:11:22:33:44:55'::macaddr"
    """

    def __new__(cls) -> typing.Self:
        """Return the MAC address serializer."""

@typing.final
class UnspecifiedSerializer(Serializer[typing.Any]):
    """
    Inserts `str(object)` with no quotes and no type check.

    Use this only when no builtin serializer matches and you already hold SQL
    text. The result is not escaped. Prefer an explicit serializer for values
    that come from outside the program.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import UnspecifiedSerializer
        >>> UnspecifiedSerializer().serialize(Sqlite, 1)
        '1'
    """

    def __new__(cls) -> typing.Self:
        """Return the unspecified serializer."""

@typing.final
class ArraySerializer(Serializer[list[_T] | tuple[_T]]):
    """
    `ARRAY[elem, …]` built by serializing each item.

    Use this for a list or tuple on PostgreSQL, DuckDB, and BigQuery. Pass
    another `ArraySerializer` as `element` for a nested array. Other dialects
    raise.

    Examples:
        >>> from rapidquery.dialect import PostgreSql
        >>> from rapidquery.serializers import ArraySerializer, IntegerSerializer
        >>> ArraySerializer(IntegerSerializer()).serialize(PostgreSql, [1, 2])
        'ARRAY[1, 2]'
    """

    def __new__(cls, element: Serializer[_T]) -> typing.Self:
        """
        Build an array serializer.

        Args:
            element: Serializer applied to every item.
        """

    @property
    def element(self) -> Serializer[_T]:
        """Serializer applied to every item."""

@typing.final
class CustomSerializer(Serializer[_T]):
    """
    Inserts the text returned by a Python callable.

    Use this for a literal spelling the builtins do not cover. The callable
    receives the dialect name and the object, and must return SQL text. That
    text is inserted as-is, with no extra quoting.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.serializers import CustomSerializer
        >>> serializer = CustomSerializer(lambda _dialect, value: str(value))
        >>> serializer.serialize(Sqlite, 3)
        '3'
    """

    def __new__(cls, function: Callable[[Dialect, _T], str]) -> typing.Self:
        """
        Build a serializer around `function`.

        Args:
            function: `fn(dialect, object) -> str`. `dialect` is the `Dialect`
                value, not its name.

        Raises:
            TypeError: `function` is not callable.
        """

    @property
    def function(self) -> Callable[[Dialect, _T], str]:
        """The callable passed to the constructor."""
