"""Data Definition Language"""

from typing import Self, final

from rapidquery.dialect import Dialect
from rapidquery.expr import Ident, _IdentExtractor

class DataDefinitionLanguage:
    """Base of every DDL statement. Construct a subclass, not this class."""

    def to_sql(self, dialect: Dialect) -> str:
        """
        Render ``self`` with values written as SQL literals.

        Args:
            dialect: Dialect that chooses quoting and literal spelling.

        Returns:
            The SQL text.

        Raises:
            RuntimeError: The dialect rejects this construct.
            ValueError: A `CustomExpr` template has a different number of `{}`
                slots than arguments.
        """

@final
class CreateSchema(DataDefinitionLanguage):
    """Represents ``CREATE SCHEMA`` statement."""

    def __new__(
        cls,
        name: _IdentExtractor,
        if_not_exists: bool = False,
        authorization: str | None = None,
        location: str | None = None,
    ) -> Self:
        """
        Construct a new ``CREATE SCHEMA`` statement.

        Args:
            name: schema name e.g. `"analysis"`. Some dialects supports multipart names (like `"public"."analysis"`)
                  so there's no limit and it is in your control.
            if_not_exists: Emits ``IF NOT EXISTS`` clause. Means do nothing if a schema with the same name
                           already exists. *If targeted dialect doesn't support this, it will be ignored.*
            authorization: Emits ``AUTHORIZATION name`` clause. Some dialects allow us to names the owner of the new
                           schema.
            location: Some dialects allow us to specify a location for the new schema. It binds the schema to a directory,
                      which can be a local FS path, HDFS, or even S3.
        """
        ...

    @property
    def name(self) -> Ident: ...
    @property
    def if_not_exists(self) -> bool: ...
    @property
    def authorization(self) -> str | None: ...
    @property
    def location(self) -> str | None: ...
