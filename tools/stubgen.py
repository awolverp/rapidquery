"""
A custom stub generator which imports compiled library and uses docstrings for
understanding and generating documents.

- Uses `@signature` tag in docstring, and fallbacks to `__text_signature__` and `__signature__`
- Detects `TypeVar`s automatically from subclasses
- Uses `@extends` tag in docstring to determine subclasses
- Uses `@return` (or `@signature`) tag in docstring to determine return type; but can automatically detect return
  type on special methods.
- Detects @typing.final
- Uses `__stub_imports__` attribute on module to detect imports
- Uses `@readonly` to detect a readonly property
"""

import dataclasses
import enum
import importlib
import inspect
import re
import sys
import types
import typing

_TAG_PATTERN = re.compile(r"^\s*@(\w+)(?:\s+(.+?))?\s*$")
_TYPEVAR_PATTERN = re.compile(r"\b([A-Z])\b")


def _split_tag_values(tagval: str | None) -> list[str]:
    """Split a comma-separated tag value, filtering empty parts."""
    if not tagval:
        return []
    return [item for item in (item.strip() for item in tagval.split(",")) if item]


@dataclasses.dataclass
class Docstring:
    document: str | None = None
    """A clean document without tags"""

    extends: list[str] = dataclasses.field(default_factory=list)
    """
    subclasses ( specified basses ).

    Example: "MySubclass", "typing.Generic[T, S]"

    Usage:
    @extends typing.Generic[T, S]
    """

    signature: str | None = None
    """
    Return type.

    Usage:
    @signature (cls, value: int)
    """

    return_type: str | None = None
    """
    Return type.

    Usage:
    @return int
    """

    setter_type: str | None = None
    """
    If present, a setter will be generated with this parameter type.
    If None, the property is read-only.

    Usage:
    @setter int
    """

    @classmethod
    def parse(
        cls,
        obj,
        custom_signature: str | None = None,
        custom_return_type: str | None = None,
    ) -> typing.Self:
        if isinstance(obj, str) or obj is None:
            docstring = obj
        else:
            docstring = inspect.getdoc(obj)

        if docstring is None:
            return cls()

        assert isinstance(docstring, str)

        clean_document: list[str] = []
        result = cls()

        pending_signature = False

        for line in docstring.splitlines():
            if pending_signature:
                result.signature += line  # type: ignore
                pending_signature = "->" not in result.signature and result.signature[-1] != ")"
                continue

            match = _TAG_PATTERN.match(line)

            if match is None:
                clean_document.append(line)
                continue

            tagname, tagval = match.group(1), match.group(2)

            if tagname == "extends":
                result.extends.extend(_split_tag_values(tagval))

            elif tagname == "return":
                if tagval:
                    result.return_type = tagval.strip()

            elif tagname == "signature":
                if tagval:
                    result.signature = tagval.strip()
                    pending_signature = (
                        "->" not in result.signature and result.signature[-1] != ")"  # type: ignore
                    )

            elif tagname == "setter":
                result.setter_type = tagval.strip() if tagval else "Incomplete"

        # Strip leading/trailing blank lines from the prose portion
        document = "\n".join(clean_document).strip()
        result.document = document or None

        if custom_return_type is not None:
            result.return_type = custom_return_type

        if custom_signature is not None:
            result.signature = custom_signature

        return result


class ImplementationType(enum.IntEnum):
    MODULE = enum.auto()
    CLASS = enum.auto()
    FUNCTION = enum.auto()
    BUILTIN_METHOD = enum.auto()
    METHOD_DESCRIPTOR = enum.auto()
    GETSET_DESCRIPTOR = enum.auto()
    UNKNOWN = enum.auto()

    def is_function(self) -> bool:
        return self in (
            ImplementationType.FUNCTION,
            ImplementationType.BUILTIN_METHOD,
            ImplementationType.METHOD_DESCRIPTOR,
        )

    def is_class(self) -> bool:
        return self == ImplementationType.CLASS

    def is_module(self) -> bool:
        return self == ImplementationType.MODULE

    def is_getset(self) -> bool:
        return self == ImplementationType.GETSET_DESCRIPTOR

    @classmethod
    def guess(cls, object):
        if inspect.ismodule(object):
            return cls.MODULE

        if inspect.isclass(object):
            return cls.CLASS

        if inspect.ismethoddescriptor(object):
            return cls.METHOD_DESCRIPTOR

        if inspect.isgetsetdescriptor(object):
            return cls.GETSET_DESCRIPTOR

        if isinstance(object, types.BuiltinMethodType):
            return cls.BUILTIN_METHOD

        if callable(object):
            return cls.FUNCTION

        return cls.UNKNOWN


