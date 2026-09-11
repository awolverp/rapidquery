import typing

from rapidquery.data_type import DataType, Serializer
from rapidquery.dialect import SqlDialect

_T = typing.TypeVar("_T")

@typing.final
class Value(typing.Generic[_T]):
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

    @typing.overload
    def __new__(
        cls,
        object: _T,
        /,
    ) -> typing.Self: ...
    @typing.overload
    def __new__(
        cls,
        object: _T,
        serializer: Serializer[_T] | DataType[_T] | None = None,
        /,
    ) -> typing.Self:
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
    def object(self) -> _T:
        """
        Returns the wrapped Python object.

        Returns:
            The object passed to the constructor (or taken from an existing
            `Value`).
        """

    @property
    def serializer(self) -> Serializer[_T]:
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
