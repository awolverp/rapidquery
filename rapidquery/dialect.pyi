from typing import Self, final

class SqlDialect:
    """
    Abstract SQL dialect.

    Concrete dialects are frozen subclasses (`PostgreSql`, `DuckDb`, ...).
    Pass an instance wherever a dialect is required.
    This base class cannot be instantiated.

    Instances of the same dialect compare equal and hash together.
    """
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

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
