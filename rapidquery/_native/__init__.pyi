import typing

from . import data_type as data_type
from . import dialect as dialect

class Ident:
    def __new__(cls, content: str, /) -> typing.Self: ...

@typing.final
class Asterisk(Ident):
    def __new__(cls) -> typing.Self: ...
