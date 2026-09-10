from typing import Any, Self, final

from rapidquery.data_type import DataType, Serializer
from rapidquery.dialect import SqlDialect

@final
class Value:
    """
    A Python object plus the serializer used to emit it as SQL.

    `Value` does not convert the object until SQL is built. Parameterized
    statements bind the object as a placeholder. Raw SQL runs the serializer
    (`None` becomes `NULL`).

    If `serializer` is omitted, one is inferred from the object's type.
    Passing an existing `Value` keeps its object (and its serializer, unless
    a new one is given). A `DataType` as `serializer` contributes only its
    serializer; the data type is not stored.
    """

    def __new__(
        cls,
        object: Any,
        serializer: Serializer | DataType | None = None,
        /,
    ) -> Self:
        """
        Creates a SQL value.

        Args:
            object: The Python object to bind or serialize. If this is already
                    a `Value`, its object is reused.
            serializer: How to render `object` as a SQL literal. A `DataType`
                    is accepted and only its serializer is kept. If omitted,
                    a serializer is inferred from `object` (or kept, when
                    `object` is already a `Value`).

        Raises:
            TypeError: If `serializer` is not a `Serializer` or `DataType`.
        """

    @property
    def object(self) -> Any:
        """
        Returns the wrapped Python object.

        Returns:
            The object passed to the constructor (or taken from an existing
            `Value`).
        """

    @property
    def serializer(self) -> Serializer:
        """
        Returns the serializer used for raw SQL literals.

        Returns:
            The `Serializer` stored on this value. Not used when the value
            is bound as a parameter.
        """

    def to_sql(self, dialect: type[SqlDialect] | SqlDialect) -> str:
        """
        Renders this value as a SQL literal for `dialect`.

        Raw path: `None` becomes `NULL`; otherwise the serializer writes a
        literal. This is not the parameterized form.

        Args:
            dialect: A dialect class (`PostgreSQL`) or instance
                    (`PostgreSQL()`).

        Returns:
            A SQL fragment suitable for embedding in a statement.

        Raises:
            TypeError: If `object` is not an accepted type for the serializer.
            ValueError: If `object` is the right type but cannot be represented.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...