def _detect_final(obj: typing.Type):
    try:

        class IsNotFinal(obj):
            pass

    except TypeError:
        return True

    else:
        return False


def _normalize_signature(signature: str, obj) -> str | None:
    if signature.startswith(("($cls", "($self")):
        signature = "(" + signature[2:]

    if inspect.isclass(obj):
        if signature == "()":
            signature = "(cls)"

        elif not signature.startswith("(cls"):
            signature = "(cls, " + signature[1:]

    return signature


def _get_signature(obj) -> str | None:
    raw_signature = getattr(obj, "__text_signature__", None)
    if raw_signature is not None:
        return _normalize_signature(raw_signature, obj)

    try:
        spec = inspect.getfullargspec(obj)
    except TypeError:
        try:
            return str(inspect.signature(obj))
        except TypeError:
            return None

    args = spec.args or []
    defaults = spec.defaults or ()
    annotations = spec.annotations or {}

    # Pad defaults to align with args (defaults apply to the last N args)
    num_without_defaults = len(args) - len(defaults)
    padded_defaults = [None] * num_without_defaults + list(defaults)

    def format_annotation(name: str) -> str:
        return f": {annotations[name].__name__}" if name in annotations else ""

    def format_default(default) -> str:
        return f" = {default!r}" if default is not None else ""

    # Regular args
    parts = []
    for arg, default in zip(args, padded_defaults):
        annotation = format_annotation(arg)
        default_str = format_default(default)
        parts.append(f"{arg}{annotation}{default_str}")

    # *args
    if spec.varargs:
        annotation = format_annotation(spec.varargs)
        parts.append(f"*{spec.varargs}{annotation}")
    elif spec.kwonlyargs:
        parts.append("*")  # Bare * to separate kwonly args

    # Keyword-only args
    kwonly_defaults = spec.kwonlydefaults or {}
    for kwarg in spec.kwonlyargs:
        annotation = format_annotation(kwarg)
        default_str = f" = {kwonly_defaults[kwarg]!r}" if kwarg in kwonly_defaults else ""
        parts.append(f"{kwarg}{annotation}{default_str}")

    # **kwargs
    if spec.varkw:
        annotation = format_annotation(spec.varkw)
        parts.append(f"**{spec.varkw}{annotation}")

    return _normalize_signature(f"({', '.join(parts)})", obj)


RETURN_TYPE_BY_NAME: dict[str, str] = {
    # --- Representation ---
    "__repr__": "str",
    "__str__": "str",
    "__format__": "str",
    "__bytes__": "bytes",
    "__fspath__": "str",
    # --- Numeric coercions ---
    "__bool__": "bool",
    "__int__": "int",
    "__float__": "float",
    "__complex__": "complex",
    "__index__": "int",
    "__trunc__": "int",
    "__floor__": "int",
    "__ceil__": "int",
    "__round__": "int",
    "__abs__": "int",
    "__sizeof__": "int",
    "__hash__": "int",
    "__len__": "int",
    "__length_hint__": "int",
    # --- Comparisons (return bool, not Self) ---
    "__eq__": "bool",
    "__ne__": "bool",
    "__lt__": "bool",
    "__le__": "bool",
    "__gt__": "bool",
    "__ge__": "bool",
    "__contains__": "bool",
    # --- Lifecycle (return Self or None) ---
    "__new__": "typing.Self",
    "__init__": "None",
    "__copy__": "typing.Self",
    "__deepcopy__": "typing.Self",
    "__pos__": "typing.Self",
    "__neg__": "typing.Self",
    "__invert__": "typing.Self",
    # --- In-place operators (return Self) ---
    "__iadd__": "typing.Self",
    "__isub__": "typing.Self",
    "__imul__": "typing.Self",
    "__itruediv__": "typing.Self",
    "__ifloordiv__": "typing.Self",
    "__imod__": "typing.Self",
    "__ipow__": "typing.Self",
    "__imatmul__": "typing.Self",
    "__ilshift__": "typing.Self",
    "__irshift__": "typing.Self",
    "__iand__": "typing.Self",
    "__ior__": "typing.Self",
    "__ixor__": "typing.Self",
    # --- Context managers ---
    "__enter__": "typing.Self",
    "__exit__": "bool",
    "__aenter__": "typing.Self",
    "__aexit__": "bool",
    # --- Iteration ---
    "__iter__": "typing.Iterator[Incomplete]",
    "__aiter__": "typing.AsyncIterator[Incomplete]",
    "__reversed__": "typing.Iterator[Incomplete]",
    # --- Async ---
    "__await__": "typing.Generator[typing.Any, None, typing.Self]",
    "__anext__": "Incomplete",
    "__next__": "Incomplete",
    # --- Deletion (always None) ---
    "__del__": "None",
    "__delattr__": "None",
    "__delitem__": "None",
    "__delslice__": "None",
    # --- Setters (always None) ---
    "__init_subclass__": "None",
    "__set_name__": "None",
    "__setattr__": "None",
    "__setitem__": "None",
    "__set__": "None",
}


