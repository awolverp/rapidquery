import datetime
import decimal
import enum
import uuid
from collections import namedtuple

import pytest

import rapidquery as rq

Case = namedtuple(
    "Case",
    ("data_type", "value", "exc"),
)


class Enum(enum.Enum):
    FIELD = "field"


class StrEnum(enum.StrEnum):
    FIELD = "field"


class IntEnum(enum.IntEnum):
    FIELD = 1


TEST_CASES = [
    # --- BlobType ---
    Case(rq.BlobType(), b"A", None),
    Case(rq.BlobType(), 4.5, TypeError),
    Case(rq.BlobType(), "Ali", TypeError),
    # --- BinaryType ---
    Case(rq.BinaryType(), b"A", None),
    Case(rq.BinaryType(), 4.5, TypeError),
    Case(rq.BinaryType(), "Ali", TypeError),
    # --- VarBinaryType ---
    Case(rq.VarBinaryType(), b"A", None),
    Case(rq.VarBinaryType(), 4.5, TypeError),
    Case(rq.VarBinaryType(), "Ali", TypeError),
    # --- BitType ---
    Case(rq.BitType(8), b"A", None),
    Case(rq.BitType(8), 4.5, TypeError),
    Case(rq.BitType(8), "Ali", TypeError),
    # --- VarBitType ---
    Case(rq.VarBitType(), b"A", None),
    Case(rq.VarBitType(), 4.5, TypeError),
    Case(rq.VarBitType(), "Ali", TypeError),
    # --- DateTimeType ---
    Case(rq.DateTimeType(), datetime.datetime.now(), None),
    Case(rq.DateTimeType(), datetime.datetime.now(tz=datetime.timezone.utc), None),
    Case(rq.DateTimeType(), datetime.datetime.now(tz=datetime.timezone.min), None),
    Case(rq.DateTimeType(), datetime.datetime.now().date(), TypeError),
    Case(rq.DateTimeType(), datetime.datetime.now().time(), TypeError),
    Case(rq.DateTimeType(), 4, TypeError),
    Case(rq.DateTimeType(), 4.5, TypeError),
    Case(rq.DateTimeType(), "Ali", TypeError),
    # --- TimestampType ---
    Case(rq.TimestampType(), datetime.datetime.now(), None),
    Case(rq.TimestampType(True), datetime.datetime.now(tz=datetime.timezone.utc), None),
    Case(rq.TimestampType(), datetime.datetime.now(tz=datetime.timezone.min), None),
    Case(rq.TimestampType(), datetime.datetime.now().date(), TypeError),
    Case(rq.TimestampType(), datetime.datetime.now().time(), TypeError),
    Case(rq.TimestampType(), datetime.datetime.now().timestamp(), None),
    Case(rq.TimestampType(), int(datetime.datetime.now().timestamp()), None),
    Case(rq.TimestampType(), "Ali", TypeError),
    # --- TimeType ---
    Case(rq.TimeType(), datetime.datetime.now(), TypeError),
    Case(rq.TimeType(), datetime.datetime.now().date(), TypeError),
    Case(rq.TimeType(), datetime.datetime.now().time(), None),
    Case(rq.TimeType(), datetime.datetime.now().timestamp(), TypeError),
    Case(rq.TimeType(), int(datetime.datetime.now().timestamp()), TypeError),
    Case(rq.TimeType(), "Ali", TypeError),
    # --- DateType ---
    Case(rq.DateType(), datetime.datetime.now(), TypeError),
    Case(rq.DateType(), datetime.datetime.now().date(), None),
    Case(rq.DateType(), datetime.datetime.now().time(), TypeError),
    Case(rq.DateType(), datetime.datetime.now().timestamp(), TypeError),
    Case(rq.DateType(), int(datetime.datetime.now().timestamp()), TypeError),
    Case(rq.DateType(), "Ali", TypeError),
    # --- JSONType ---
    Case(rq.JSONType(), 4.5, None),
    Case(rq.JSONType(), 4456746532, None),
    Case(rq.JSONType(), "Ali", None),
    Case(rq.JSONType(), {"key": "value"}, None),
    Case(rq.JSONType(), [2, 3], None),
    Case(rq.JSONType(), b"bytes", TypeError),
    Case(rq.JSONType(), (1, 2), TypeError),
    Case(rq.JSONType(), datetime.datetime.now(), TypeError),
    # --- JSONBinaryType ---
    Case(rq.JSONBinaryType(), 4.5, None),
    Case(rq.JSONBinaryType(), 4456746532, None),
    Case(rq.JSONBinaryType(), "Ali", None),
    Case(rq.JSONBinaryType(), {"key": "value"}, None),
    Case(rq.JSONBinaryType(), [2, 3], None),
    Case(rq.JSONBinaryType(), b"bytes", TypeError),
    Case(rq.JSONBinaryType(), (1, 2), TypeError),
    Case(rq.JSONBinaryType(), datetime.datetime.now(), TypeError),
    # --- DecimalType ---
    Case(rq.DecimalType(None), 4.5, None),
    Case(rq.DecimalType((10, 4)), "5.6", None),
    Case(rq.DecimalType(), 4, None),
    Case(rq.DecimalType(), "9e-10", None),
    Case(rq.DecimalType(), decimal.Decimal("1.0"), None),
    Case(rq.DecimalType(), "Ali", ValueError),
    # --- UUIDType ---
    Case(rq.UUIDType(), uuid.uuid4(), None),
    Case(rq.UUIDType(), uuid.uuid4().hex, TypeError),
    Case(rq.UUIDType(), uuid.uuid4().int, TypeError),
    # --- INETType ---
    Case(rq.INETType(), "1.2.3.4", None),
    Case(rq.INETType(), "1.2.3.4/23", None),
    Case(rq.INETType(), "127.0.0.1", None),
    Case(rq.INETType(), "127.0.0.1:8080", ValueError),
    Case(rq.INETType(), "EA:31:7D:14:D8:40", ValueError),
    Case(rq.INETType(), "invalid", ValueError),
    Case(rq.INETType(), uuid.uuid4().hex, ValueError),
    Case(rq.INETType(), 45, TypeError),
    # --- MacAddressType ---
    Case(rq.MacAddressType(), "EA:31:7D:14:D8:40", None),
    Case(rq.MacAddressType(), "1.2.3.4", ValueError),
    Case(rq.MacAddressType(), "invalid", ValueError),
    Case(rq.MacAddressType(), uuid.uuid4().hex, ValueError),
    Case(rq.MacAddressType(), 45, TypeError),
    # --- EnumType ---
    Case(rq.EnumType("USER", ("a", "b")), "string", None),
    Case(rq.EnumType("USER", ("a", "b")), Enum.FIELD, None),
    Case(rq.EnumType("USER", ("a", "b")), StrEnum.FIELD, None),
    Case(rq.EnumType("USER", ("a", "b")), IntEnum.FIELD, TypeError),
    Case(rq.EnumType("USER", ("a", "b")), Enum, TypeError),
    Case(rq.EnumType("USER", ("a", "b")), 4.5, TypeError),
    Case(rq.EnumType("USER", ("a", "b")), 45, TypeError),
    # --- BigIntegerType ---
    Case(rq.BigIntegerType(), 1638479230, None),
    Case(rq.BigIntegerType(), -1638479230, None),
    Case(rq.BigIntegerType(), 4.5, TypeError),
    Case(rq.BigIntegerType(), "Ali", TypeError),
    Case(rq.BigIntegerType(), 9223372036854775807, None),
    Case(rq.BigIntegerType(), 9223372036854775807 + 1, OverflowError),
    # --- IntegerType ---
    Case(rq.IntegerType(), 37, None),
    Case(rq.IntegerType(), -37, None),
    Case(rq.IntegerType(), 4.5, TypeError),
    Case(rq.IntegerType(), "Ali", TypeError),
    Case(rq.IntegerType(), 2147483647, None),
    Case(rq.IntegerType(), 2147483647 + 1, OverflowError),
    # --- SmallIntegerType ---
    Case(rq.SmallIntegerType(), 647, None),
    Case(rq.SmallIntegerType(), -647, None),
    Case(rq.SmallIntegerType(), 4.5, TypeError),
    Case(rq.SmallIntegerType(), "Ali", TypeError),
    Case(rq.SmallIntegerType(), 32767, None),
    Case(rq.SmallIntegerType(), 32767 + 1, OverflowError),
    # --- TinyIntegerType ---
    Case(rq.TinyIntegerType(), 10, None),
    Case(rq.TinyIntegerType(), -10, None),
    Case(rq.TinyIntegerType(), 4.5, TypeError),
    Case(rq.TinyIntegerType(), "Ali", TypeError),
    Case(rq.TinyIntegerType(), 127, None),
    Case(rq.TinyIntegerType(), 127 + 1, OverflowError),
    # --- BigUnsignedType ---
    Case(rq.BigUnsignedType(), 89438302, None),
    Case(rq.BigUnsignedType(), -1, OverflowError),
    Case(rq.BigUnsignedType(), 4.5, TypeError),
    Case(rq.BigUnsignedType(), "Ali", TypeError),
    Case(rq.BigUnsignedType(), 18446744073709551615, None),
    Case(rq.BigUnsignedType(), 18446744073709551615 + 1, OverflowError),
    # --- UnsignedType ---
    Case(rq.UnsignedType(), 89438302, None),
    Case(rq.UnsignedType(), -1, OverflowError),
    Case(rq.UnsignedType(), 4.5, TypeError),
    Case(rq.UnsignedType(), "Ali", TypeError),
    Case(rq.UnsignedType(), 4294967295, None),
    Case(rq.UnsignedType(), 4294967295 + 1, OverflowError),
    # --- SmallUnsignedType ---
    Case(rq.SmallUnsignedType(), 978, None),
    Case(rq.SmallUnsignedType(), -1, OverflowError),
    Case(rq.SmallUnsignedType(), 4.5, TypeError),
    Case(rq.SmallUnsignedType(), "Ali", TypeError),
    Case(rq.SmallUnsignedType(), 65535, None),
    Case(rq.SmallUnsignedType(), 65535 + 1, OverflowError),
    # --- TinyUnsignedType ---
    Case(rq.TinyUnsignedType(), 20, None),
    Case(rq.TinyUnsignedType(), -1, OverflowError),
    Case(rq.TinyUnsignedType(), 4.5, TypeError),
    Case(rq.TinyUnsignedType(), "Ali", TypeError),
    Case(rq.TinyUnsignedType(), 255, None),
    Case(rq.TinyUnsignedType(), 255 + 1, OverflowError),
    # --- FloatType ---
    Case(rq.FloatType(), 20.0, None),
    Case(rq.FloatType(), 20, None),
    Case(rq.FloatType(), 4.5, None),
    Case(rq.FloatType(), "Ali", TypeError),
    # --- DoubleType ---
    Case(rq.DoubleType(), 20.0, None),
    Case(rq.DoubleType(), 20, None),
    Case(rq.DoubleType(), 4.5, None),
    Case(rq.DoubleType(), "Ali", TypeError),
    # --- TextType ---
    Case(rq.TextType(), "A", None),
    Case(rq.TextType(), "A" * 10000, None),
    Case(rq.TextType(), b"B", TypeError),
    Case(rq.TextType(), 1, TypeError),
    Case(rq.TextType(), 1.4, TypeError),
    # --- StringType ---
    Case(rq.StringType(), "A", None),
    Case(rq.StringType(), "A" * 10000, None),
    Case(rq.StringType(), b"B", TypeError),
    Case(rq.StringType(), 1, TypeError),
    Case(rq.StringType(), 1.4, TypeError),
    # --- CharType ---
    Case(rq.CharType(), "A", None),
    Case(rq.CharType(), "A" * 10000, None),
    Case(rq.CharType(), b"B", TypeError),
    Case(rq.CharType(), 1, TypeError),
    Case(rq.CharType(), 1.4, TypeError),
    # --- ArrayType ---
    Case(rq.ArrayType(rq.IntegerType()), [1, 2], None),
    Case(rq.ArrayType(rq.IntegerType()), [1, 2], None),
    Case(rq.ArrayType(rq.IntegerType()), (1, 2, 5), None),
    Case(rq.ArrayType(rq.FloatType()), [1.3, 2.7], None),
    Case(rq.ArrayType(rq.IntegerType()), [1, "A"], TypeError),
    Case(rq.ArrayType(rq.IntegerType()), [1, 2, 3, 4, 5, 6, 7, 8, "A", 10], TypeError),
    Case(rq.ArrayType(rq.TextType()), ["B", "A"], TypeError),
    # --- VectorType ---
    Case(rq.VectorType(6), [1.4, 2.1], None),
    Case(rq.VectorType(5), [1, 2], None),
    Case(rq.VectorType(4), (1, 2, 5.3), None),
    Case(rq.VectorType(3), [1.3, 2.7], None),
    Case(rq.VectorType(2), [1, "A"], TypeError),
    Case(rq.VectorType(1), [1, 2, 3, 4, 5, 6, 7, 8, "A", 10], TypeError),
    Case(rq.VectorType(), ["B", "A"], TypeError),
]


class TestSQLType:
    @pytest.mark.parametrize("case", TEST_CASES)
    def test_validate_serialize(self, case: Case):
        try:
            val = rq.Value(case.value, case.data_type)
            rq.Expr(val)  # Force Value to adapt
        except Exception as e:
            if case.exc is not None and type(e) is case.exc:
                return

            raise e

        assert val.sql_type is case.data_type
        assert val.value == case.value

    @pytest.mark.parametrize("case", TEST_CASES)
    def test_type_name(self, case: Case):
        assert case.data_type.__type_name__
