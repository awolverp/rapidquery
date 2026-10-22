"""Serializers that turn a Python object into a SQL literal.

Pass one to `Value` when inference is wrong, or call `serialize` to render a
single literal. Each builtin covers one Python type. `ArraySerializer` and
`CustomSerializer` cover lists and caller-defined spellings. The dialect
still chooses quotes, keywords, and whether the form exists.

Examples:
    Force a string serializer instead of inferring one from the object:

    >>> from rapidquery.dialect import PostgreSql
    >>> from rapidquery.expr import Value
    >>> from rapidquery.serializers import StringSerializer
    >>> Value("hi", StringSerializer()).to_sql(PostgreSql)
    "E'hi'"
"""

from ._core.serializers import (
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

__all__ = [
    "ArraySerializer",
    "BitSerializer",
    "BooleanSerializer",
    "BytesSerializer",
    "CustomSerializer",
    "DateSerializer",
    "DatetimeSerializer",
    "FloatSerializer",
    "InetSerializer",
    "IntegerSerializer",
    "JSONSerializer",
    "MacAddressSerializer",
    "Serializer",
    "StringSerializer",
    "TimeSerializer",
    "UnspecifiedSerializer",
    "UuidSerializer",
]
