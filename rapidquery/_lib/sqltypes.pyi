from __future__ import annotations

import typing
import datetime
import uuid
import enum
import decimal

__all__ = [
    "Array",
    "BigInteger",
    "BigUnsigned",
    "Binary",
    "Bit",
    "Blob",
    "Boolean",
    "Char",
    "Date",
    "DateTime",
    "Decimal",
    "Double",
    "Enum",
    "Float",
    "INET",
    "Integer",
    "JSON",
    "JSONBinary",
    "MacAddress",
    "SQLTypeAbstract",
    "SmallInteger",
    "SmallUnsigned",
    "String",
    "Text",
    "Time",
    "Timestamp",
    "TinyInteger",
    "TinyUnsigned",
    "UUID",
    "Unsigned",
    "VarBinary",
    "VarBit",
    "Vector",
]

T = typing.TypeVar("T")

@typing.final
class Array(SQLTypeAbstract[list[T]]):
    """
    Array column type for storing arrays of elements.

    Represents a column that stores arrays of a specified element type.
    Useful in databases that support native array types (like PostgreSQL)
    for storing lists of values in a single column.
    """

    def __new__(cls, element: SQLTypeAbstract[T]) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def element(self) -> SQLTypeAbstract[T]: ...

@typing.final
class BigInteger(SQLTypeAbstract[int]):
    """
    Large integer column type (BIGINT).

    Stores 64-bit integers for very large numeric values. Essential for
    high-volume systems, timestamps, large counters, or when integer
    overflow is a concern.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class BigUnsigned(SQLTypeAbstract[int]):
    """
    Unsigned big integer column type.

    Stores very large positive integers only. Provides the maximum positive
    integer range for high-volume systems or when very large positive
    values are required.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Binary(SQLTypeAbstract[bytes]):
    """
    Fixed-length binary data column type (BINARY).

    Stores binary data of a fixed length. Values shorter than the specified
    length are padded. Useful for storing hashes, keys, or other binary
    data with consistent length.
    """

    def __new__(cls, length: int = 255) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class Bit(SQLTypeAbstract[bytes]):
    """
    Fixed-length bit string column type (BIT).

    Stores a fixed number of bits. Useful for storing boolean flags efficiently
    or binary data where individual bits have meaning.
    """

    def __new__(cls, length: int) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class Blob(SQLTypeAbstract[bytes]):
    """
    Binary large object column type (BLOB).

    Stores large binary data such as images, documents, audio files, or
    any binary content. Size limits vary by database system.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Boolean(SQLTypeAbstract[bool]):
    """
    Boolean column type (BOOLEAN).

    Stores true/false values. The standard way to store boolean data,
    though implementation varies by database (some use TINYINT(1) or
    similar representations).
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Char(SQLTypeAbstract[str]):
    """
    Fixed-length character string column type (CHAR).

    Represents a fixed-length character string. Values shorter than the
    specified length are padded with spaces. Suitable for storing data
    with consistent, known lengths like country codes or status flags.
    """

    def __new__(cls, length: int | None = ...) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int | None: ...

@typing.final
class Date(SQLTypeAbstract[datetime.date]):
    """
    Date-only column type (DATE).

    Stores date information without time component. Ideal for birth dates,
    deadlines, or any date-based data where time precision is not needed.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class DateTime(SQLTypeAbstract[datetime.datetime]):
    """
    Date and time column type (DATETIME).

    Stores both date and time information without timezone awareness.
    Suitable for recording timestamps, event times, or scheduling information
    when timezone handling is managed at the application level.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Decimal(SQLTypeAbstract[decimal.Decimal | int | float | str]):
    """
    Exact numeric decimal column type (DECIMAL/NUMERIC).

    Stores exact numeric values with fixed precision and scale. Essential for
    financial calculations, currency values, or any situation where exact
    decimal representation is required without floating-point approximation.
    """

    def __new__(cls, context: tuple[int, int] | None = None) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def context(self) -> tuple[int, int] | None: ...

