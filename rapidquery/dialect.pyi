from typing import Self, final

class SqlDialect:
    """
    Abstract SQL dialect.

    Concrete dialects are frozen subclasses (`PostgreSql`, `DuckDb`, ...).
    Pass an instance wherever a dialect is required.
    This base class cannot be instantiated.

    Instances of the same dialect compare equal and hash together.
    """

    @property
    def positional_parameter(self) -> str: ...
    @property
    def bit_format(self) -> str | None: ...
    @property
    def supports_array(self) -> bool: ...
    @property
    def supports_inet(self) -> bool: ...
    @property
    def supports_mac_address(self) -> bool: ...
    @property
    def null_treatment_syntax(self) -> str: ...
    @property
    def like_any_syntax(self) -> str: ...
    @property
    def bool_format(self) -> str: ...
    @property
    def identifier_format(self) -> str: ...
    @property
    def identifier_escape(self) -> str: ...
    @property
    def string_format(self) -> str: ...
    @property
    def string_escape(self) -> str: ...
    @property
    def bytes_format(self) -> str: ...
    def format_boolean(self, value: bool) -> str: ...
    def escape_string(self, string: str) -> str: ...
    def quote_identifier(self, ident: str) -> str: ...
    def write_string_literal(self, raw: str) -> str: ...
    def format_bits(self, bits: str) -> str: ...
    def format_bytes(self, data: bytes) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class GenericDialect(SqlDialect):
    def __new__(
        cls,
        *,
        positional_parameter: str,
        bit_format: str | None,
        supports_array: bool,
        supports_inet: bool,
        supports_mac_address: bool,
        null_treatment_syntax: str,
        like_any_syntax: str,
        bool_format: str,
        identifier_format: str,
        identifier_escape: str,
        string_format: str,
        string_escape: str,
        bytes_format: str,
    ) -> Self: ...

@final
class BigQuery(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Databricks(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class DuckDb(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Hive(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class MsSql(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class MySql(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Oracle(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class PostgreSql(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Redshift(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Snowflake(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Spark(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Sqlite(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Teradata(SqlDialect):
    def __new__(cls) -> Self: ...
