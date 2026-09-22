import datetime
import decimal
import enum
import typing
import uuid
from collections.abc import Callable

from rapidquery.dialect import SqlDialect

_T = typing.TypeVar("_T")

class Serializer(typing.Generic[_T]):
    """
    Converts a Python object into a dialect-specific SQL literal string.

    Serializers produce a correctly escaped and formatted SQL value that can
    be embedded in queries. Each concrete subclass handles a particular
    category of values (integers, strings, dates, arrays, etc.). Use
    `CustomSerializer` when the built-in ones are insufficient.

    This base class cannot be instantiated. Pass a dialect class or instance
    to `serialize()`. Unit serializers compare equal by variant;
    `ArraySerializer` by its element tree; `CustomSerializer` by function
    identity.
    """

    def serialize(self, dialect: SqlDialect, object: _T) -> str:
        """
        Renders `object` as a SQL literal for `dialect`.

        Args:
            dialect: A dialect class (`PostgreSQL`) or instance (`PostgreSQL()`).
            object: The Python value to convert.

        Returns:
            A SQL fragment suitable for embedding in a statement.

        Raises:
            TypeError: If `object` is not an accepted type for this serializer.
            ValueError: If `object` is the right type but cannot be represented
                    (for example a non-finite float).
            NotImplementedError: If `dialect` has no native literal for this
                    serializer (bit strings, `ARRAY[…]`, inet, macaddr).
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@typing.final
class IntegerSerializer(Serializer[int]):
    """
    Serializes Python integers as bare SQL integer literals.

    Accepts Python `int` of any width (including values beyond 64-bit).
    Rejects `bool`; `True` and `False` are not treated as integers.

    Raises:
        TypeError: If the value is not an `int`.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class FloatSerializer(Serializer[int | float | decimal.Decimal]):
    """
    Serializes numeric values as bare SQL numeric literals.

    Accepts `float`, `int`, and `decimal.Decimal`. Finite floats are formatted
    so that `1.0` stays distinct from `1`. `str` is not accepted.

    Raises:
        TypeError: If the value is not a `float`, `int`, or `decimal.Decimal`.
        ValueError: If the value is a non-finite float (`nan`, `inf`, `-inf`).
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class BooleanSerializer(Serializer[bool]):
    """
    Serializes Python booleans as dialect boolean literals.

    Accepts Python `bool` only. The dialect chooses the spelling
    (`TRUE`/`FALSE` or `1`/`0`).

    Raises:
        TypeError: If the value is not a `bool`.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class StringSerializer(Serializer[str | enum.Enum]):
    """
    Serializes text as a quoted, escaped SQL string literal.

    Accepts `str`, or an `enum.StrEnum`. The
    dialect supplies quoting and escaping.

    Raises:
        TypeError: If the value is not a `str` or a string-valued enum member.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class UuidSerializer(Serializer[uuid.UUID | str]):
    """
    Serializes UUIDs as quoted SQL string literals.

    Accepts `uuid.UUID` or `str`.

    Raises:
        TypeError: If the value is not a `uuid.UUID` or `str`.
        ValueError: If the value is `str`, but it is not a UUID.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class BytesSerializer(Serializer[bytes]):
    """
    Serializes binary data as a dialect blob/bytea literal.

    Accepts exact `bytes` (not `bytearray` or `memoryview`).

    Raises:
        TypeError: If the value is not `bytes`.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class BitSerializer(Serializer[str]):
    """
    Serializes bit strings as dialect bit-string literals.

    Accepts a `str` consisting only of `'0'` and `'1'` characters.
    Supported on PostgreSQL, DuckDB, MySQL, and Teradata.

    Raises:
        TypeError: If the value is not a `str`.
        ValueError: If the string is empty or contains characters other than `'0'` and `'1'`.
        NotImplementedError: If the dialect has no native bit-string literal.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class JSONSerializer(Serializer[typing.Any]):
    """
    Serializes a JSON-encodable object as a quoted SQL string literal.

    Accepts any object that the JSON encoder can dump. Uses `orjson` when it
    is installed, otherwise the stdlib `json` module. The JSON text is then
    wrapped with the dialect's string quoting. With `orjson`, datetime, UUID,
    enum.StrEnum, and dataclass are encoded natively; stdlib `json`
    rejects those.

    Raises:
        TypeError: If the value cannot be JSON-encoded.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class DateSerializer(Serializer[datetime.date]):
    """
    Serializes dates as quoted `YYYY-MM-DD` literals.

    Accepts exact `datetime.date` (not `datetime.datetime`).

    Raises:
        TypeError: If the value is not a `datetime.date`.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class TimeSerializer(Serializer[datetime.time]):
    """
    Serializes times as quoted `HH:MM:SS[.ffffff]` literals.

    Accepts `datetime.time`.

    Raises:
        TypeError: If the value is not a `datetime.time`.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class DatetimeSerializer(Serializer[datetime.datetime]):
    """
    Serializes timestamps as quoted SQL timestamp literals.

    Accepts naive or aware `datetime.datetime`, or `int`/`float` UTC epoch
    seconds. Naive values keep wall time. Aware values keep their original
    offset (they are not converted to UTC).

    Raises:
        TypeError: If the value is not a `datetime.datetime`, `int`, or `float`.
        OverflowError: If an epoch value is out of range for a timestamp.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class InetSerializer(Serializer[typing.Any]):
    """
    Serializes IP addresses and networks as quoted SQL literals.

    Accepts `str` or `ipaddress` objects (`IPv4Address`, `IPv6Address`,
    `IPv4Network`, `IPv6Network`, and related types whose `str()` is a valid
    address or CIDR). Supported on PostgreSQL and DuckDB.

    Raises:
        ValueError: If the value is not a valid IPv4/IPv6 address or network.
        NotImplementedError: If the dialect has no native INET type.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class MacAddressSerializer(Serializer[str]):
    """
    Serializes MAC addresses as quoted SQL literals.

    Accepts a `str` using `:` or `-` as the separator.
    Supported on PostgreSQL.

    Raises:
        ValueError: If the value is not a valid MAC address.
        NotImplementedError: If the dialect has no native MACADDR type.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class UnspecifiedSerializer(Serializer[typing.Any]):
    """
    Inserts `str(obj)` into SQL with no quoting or type check.

    Used when no serializer could be inferred from the type name. The result
    is not a safe SQL value; prefer an explicit serializer.
    """

    def __new__(cls) -> typing.Self: ...

