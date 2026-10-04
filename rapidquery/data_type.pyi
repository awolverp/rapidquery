"""SQL data type names."""

import typing

@typing.final
class DataType:
    """
    Represents a SQL data type.

    A data type is a SQL type name. The name can be any string; it is not
    limited to a set of predefined values.

    Known names are interned and canonicalized (`"int"` becomes `"INT"`).
    Unknown names and parameterized spellings (`VARCHAR(255)`) are kept as
    given. Interning applies to the **name string**, not to `DataType`
    instances: `DataType("int")` is not a singleton.
    """

    def __new__(cls, name: str, /) -> typing.Self:
        """
        Creates a new SQL data type.

        Args:
            name: The SQL type spelling. Not limited to a predefined set.
                    Known names are interned and canonicalized (`"int"`
                    becomes `"INT"`). Unknown names and parameterized
                    spellings (`VARCHAR(255)`) are kept as given.
        """

    @property
    def name(self) -> str:
        """
        Returns the type name.

        Returns:
            The canonical type name. For interned names this may differ in
            casing or spacing from the string passed to the constructor.
        """

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
