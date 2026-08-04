from typing import Any, Callable, Type, final

from typing_extensions import Self

from rapidquery._native.dialect import SqlDialect

class Serializer:
    """
    Converts a Python object into a dialect-specific SQL literal string.

    Serializers are responsible for producing a correctly escaped and formatted
    SQL value representation that can be embedded in queries. Each concrete
    subclass handles a particular category of values (integers, strings, dates,
    arrays, etc.). A custom serializer can be created via `CustomSerializer`
    when the built-in ones are insufficient.

    The primary entry point is the `serialize` method, which receives the target
    SQL dialect and the Python object to convert.
    """

    def serialize(self, dialect: Type[SqlDialect] | SqlDialect, object: Any) -> str: ...

@final
class IntegerSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class FloatSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class BooleanSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class StringSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class UuidSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class BytesSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class BitSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class JSONSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class DateSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class TimeSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class DatetimeSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class InetSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class MacAddressSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class UnspecifiedSerializer(Serializer):
    def __new__(cls) -> Self: ...

@final
class ArraySerializer(Serializer):
    def __new__(cls, element: Serializer) -> Self: ...
    @property
    def element(self) -> Serializer: ...

@final
class CustomSerializer(Serializer):
    def __new__(cls, function: Callable[[str, Any], str]) -> Self: ...
    @property
    def function(self) -> Callable[[str, Any], str]: ...

@final
class DataType:
    """
    Reperesents a SQL data type. It includes a name, and a serializer.

    The name of a SQL data type can be any string; it is not limited to a set of predefined values.
    Additionally, the serializer for each data type is fully customizable.
    """

    def __new__(cls, name: str, serializer: Serializer | None = None, /) -> Self:
        """
        Creates a new SQL data type.

        Args:
            name: The name of a data type. *It is not limited to a set of predefined values.*

                  **Important memory optimization note**: At import time, the library pre-allocates
                  and interns (reserves in memory) several simple and commonly used data types
                  (e.g. `INTEGER`, `VARCHAR`, `TIMESTAMP`, `BOOLEAN`, etc.). If you instantiate
                  a `DataType` using one of these common names, the library will automatically
                  return a reference to the pre-allocated instance instead of creating a new one.
                  This reduces memory consumption and improves performance when working with
                  standard SQL types.

                  As a result of this interning mechanism, when you access the `.name` property
                  on a pre-allocated instance, the returned string **may be slightly different**
                  from the exact string you passed in (for example in casing, spacing, or canonical
                  form chosen by the library). This is expected behavior and does not affect
                  correctness.

            serializer: The `serializer` is used whenever a Python object needs to be converted
                        into a valid SQL value during query construction. This is commonly required
                        when setting default values, inserting data, or generating other SQL statements.
                        You can use one of the built-in serializers (we provide ready-to-use
                        serializers for most common data types), or you can create and register your
                        own custom serializer. This parameter is optional. However, it is strongly
                        recommended to always specify it explicitly. If you do not provide a serializer,
                        the `DataType` will attempt to automatically detect and assign one based on the
                        `name`. If no suitable serializer is found, it will fall back to `UnspecifiedSerializer`.
        """
        ...

    @property
    def name(self) -> str:
        """
        Returns the name.

        Note: For pre-allocated common data types (see `__new__` documentation), the returned
        value may be a canonicalized version of the original input string.
        """
        ...
    @property
    def serializer(self) -> Serializer: ...
    def serialize(self, dialect: Type[SqlDialect] | SqlDialect, object: Any) -> str:
        """
        Shorthand for `self.serializer.serialize(dialect, object)`.
        """
        ...
