from collections.abc import Sequence
from typing import final

@final
class BigQueryDialect:
    """
    A dialect for [Google Bigquery](https://cloud.google.com/bigquery/)
    """
    def __new__(cls, /) -> BigQueryDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class DatabricksDialect:
    """
    A dialect for [Databricks SQL](https://www.databricks.com/)
    
    See <https://docs.databricks.com/en/sql/language-manual/index.html>.
    """
    def __new__(cls, /) -> DatabricksDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class DuckDbDialect:
    """
    A dialect for [DuckDB](https://duckdb.org/)
    """
    def __new__(cls, /) -> DuckDbDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class HiveDialect:
    """
    A [`Dialect`] for [Hive](https://hive.apache.org/).
    """
    def __new__(cls, /) -> HiveDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class MsSqlDialect:
    """
    A dialect for [Microsoft SQL Server](https://www.microsoft.com/en-us/sql-server/)
    """
    def __new__(cls, /) -> MsSqlDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class MySqlDialect:
    """
    A dialect for [MySQL](https://www.mysql.com/)
    """
    def __new__(cls, /) -> MySqlDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class OracleDialect:
    """
    A [`Dialect`] for [Oracle Databases](https://docs.oracle.com/en/database/oracle/oracle-database/21/sqlrf/index.html)
    """
    def __new__(cls, /) -> OracleDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class PostgreSqlDialect:
    """
    A dialect for [PostgreSQL](https://www.postgresql.org/)
    """
    def __new__(cls, /) -> PostgreSqlDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class RedshiftDialect:
    """
    A dialect for [RedShift](https://aws.amazon.com/redshift/)
    """
    def __new__(cls, /) -> RedshiftDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class SQLiteDialect:
    """
    A dialect for [SQLite](https://www.sqlite.org)
    """
    def __new__(cls, /) -> SQLiteDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class SnowflakeDialect:
    """
    A dialect for [Snowflake](https://www.snowflake.com/)
    """
    def __new__(cls, /) -> SnowflakeDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class SparkDialect:
    """
    A [`Dialect`] for [Apache Spark SQL](https://spark.apache.org/docs/latest/sql-ref.html).
    
    See <https://spark.apache.org/docs/latest/sql-ref-syntax.html>.
    """
    def __new__(cls, /) -> SparkDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """

@final
class TeradataDialect:
    """
    A Dialect for [Teradata](https://docs.teradata.com/).
    """
    def __new__(cls, /) -> TeradataDialect: ...
    def encode_bits(self, /, bits: str) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Raises `ValueError` if `bits` includes something other than `0` and `1`.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bits("10101010001") == "1361"
        ```
        """
    def encode_bytes(self, /, bytes: Sequence[int]) -> str:
        """
        Render a byte slice as a literal in the configured form.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.encode_bytes(b"bytes") == "X'6279746573'"
        ```
        """
    def escape_string(self, /, s: str) -> str:
        """
        Escapes string literals.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.escape_string("A'Wolver'P\0") == "A''Wolver''P"
        ```
        """
    def get_identifier_quotes(self, /) -> tuple[str, str] |None:
        """
        Returns the start/end characters used to quote identifiers.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_identifier_quotes() == ('"', '"')
        ```
        """
    def get_name(self, /) -> str:
        """
        Returns dialect name
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.get_name() == "SQLite"
        ```
        """
    def quote_string(self, /, s: str) -> str:
        """
        Quotes string literals. this method DOES NOT escape characters.
        
        Example:
        ```python
        dialect = SQLiteDialect()
        assert dialect.quote_string("string") == "'string'"
        ```
        """
