from typing import Self, final

class SqlDialect:
    @property
    def name(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@final
class BigQuery(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Databricks(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class DuckDB(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Hive(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class MSSQL(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class MySql(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Oracle(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class PostgreSQL(SqlDialect):
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
class SQLite(SqlDialect):
    def __new__(cls) -> Self: ...

@final
class Teradata(SqlDialect):
    def __new__(cls) -> Self: ...
