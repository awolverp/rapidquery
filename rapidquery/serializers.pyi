import datetime
import decimal
import enum
import typing
import uuid
from collections.abc import Callable

from rapidquery.dialect import Dialect

_T = typing.TypeVar("_T")

class Serializer(typing.Generic[_T]):
    def serialize(self, dialect: Dialect, object: _T) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

@typing.final
class IntegerSerializer(Serializer[int]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class FloatSerializer(Serializer[int | float | decimal.Decimal]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class BooleanSerializer(Serializer[bool]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class StringSerializer(Serializer[str | enum.Enum]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class UuidSerializer(Serializer[uuid.UUID | str]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class BytesSerializer(Serializer[bytes]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class BitSerializer(Serializer[str]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class JSONSerializer(Serializer[typing.Any]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class DateSerializer(Serializer[datetime.date]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class TimeSerializer(Serializer[datetime.time]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class DatetimeSerializer(Serializer[datetime.datetime | int | float]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class InetSerializer(Serializer[typing.Any]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class MacAddressSerializer(Serializer[str]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class UnspecifiedSerializer(Serializer[typing.Any]):
    def __new__(cls) -> typing.Self: ...

@typing.final
class ArraySerializer(Serializer[list[_T] | tuple[_T]]):
    def __new__(cls, element: Serializer[_T]) -> typing.Self: ...
    @property
    def element(self) -> Serializer[_T]: ...

@typing.final
class CustomSerializer(Serializer[_T]):
    def __new__(cls, function: Callable[[Dialect, _T], str]) -> typing.Self: ...
    @property
    def function(self) -> Callable[[Dialect, _T], str]: ...