@typing.final
class Double(SQLTypeAbstract[float | int]):
    """
    Double-precision floating point column type (DOUBLE).

    Stores approximate numeric values with double precision. Provides higher
    precision than FLOAT for scientific calculations or when more accuracy
    is required in floating-point operations.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Enum(SQLTypeAbstract[str | enum.Enum]):
    """
    Enumeration column type (ENUM).

    Stores one value from a predefined set of allowed string values.
    Provides type safety and storage efficiency for categorical data
    with a fixed set of possible values.
    """

    def __new__(cls, name: str, variants: typing.Iterable[str]) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def name(self) -> str: ...
    @property
    def variants(self) -> typing.Sequence[str]: ...

@typing.final
class Float(SQLTypeAbstract[float | int]):
    """
    Single-precision floating point column type (FLOAT).

    Stores approximate numeric values with single precision. Suitable for
    scientific calculations, measurements, or any numeric data where some
    precision loss is acceptable in exchange for storage efficiency.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class INET(SQLTypeAbstract[str]):
    """
    Internet address column type (INET).

    Stores IPv4 or IPv6 addresses, with or without subnet specification.
    More flexible than CIDR type, allowing both host addresses and network ranges.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Integer(SQLTypeAbstract[int]):
    """
    Standard integer column type (INTEGER/INT).

    The most common integer type, typically storing 32-bit integers in the
    range -2,147,483,648 to 2,147,483,647 (signed). Suitable for most
    numeric data including IDs, quantities, and counters.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class JSON(SQLTypeAbstract[typing.Any]):
    """
    JSON data column type (JSON).

    Stores JSON documents with validation and indexing capabilities.
    Allows for flexible schema design and complex nested data structures
    while maintaining some query capabilities.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class JSONBinary(SQLTypeAbstract[typing.Any]):
    """
    Binary JSON column type (JSONB).

    Stores JSON documents in a binary format for improved performance.
    Provides faster query and manipulation operations compared to text-based
    JSON storage, with additional indexing capabilities.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class MacAddress(SQLTypeAbstract[str]):
    """
    MAC address column type (MACADDR).

    Stores MAC (Media Access Control) addresses for network devices.
    Provides validation and formatting for 6-byte MAC addresses.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

class SQLTypeAbstract(typing.Generic[T]):
    """
    Base class for all SQL column data types.

    This abstract base class represents SQL data types that can be used in
    column definitions. Each subclass implements a specific SQL data type
    with its particular characteristics, constraints, and backend-specific
    representations.
    """

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class SmallInteger(SQLTypeAbstract[int]):
    """
    Small integer column type (SMALLINT).

    Typically stores integers in the range -32,768 to 32,767 (signed) or
    0 to 65,535 (unsigned). Good for moderate-sized counters or numeric codes.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class SmallUnsigned(SQLTypeAbstract[int]):
    """
    Unsigned small integer column type.

    Stores moderate positive integers only, typically 0 to 65,535. Good for
    larger counters or numeric codes that are always positive.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class String(SQLTypeAbstract[str]):
    """
    Variable-length character string column type (VARCHAR).

    Represents a variable-length character string with a maximum length limit.
    This is the most common string type for storing text data of varying lengths
    like names, descriptions, or user input.
    """

    def __new__(cls, length: int | None = ...) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int | None: ...

@typing.final
class Text(SQLTypeAbstract[str]):
    """
    Large text column type (TEXT).

    Represents a large text field capable of storing long strings without
    a predefined length limit. Suitable for storing articles, comments,
    descriptions, or any text content that may be very long.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Time(SQLTypeAbstract[datetime.time]):
    """
    Time-only column type (TIME).

    Stores time information without date component. Useful for storing
    daily schedules, opening hours, or any time-based data that repeats
    daily regardless of the specific date.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Timestamp(SQLTypeAbstract[datetime.datetime]):
    """
    Timestamp column type (TIMESTAMP).

    Stores timestamp values, often with automatic update capabilities.
    Behavior varies by database system.
    """

    def __new__(cls, timezone: bool = False) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def timezone(self) -> bool: ...

@typing.final
class TinyInteger(SQLTypeAbstract[int]):
    """
    Very small integer column type (TINYINT).

    Typically stores integers in the range -128 to 127 (signed) or 0 to 255
    (unsigned). Useful for flags, small counters, or enumerated values.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class TinyUnsigned(SQLTypeAbstract[int]):
    """
    Unsigned tiny integer column type.

    Stores small positive integers only, typically 0 to 255. Useful for
    small counters, percentages, or enumerated values that are always positive.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class UUID(SQLTypeAbstract[uuid.UUID]):
    """
    UUID column type (UUID).

    Stores universally unique identifiers. Ideal for distributed systems,
    primary keys, or any situation where globally unique identifiers are
    needed without central coordination.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class Unsigned(SQLTypeAbstract[int]):
    """
    Unsigned integer column type.

    Stores positive integers only, typically 0 to 4,294,967,295. Doubles the
    positive range compared to signed integers, useful for IDs and counters
    that will never be negative.
    """

    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

@typing.final
class VarBinary(SQLTypeAbstract[bytes]):
    """
    Variable-length binary data column type (VARBINARY).

    Stores binary data of variable length up to a specified maximum.
    More storage-efficient than BINARY for binary data of varying lengths.
    """

    def __new__(cls, length: int | None = None) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class VarBit(SQLTypeAbstract[bytes]):
    """
    Variable-length bit string column type (VARBIT).

    Stores a variable number of bits up to a specified maximum. More flexible
    than fixed BIT type for bit strings of varying lengths.
    """

    def __new__(cls, length: int) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int: ...

@typing.final
class Vector(SQLTypeAbstract[list[float]]):
    """
    Vector column type for storing mathematical vectors.

    Specialized type for storing vector data, often used in machine learning,
    similarity search, or mathematical applications.
    """

    def __new__(cls, length: int | None = None) -> typing.Self: ...
    def __repr__(self, /) -> str:
        """Return repr(self)."""
        ...

    @property
    def __type_name__(self) -> str:
        """
        Type name. e.g. `'INTEGER'`, `'STRING'`

        It also may be a property. This function must NOT raise any error.
        """
        ...

    @property
    def length(self) -> int | None: ...
