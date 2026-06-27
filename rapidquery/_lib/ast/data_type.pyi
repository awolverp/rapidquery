from rapidquery._lib.dialect import BigQueryDialect, DatabricksDialect, DuckDbDialect, HiveDialect, MsSqlDialect, MySqlDialect, OracleDialect, PostgreSqlDialect, RedshiftDialect, SnowflakeDialect, SparkDialect, SqliteDialect, TeradataDialect
from typing import Any, Callable, final

@final
class ArraySerializer:
    """
    Serializes a Python `list`/`tuple` by serializing each element with `element`
    and wrapping in `ARRAY[...]`.
    """
    def __new__(cls, /, element: 'ArraySerializer' |'BitSerializer' |'BooleanSerializer' |'BytesSerializer' |'DateSerializer' |'DatetimeSerializer' |'FloatSerializer' |'InetSerializer' |'IntegerSerializer' |'JSONSerializer' |'MacAddressSerializer' |'StringSerializer' |'TimeSerializer' |'UnspecifiedSerializer' |'UuidSerializer' |Callable[[BigQueryDialect |DatabricksDialect |DuckDbDialect |HiveDialect |MsSqlDialect |MySqlDialect |OracleDialect |PostgreSqlDialect |RedshiftDialect |SnowflakeDialect |SparkDialect |SqliteDialect |TeradataDialect, str], str]) -> ArraySerializer: ...

@final
class BitSerializer:
    """
    Serializes a bit string given as a `str` of `0`/`1` (e.g. `"1010"`) into a
    `B'1010'` literal.
    """
    def __new__(cls, /) -> BitSerializer: ...

@final
class BooleanSerializer:
    def __new__(cls, /) -> BooleanSerializer: ...

@final
class BytesSerializer:
    """
    Serializes a `uuid.UUID` (or 36-char `str`) as a quoted literal.
    """
    def __new__(cls, /) -> BytesSerializer: ...

@final
class DataType:
    """
    SQL data type.
    """
    def __new__(cls, /, name: str, serializer: 'ArraySerializer' |'BitSerializer' |'BooleanSerializer' |'BytesSerializer' |'DateSerializer' |'DatetimeSerializer' |'FloatSerializer' |'InetSerializer' |'IntegerSerializer' |'JSONSerializer' |'MacAddressSerializer' |'StringSerializer' |'TimeSerializer' |'UnspecifiedSerializer' |'UuidSerializer' |Callable[[BigQueryDialect |DatabricksDialect |DuckDbDialect |HiveDialect |MsSqlDialect |MySqlDialect |OracleDialect |PostgreSqlDialect |RedshiftDialect |SnowflakeDialect |SparkDialect |SqliteDialect |TeradataDialect, str], str] |None = None) -> DataType: ...
    def __repr__(self, /) -> str: ...
    @property
    def name(self, /) -> str: ...
    def serialize(self, /, dialect: BigQueryDialect |DatabricksDialect |DuckDbDialect |HiveDialect |MsSqlDialect |MySqlDialect |OracleDialect |PostgreSqlDialect |RedshiftDialect |SnowflakeDialect |SparkDialect |SqliteDialect |TeradataDialect |type[BigQueryDialect] |type[DatabricksDialect] |type[DuckDbDialect] |type[HiveDialect] |type[MsSqlDialect] |type[MySqlDialect] |type[OracleDialect] |type[PostgreSqlDialect] |type[RedshiftDialect] |type[SnowflakeDialect] |type[SparkDialect] |type[SqliteDialect] |type[TeradataDialect], value: Any) -> str: ...
    @property
    def serializer(self, /) -> Any: ...

@final
class DateSerializer:
    """
    `datetime.date` -> `'YYYY-MM-DD'`.
    """
    def __new__(cls, /) -> DateSerializer: ...

@final
class DatetimeSerializer:
    """
    `datetime.datetime` -> `'YYYY-MM-DD HH:MM:SS[.ffffff][+HH:MM]'`.
    
    `int`/`float` are treated as a UTC epoch and accepted too.
    """
    def __new__(cls, /) -> DatetimeSerializer: ...

@final
class FloatSerializer:
    """
    Serializes Python `float`, `int`, `decimal.Decimal`, or numeric `str` as a
    bare numeric literal.
    
    `Decimal`/`str`/`int` go through their exact text form (no precision loss);
    only a real `float` is formatted from the f64. Non-finite floats are rejected.
    """
    def __new__(cls, /) -> FloatSerializer: ...

@final
class InetSerializer:
    """
    `str` (or `ipaddress.*`) -> quoted literal. Validated as IPv4/IPv6 network.
    """
    def __new__(cls, /) -> InetSerializer: ...

@final
class IntegerSerializer:
    """
    Serializes any Python `int` as a bare decimal literal.
    
    Uses `str(int)` so arbitrary precision (beyond i64/u64) is preserved - one
    serializer genuinely covers TINYINT through BIGINT and the unsigned variants.
    """
    def __new__(cls, /) -> IntegerSerializer: ...

@final
class JSONSerializer:
    """
    Serializes any JSON-encodable Python object via `json.dumps`, then wraps the
    result as a quoted string literal.
    """
    def __new__(cls, /) -> JSONSerializer: ...

@final
class MacAddressSerializer:
    """
    `str` -> quoted literal. Validated as a MAC address.
    """
    def __new__(cls, /) -> MacAddressSerializer: ...

@final
class StringSerializer:
    """
    Serializes a Python `str` (or `enum.Enum` whose value is a str) as a quoted,
    escaped string literal: `"LIKE"` -> `'LIKE'`.
    """
    def __new__(cls, /) -> StringSerializer: ...

@final
class TimeSerializer:
    """
    `datetime.time` -> `'HH:MM:SS[.ffffff]'`.
    """
    def __new__(cls, /) -> TimeSerializer: ...

@final
class UnspecifiedSerializer:
    """
    When you don't know about data type, you can use this. Uses values' `__str__` method
    and return it as a serialized SQL value.
    """
    def __new__(cls, /) -> UnspecifiedSerializer: ...

@final
class UuidSerializer:
    """
    Serializes a `uuid.UUID` (or 36-char `str`) as a quoted literal.
    """
    def __new__(cls, /) -> UuidSerializer: ...
