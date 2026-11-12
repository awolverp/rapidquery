"""RapidQuery exceptions that are not Python builtins."""

class Error(Exception):
    """Base of RapidQuery errors that are not a builtin.

    Catch this to handle every library-specific failure. ``TypeError``,
    ``ValueError``, and ``IndexError`` are raised as those builtins when that
    meaning applies. This class itself is not raised.

    Examples:
        >>> from rapidquery.exc import DialectError, Error, SerializeError
        >>> issubclass(DialectError, Error) and issubclass(SerializeError, Error)
        True
    """

class DialectError(Error):
    """This dialect cannot render a construct RapidQuery otherwise accepts.

    Raised while rendering, and while serializing a literal, when a feature
    flag is off, a literal template is absent, or the dialect options make a
    clause impossible. The same object may render on another dialect.

    Examples:
        >>> from rapidquery.ddl import CreateSchema
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.exc import DialectError
        >>> try:
        ...     CreateSchema("app").to_sql(Sqlite)
        ... except DialectError as exc:
        ...     "CREATE SCHEMA" in str(exc)
        True
    """

class SerializeError(Error):
    """A Python object could not be written as a SQL literal.

    Raised when a serializer rejects the object's type or its value. A dialect
    with no literal spelling for that kind raises `DialectError` instead.

    Examples:
        >>> from rapidquery.dialect import Sqlite
        >>> from rapidquery.exc import SerializeError
        >>> from rapidquery.serializers import IntegerSerializer
        >>> try:
        ...     IntegerSerializer().serialize(Sqlite, "nope")
        ... except SerializeError as exc:
        ...     "IntegerSerializer" in str(exc)
        True
    """
