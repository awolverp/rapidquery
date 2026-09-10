from collections.abc import Callable
from typing import Any, Self, final

from rapidquery.dialect import SqlDialect

class Serializer:
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

    def serialize(self, dialect: type[SqlDialect] | SqlDialect, object: Any) -> str:
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
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...

@final
class IntegerSerializer(Serializer):
    """
    Serializes Python integers as bare SQL integer literals.

    Accepts Python `int` of any width (including values beyond 64-bit).
    Rejects `bool`; `True` and `False` are not treated as integers.

    Raises:
        TypeError: If the value is not an `int`.
    """

    def __new__(cls) -> Self: ...

@final
class FloatSerializer(Serializer):
    """
    Serializes numeric values as bare SQL numeric literals.

    Accepts `float`, `int`, and `decimal.Decimal`. Finite floats are formatted
    so that `1.0` stays distinct from `1`. `str` is not accepted.

    Raises:
        TypeError: If the value is not a `float`, `int`, or `decimal.Decimal`.
        ValueError: If the value is a non-finite float (`nan`, `inf`, `-inf`).
    """

    def __new__(cls) -> Self: ...

@final
class BooleanSerializer(Serializer):
    """
    Serializes Python booleans as dialect boolean literals.

    Accepts Python `bool` only. The dialect chooses the spelling
    (`TRUE`/`FALSE` or `1`/`0`).

    Raises:
        TypeError: If the value is not a `bool`.
    """

    def __new__(cls) -> Self: ...

@final
class StringSerializer(Serializer):
    """
    Serializes text as a quoted, escaped SQL string literal.

    Accepts `str`, or an `enum.Enum` member whose `.value` is a `str`. The
    dialect supplies quoting and escaping.

    Raises:
        TypeError: If the value is not a `str` or a string-valued enum member.
    """

    def __new__(cls) -> Self: ...

@final
class UuidSerializer(Serializer):
    """
    Serializes UUIDs as quoted SQL string literals.

    Accepts `uuid.UUID` or `str`.

    Raises:
        TypeError: If the value is not a `uuid.UUID` or `str`.
    """

    def __new__(cls) -> Self: ...

@final
class BytesSerializer(Serializer):
    """
    Serializes binary data as a dialect blob/bytea literal.

    Accepts exact `bytes` (not `bytearray` or `memoryview`).

    Raises:
        TypeError: If the value is not `bytes`.
    """

    def __new__(cls) -> Self: ...

@final
class BitSerializer(Serializer):
    """
    Serializes bit strings as dialect bit-string literals.

    Accepts a `str` consisting only of `'0'` and `'1'` characters.

    Raises:
        TypeError: If the value is not a `str`.
        ValueError: If the string is empty or contains characters other than `'0'` and `'1'`.
    """

    def __new__(cls) -> Self: ...

@final
class JSONSerializer(Serializer):
    """
    Serializes a JSON-encodable object as a quoted SQL string literal.

    Accepts any object that the JSON encoder can dump. Uses `orjson` when it
    is installed, otherwise the stdlib `json` module. The JSON text is then
    wrapped with the dialect's string quoting. With `orjson`, datetime, UUID,
    dataclass, and non-str dict keys are encoded natively; stdlib `json`
    rejects those.

    Raises:
        TypeError: If the value cannot be JSON-encoded.
    """

    def __new__(cls) -> Self: ...

@final
class DateSerializer(Serializer):
    """
    Serializes dates as quoted `YYYY-MM-DD` literals.

    Accepts exact `datetime.date` (not `datetime.datetime`).

    Raises:
        TypeError: If the value is not a `datetime.date`.
    """

    def __new__(cls) -> Self: ...

@final
class TimeSerializer(Serializer):
    """
    Serializes times as quoted `HH:MM:SS[.ffffff]` literals.

    Accepts `datetime.time`.

    Raises:
        TypeError: If the value is not a `datetime.time`.
    """

    def __new__(cls) -> Self: ...

@final
class DatetimeSerializer(Serializer):
    """
    Serializes timestamps as quoted SQL timestamp literals.

    Accepts naive or aware `datetime.datetime`, or `int`/`float` UTC epoch
    seconds. Naive values keep wall time. Aware values keep their original
    offset (they are not converted to UTC).

    Raises:
        TypeError: If the value is not a `datetime.datetime`, `int`, or `float`.
        OverflowError: If an epoch value is out of range for a timestamp.
    """

    def __new__(cls) -> Self: ...

@final
class InetSerializer(Serializer):
    """
    Serializes IP addresses and networks as quoted SQL literals.

    Accepts `str` or `ipaddress` objects (`IPv4Address`, `IPv6Address`,
    `IPv4Network`, `IPv6Network`, and related types whose `str()` is a valid
    address or CIDR).

    Raises:
        ValueError: If the value is not a valid IPv4/IPv6 address or network.
    """

    def __new__(cls) -> Self: ...

@final
class MacAddressSerializer(Serializer):
    """
    Serializes MAC addresses as quoted SQL literals.

    Accepts a `str` using `:` or `-` as the separator.

    Raises:
        ValueError: If the value is not a valid MAC address.
    """

    def __new__(cls) -> Self: ...

@final
class UnspecifiedSerializer(Serializer):
    """
    Inserts `str(obj)` into SQL with no quoting or type check.

    Used when no serializer could be inferred from the type name. The result
    is not a safe SQL value; prefer an explicit serializer.
    """

    def __new__(cls) -> Self: ...

@final
class ArraySerializer(Serializer):
    """
    Serializes sequences as `ARRAY[elem, ...]` literals.

    Each element is serialized independently with `element`. Accepts `list`
    or `tuple`. Nested arrays are expressed by wrapping another
    `ArraySerializer`.
    """

    def __new__(cls, element: Serializer) -> Self:
        """
        Creates an array serializer.

        Args:
            element: The serializer applied to every array item. Nested arrays
                    are expressed by passing another `ArraySerializer`.
        """

    @property
    def element(self) -> Serializer:
        """
        Returns the serializer used for each array element.

        Returns:
            The `Serializer` passed to the constructor.
        """

@final
class CustomSerializer(Serializer):
    """
    Serializes values by calling a user-supplied Python function.

    `function` has the form `fn(dialect_name: str, obj) -> str`. The return
    value is inserted into SQL as-is (not re-quoted or escaped). It is the
    caller's responsibility to produce a safe fragment.
    """

    def __new__(cls, function: Callable[[str, Any], str]) -> Self:
        """
        Creates a serializer backed by a Python callable.

        Args:
            function: A callable of the form `fn(dialect_name: str, obj) -> str`.
                    The return value is inserted into SQL as-is (not re-quoted).
                    Must return a `str`.
        """

    @property
    def function(self) -> Callable[[str, Any], str]:
        """
        Returns the callable passed to the constructor.

        Returns:
            The function given as `function` to `__new__`.
        """

@final
class DataType:
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

    def __new__(cls, name: str, serializer: Serializer | None = None, /) -> Self:
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
    def serializer(self) -> Serializer:
        """
        Returns the serializer used to render Python values as SQL literals.

        Returns:
            The `Serializer` associated with this data type.
        """

    def serialize(self, dialect: type[SqlDialect] | SqlDialect, object: Any) -> str:
        """
        Renders `object` as a SQL literal for `dialect`.

        Shorthand for `self.serializer.serialize(dialect, object)`.

        Args:
            dialect: A dialect class (`PostgreSQL`) or instance (`PostgreSQL()`).
            object: The Python value to convert.

        Returns:
            A SQL fragment produced by this type's serializer.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...
