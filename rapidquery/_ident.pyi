from collections.abc import Iterator, Sequence
from typing import Self, final, overload

from rapidquery.dialect import SqlDialect

@final
class Unquote:
    """
    Marker for an unquoted identifier part.

    The body is emitted without dialect delimiters. It must be a simple
    ASCII name (`[A-Za-z_][A-Za-z0-9_]*`). This is **not** a raw-SQL
    fragment.
    """

    def __new__(cls, name: str, /) -> Self:
        """
        Creates an unquoted identifier part.

        Args:
            name: The identifier body, unquoted.

        Raises:
            ValueError: If `name` is empty or not a simple ASCII identifier
                    (letters, digits, underscore; first character not a digit).
        """

    @property
    def name(self) -> str:
        """
        Returns the unquoted identifier body.

        Returns:
            The string passed to the constructor.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...

@final
class Asterisk:
    """
    SQL wildcard `*`.

    Pass the class (`Asterisk`) or an instance (`Asterisk()`) as an
    identifier part. It must be the last part of an `Ident` (`t.*`).
    `Ident("*")` is a quoted column named star, not this wildcard.
    """

    def __new__(cls) -> Self: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...

@final
class Ident:
    """
    Represents a SQL identifier (`schema.table.column`).

    An identifier is a sequence of parts. Each constructor argument is one
    part: a quoted name (`str`), an `Unquote` name, an `Asterisk` wildcard,
    or a nested `Ident` whose parts are flattened.
    """

    def __new__(cls, *parts: str | Unquote | Ident | Asterisk | type[Asterisk]) -> Self:
        """
        Creates a SQL identifier.

        Args:
            parts: One argument per part. A `str` is always quoted, including
                    `"*"` (a column named star). Pass `Asterisk` for the SQL
                    wildcard. Dotted names are `Ident("schema", "table")`, not
                    `Ident("schema.table")`.

                    **Note**: Sequences are not accepted. `Ident(["table", "id"])`
                    is a `TypeError`; write `Ident("table", "id")`.

        Raises:
            TypeError: If an argument is not `str`, `Unquote`, `Asterisk`, or
                    `Ident`.
            ValueError: If there are no parts, a part is empty, `Asterisk` is
                    not last, an `Unquote` body is not a simple ASCII name, or
                    there are more than 10 parts.
        """

    def __len__(self) -> int: ...
    @overload
    def __getitem__(self, index: int) -> str | Unquote | Asterisk: ...
    @overload
    def __getitem__(self, index: slice) -> Ident: ...
    def __getitem__(self, index: int | slice) -> str | Unquote | Asterisk | Ident:
        """
        Returns a part by index, or a sub-identifier by slice.

        Args:
            index: An `int` (negative indices count from the end) or a
                    `slice`.

        Returns:
            A `str`, `Unquote`, or `Asterisk` for an integer index. An
            `Ident` for a slice.

        Raises:
            IndexError: If the integer index is out of range.
            ValueError: If the slice selects no parts.
        """

    def __iter__(self) -> Iterator[str | Unquote | Asterisk]: ...
    def to_list(self) -> Sequence[str | Unquote | Asterisk]:
        """
        Returns the parts as a list (`str`, `Unquote`, or `Asterisk`).
        """

    def to_sql(self, dialect: type[SqlDialect] | SqlDialect) -> str:
        """
        Renders this identifier as SQL for `dialect`.

        Args:
            dialect: A dialect class (`PostgreSQL`) or instance
                    (`PostgreSQL()`).

        Returns:
            The dotted identifier with each part quoted or emitted according
            to its kind.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...