OVERRIDE_SPECIAL_METHODS: dict[str, Docstring] = {
    "Expr.__eq__": Docstring(
        document="Create an equality comparison expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__ne__": Docstring(
        document="Create an inequality comparison expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__gt__": Docstring(
        document="Create a greater-than comparison expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__ge__": Docstring(
        document="Create a greater-than-or-equal comparison expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__lt__": Docstring(
        document="Create a less-than comparison expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__le__": Docstring(
        document="Create a less-than-or-equal comparison expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__add__": Docstring(
        document="Create an addition expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__sub__": Docstring(
        document="Create a subtraction expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__and__": Docstring(
        document="Create a logical AND expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__neg__": Docstring(
        document="Create a negation expression.",
        signature="(self)",
        return_type="typing.Self",
    ),
    "Expr.__or__": Docstring(
        document="Create a logical OR expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__truediv__": Docstring(
        document="Create a division expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__lshift__": Docstring(
        document="Create a bitwise left shift expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__rshift__": Docstring(
        document="Create a bitwise right shift expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__mod__": Docstring(
        document="Create a modulo expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
    "Expr.__mul__": Docstring(
        document="Create a multiplication expression.",
        signature="(self, other: object)",
        return_type="typing.Self",
    ),
}


def _override_docstring(obj) -> Docstring | None:
    objclass = getattr(obj, "__objclass__", None)
    if objclass is None:
        return

    key = objclass.__name__ + "." + obj.__name__
    try:
        return OVERRIDE_SPECIAL_METHODS[key]
    except KeyError:
        pass

    if obj.__name__.startswith("__r"):
        # try without __r
        key = objclass.__name__ + ".__" + obj.__name__[3:]

        try:
            return OVERRIDE_SPECIAL_METHODS[key]
        except KeyError:
            pass


@dataclasses.dataclass
class Implementation:
    """An implementation - a function, class, property, ..."""

    type: ImplementationType
    docstring: Docstring
    typevars: list[str] = dataclasses.field(default_factory=list)
    final: bool = False
    ignore_comment: str = ""

    @classmethod
    def parse(
        cls,
        obj,
        custom_signature: str | None = None,
        custom_return_type: str | None = None,
    ):
        type = ImplementationType.guess(obj)

        if type == ImplementationType.UNKNOWN:
            raise TypeError(f"Unknown implementation type: {obj}")

        docstring = Docstring.parse(obj, custom_signature, custom_return_type)

        # Override docstring if need
        if type.is_function():
            if _override := _override_docstring(obj):
                docstring = _override

        result = Implementation(
            type=type,
            docstring=docstring,
            final=_detect_final(obj) if type.is_class() else False,
        )

        # Detect subclasses
        if not result.docstring.extends and type.is_class():
            result.docstring.extends = list(
                map(
                    lambda x: x.__name__,
                    filter(lambda x: x is not object, obj.__bases__),
                )
            )

        # Detect signature
        if not result.docstring.signature:
            result.docstring.signature = _get_signature(obj)
        else:
            result.docstring.signature = _normalize_signature(result.docstring.signature, obj)

        assert result.docstring.signature is not None

        # Remove `-> type` from signature and use it for return_type if possible
        if "->" in result.docstring.signature:
            result.docstring.signature, _signature_return_type = result.docstring.signature.split(
                "->", 1
            )
        else:
            _signature_return_type = None

        # Detect return type
        if not result.docstring.return_type:
            if _signature_return_type:
                result.docstring.return_type = _signature_return_type.strip()

            elif type.is_function():
                result.docstring.return_type = RETURN_TYPE_BY_NAME.get(obj.__name__, "Incomplete")
            elif type.is_class():
                result.docstring.return_type = RETURN_TYPE_BY_NAME.get("__new__", "Incomplete")
            else:
                result.docstring.return_type = "Incomplete"

        # Detect typevars
        typevars = []

        for subclass in result.docstring.extends:
            typevars.extend(_TYPEVAR_PATTERN.findall(subclass))

        if result.docstring.return_type:
            typevars.extend(_TYPEVAR_PATTERN.findall(result.docstring.return_type))

        if result.docstring.signature:
            typevars.extend(_TYPEVAR_PATTERN.findall(result.docstring.signature))

        result.typevars = list(dict.fromkeys(typevars))

        # Detect `type: ignore`
        if result.docstring.return_type != "Incomplete":
            if type.is_function():
                _real_return_type = RETURN_TYPE_BY_NAME.get(
                    obj.__name__, result.docstring.return_type
                )
            elif type.is_class():
                _real_return_type = RETURN_TYPE_BY_NAME.get("__new__", result.docstring.return_type)
            else:
                _real_return_type = result.docstring.return_type

            if result.docstring.return_type != _real_return_type:
                result.ignore_comment = "# type: ignore[override]"

        return result


def detect_classmethod(class_, member) -> bool:
    sig = getattr(member, "__text_signature__", "") or ""
    return (
        getattr(member, "__self__", None) is class_
        or member.__name__.startswith("from_")
        or "(cls" in sig
        or "$cls" in sig
    )


class StubGenerator:
    def __init__(self, root: types.ModuleType) -> None:
        # root module
        self.root = root

        # content lines
        self._import_lines: list[str] = [
            "import typing",
            "from _typeshed import Incomplete",
        ]
        self._typevar_lines: list[str] = []
        self._body_lines: list[str] = []

        # special members
        self.undefined_members: list[tuple[str, typing.Any]] = []

        # Manage indent
        self._indent = 0

    def _append_import_lines(self, *lines: str) -> None:
        self._import_lines.extend(lines)

    def _append_body_lines(self, *lines: str) -> None:
        self._body_lines.extend(((" " * self._indent) + i) for i in lines)

    def _append_document(self, document: str | None) -> None:
        if not document:
            return

        if len(document) > 70:
            self._append_body_lines('"""')
            self._append_body_lines(*document.splitlines())
            self._append_body_lines('"""')
        else:
            self._append_body_lines('"""' + document.strip() + '"""')

    def _append_typevars(self, names: typing.Iterable[str]) -> None:
        for name in names:
            self._typevar_lines.append("%s = typing.TypeVar(%r)" % (name, name))

    def _visit_module(self, module: types.ModuleType) -> None:
        for member_key, member_val in inspect.getmembers(module):
            if member_key == "__stub_imports__":
                self._import_lines.extend(member_val)
                continue

            if member_key.startswith("__"):
                continue

            member_type = ImplementationType.guess(member_val)

            if member_type.is_class():
                # Class definition
                self._visit_class(member_val)

            elif member_type.is_function():
                # Function definition
                self._visit_function(member_val)

            elif member_key.isupper():
                # Constant variable definition
                self._visit_constant(member_key, member_val)

            elif member_type.is_module():
                # Module definition
                raise NotImplementedError

            else:
                self.undefined_members.append((member_key, member_val))

    def _visit_function(
        self,
        function: typing.Union[
            types.FunctionType,
            types.MethodDescriptorType,
            types.BuiltinMethodType,
        ],
        # Very useful for __new__ methods
        custom_signature: str | None = None,
        custom_return_type: str | None = None,
        # Useful for `@classmethod`s
        parent: type | None = None,
    ) -> None:
        impl = Implementation.parse(function, custom_signature, custom_return_type)

        self._append_typevars(impl.typevars)

        # Append newline only for formatting
        self._append_body_lines("")

        if parent and detect_classmethod(parent, function):
            self._append_body_lines("@classmethod")

        self._append_body_lines(
            "def %s%s -> %s: %s"
            % (
                function.__name__,
                impl.docstring.signature,
                impl.docstring.return_type,
                impl.ignore_comment,
            )
        )

        self._indent += 4
        self._append_document(impl.docstring.document)
        self._append_body_lines("...")
        self._indent -= 4

    def _visit_getset_descriptor(
        self,
        descriptor: types.GetSetDescriptorType,
    ):
        impl = Implementation.parse(descriptor)

        # Append newline only for formatting
        self._append_body_lines("")

        self._append_body_lines(
            "@property",
            "def %s(self) -> %s:" % (descriptor.__name__, impl.docstring.return_type),
        )

        self._indent += 4
        self._append_document(impl.docstring.document)
        self._append_body_lines("...")
        self._indent -= 4

        if impl.docstring.setter_type is not None:
            self._append_body_lines(
                f"@{descriptor.__name__}.setter",
                f"def {descriptor.__name__}(self, value: {impl.docstring.setter_type}) -> None:",
            )
            self._indent += 4
            self._append_body_lines("...")
            self._indent -= 4

    def _visit_class(self, class_: typing.Type) -> None:
        impl = Implementation.parse(class_)

        self._append_typevars(impl.typevars)

        # Append newline only for formatting
        self._append_body_lines("")

        if impl.final:
            self._append_body_lines("@typing.final")

        if impl.docstring.extends:
            _line = "class %s(%s):" % (
                class_.__name__,
                ", ".join(impl.docstring.extends),
            )
        else:
            _line = "class %s:" % class_.__name__

        self._append_body_lines(_line)

        self._indent += 4
        self._append_document(impl.docstring.document)

        # Implement non-empty `__new__`
        if impl.docstring.signature and impl.docstring.signature != "(cls)":
            self._visit_function(
                class_.__new__,
                custom_signature=impl.docstring.signature,
                custom_return_type=impl.docstring.return_type,
            )

        hash_is_none = False

        for attr_key, attr_val in inspect.getmembers(class_):
            attr_type = ImplementationType.guess(attr_val)

            if attr_key in ("__new__", "__doc__", "__class__", "__module__"):
                continue

            if attr_key == "__hash__" and attr_val is None:
                hash_is_none = True
                continue

            if attr_type == ImplementationType.METHOD_DESCRIPTOR:
                if attr_val.__objclass__ is class_:
                    # Method definition
                    self._visit_function(attr_val, parent=class_)

            elif attr_type == ImplementationType.BUILTIN_METHOD:
                if not attr_key.startswith("__") and attr_val.__self__ is class_:
                    self._visit_function(attr_val, parent=class_)

            elif attr_type == ImplementationType.GETSET_DESCRIPTOR:
                if attr_val.__objclass__ is class_:
                    # Property definition
                    self._visit_getset_descriptor(attr_val)

            elif attr_key.isupper():
                self._visit_constant(attr_key, attr_val)

            else:
                self.undefined_members.append(
                    (
                        "%s.%s" % (class_.__name__, attr_key),
                        attr_val,
                    )
                )

        if hash_is_none:
            self._append_body_lines("", "__hash__ = None  # type: ignore")

        self._indent -= 4

    def _visit_constant(self, name: str, value: typing.Any) -> None:
        self._append_body_lines("%s: typing.Final[%s] = ..." % (name, type(value).__name__))

    def generate(self) -> typing.Self:
        self._visit_module(self.root)
        return self

    def result(self) -> str:
        content = ""

        root_doc = Docstring.parse(self.root)
        if root_doc.document:
            content += '"""\n%s\n"""\n\n' % root_doc.document

        content += "from __future__ import annotations\n"

        content += "\n".join(dict.fromkeys(self._import_lines))
        content += "\n\n"
        content += "\n".join(dict.fromkeys(self._typevar_lines))
        content += "\n\n"
        content += "\n".join(self._body_lines)

        return content


def main():
    if len(sys.argv) != 2 or sys.argv[1] in ("help", "-h", "--help"):
        print(f"Usage:\n\t{sys.argv[0]} IMPORT_NAME")
        return

    _, import_name = sys.argv
    module_root = importlib.import_module(import_name)

    stub = StubGenerator(module_root).generate()

    for key, val in stub.undefined_members:
        print(f"Warning: undefined member: {key}\n\t{val}", file=sys.stderr)

    print(stub.result())


if __name__ == "__main__":
    main()