@typing.final
class ArraySerializer(Serializer[list[_T] | tuple[_T]]):
    """
    Serializes sequences as `ARRAY[elem, ...]` literals.

    Each element is serialized independently with `element`. Accepts `list`
    or `tuple`. Nested arrays are expressed by wrapping another
    `ArraySerializer`. Supported on PostgreSQL, DuckDB, and BigQuery.

    Raises:
        TypeError: If the value is not a `list` or `tuple`.
        NotImplementedError: If the dialect has no `ARRAY[…]` constructor.
    """

    def __new__(cls, element: Serializer[_T]) -> typing.Self:
        """
        Creates an array serializer.

        Args:
            element: The serializer applied to every array item. Nested arrays
                    are expressed by passing another `ArraySerializer`.
        """

    @property
    def element(self) -> Serializer[_T]:
        """
        Returns the serializer used for each array element.

        Returns:
            The `Serializer` passed to the constructor.
        """

@typing.final
class CustomSerializer(Serializer[_T]):
    """
    Serializes values by calling a user-supplied Python function.

    `function` has the form `fn(dialect_name: str, obj) -> str`. The return
    value is inserted into SQL as-is (not re-quoted or escaped). It is the
    caller's responsibility to produce a safe fragment.
    """

    def __new__(cls, function: Callable[[SqlDialect, _T], str]) -> typing.Self:
        """
        Creates a serializer backed by a Python callable.

        Args:
            function: A callable of the form `fn(dialect_name: str, obj) -> str`.
                    The return value is inserted into SQL as-is (not re-quoted).
                    Must return a `str`.
        """

    @property
    def function(self) -> Callable[[str, _T], str]:
        """
        Returns the callable passed to the constructor.

        Returns:
            The function given as `function` to `__new__`.
        """

@typing.final
class DataType(typing.Generic[_T]):
    """
    Represents a SQL data type.

    A data type is a SQL name plus the serializer used to turn Python objects
    into SQL literals. The name can be any string; it is not limited to a set
    of predefined values. The serializer is fully customizable.

    Known names are interned and canonicalized (`"int"` becomes `"INT"`).
    Unknown names and parameterized spellings (`VARCHAR(255)`) are kept as
    given. Interning applies to the **name string**, not to `DataType`
    instances: `DataType("int")` is not a singleton.

    If `serializer` is omitted at construction, one is inferred from the
    name; unrecognized names get `UnspecifiedSerializer`.
    """

    def __new__(
        cls, name: str, serializer: Serializer[_T] | None = None, /
    ) -> typing.Self:
        """
        Creates a new SQL data type.

        Args:
            name: The SQL type spelling. Not limited to a predefined set.
                    Known names are interned and canonicalized (`"int"`
                    becomes `"INT"`). Unknown names and parameterized
                    spellings (`VARCHAR(255)`) are kept as given.
            serializer: Used when a Python object is rendered as a SQL
                    literal (defaults, inserts, raw SQL). Built-in
                    serializers cover common types; `CustomSerializer`
                    covers the rest. If omitted, a serializer is inferred
                    from `name`; unrecognized names get
                    `UnspecifiedSerializer`.
        """

    @property
    def name(self) -> str:
        """
        Returns the type name.

        Returns:
            The canonical type name. For interned names this may differ in
            casing or spacing from the string passed to the constructor.
        """

    @property
    def serializer(self) -> Serializer[_T]:
        """
        Returns the serializer used to render Python values as SQL literals.

        Returns:
            The `Serializer` associated with this data type.
        """

    def serialize(self, dialect: SqlDialect, object: _T) -> str:
        """
        Renders `object` as a SQL literal for `dialect`.

        Shorthand for `self.serializer.serialize(dialect, object)`.

        Args:
            dialect: A dialect class (`PostgreSQL`) or instance (`PostgreSQL()`).
            object: The Python value to convert.

        Returns:
            A SQL fragment produced by this type's serializer.

        Raises:
            TypeError: If `object` is not an accepted type for this serializer.
            ValueError: If `object` is the right type but cannot be represented.
            NotImplementedError: If `dialect` has no native literal for this
                    type's serializer (bit strings, `ARRAY[…]`, inet, macaddr).
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
