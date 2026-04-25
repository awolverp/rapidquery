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


class StrEnum(str, enum.Enum):
    FIELD = "field"


class IntEnum(int, enum.Enum):
    FIELD = 1


TEST_CASES = [
    # --- BlobType ---
    Case(rq.sqltypes.Blob(), b"A", None),
    Case(rq.sqltypes.Blob(), 4.5, TypeError),
    Case(rq.sqltypes.Blob(), "Ali", TypeError),
    # --- BinaryType ---
    Case(rq.sqltypes.Binary(), b"A", None),
    Case(rq.sqltypes.Binary(), 4.5, TypeError),
    Case(rq.sqltypes.Binary(), "Ali", TypeError),
    # --- VarBinaryType ---
    Case(rq.sqltypes.VarBinary(), b"A", None),
    Case(rq.sqltypes.VarBinary(), 4.5, TypeError),
    Case(rq.sqltypes.VarBinary(), "Ali", TypeError),
    # --- BitType ---
    Case(rq.sqltypes.Bit(8), b"A", None),
    Case(rq.sqltypes.Bit(8), 4.5, TypeError),
    Case(rq.sqltypes.Bit(8), "Ali", TypeError),
    # --- VarBitType ---
    Case(rq.sqltypes.VarBit(8), b"A", None),
    Case(rq.sqltypes.VarBit(8), 4.5, TypeError),
    Case(rq.sqltypes.VarBit(8), "Ali", TypeError),
    # --- DateTimeType ---
    Case(rq.sqltypes.DateTime(), datetime.datetime.now(), None),
    Case(rq.sqltypes.DateTime(), datetime.datetime.now(tz=datetime.timezone.utc), None),
    Case(rq.sqltypes.DateTime(), datetime.datetime.now(tz=datetime.timezone.min), None),
    Case(rq.sqltypes.DateTime(), datetime.datetime.now().date(), TypeError),
    Case(rq.sqltypes.DateTime(), datetime.datetime.now().time(), TypeError),
    Case(rq.sqltypes.DateTime(), 4, TypeError),
    Case(rq.sqltypes.DateTime(), 4.5, TypeError),
    Case(rq.sqltypes.DateTime(), "Ali", TypeError),
    # --- TimestampType ---
    Case(rq.sqltypes.Timestamp(), datetime.datetime.now(), None),
    Case(
        rq.sqltypes.Timestamp(True),
        datetime.datetime.now(tz=datetime.timezone.utc),
        None,
    ),
    Case(rq.sqltypes.Timestamp(), datetime.datetime.now(tz=datetime.timezone.min), None),
    Case(rq.sqltypes.Timestamp(), datetime.datetime.now().date(), TypeError),
    Case(rq.sqltypes.Timestamp(), datetime.datetime.now().time(), TypeError),
    Case(rq.sqltypes.Timestamp(), datetime.datetime.now().timestamp(), None),
    Case(rq.sqltypes.Timestamp(), int(datetime.datetime.now().timestamp()), None),
    Case(rq.sqltypes.Timestamp(), "Ali", TypeError),
    # --- TimeType ---
    Case(rq.sqltypes.Time(), datetime.datetime.now(), TypeError),
    Case(rq.sqltypes.Time(), datetime.datetime.now().date(), TypeError),
    Case(rq.sqltypes.Time(), datetime.datetime.now().time(), None),
    Case(rq.sqltypes.Time(), datetime.datetime.now().timestamp(), TypeError),
    Case(rq.sqltypes.Time(), int(datetime.datetime.now().timestamp()), TypeError),
    Case(rq.sqltypes.Time(), "Ali", TypeError),
    # --- DateType ---
    Case(rq.sqltypes.Date(), datetime.datetime.now(), TypeError),
    Case(rq.sqltypes.Date(), datetime.datetime.now().date(), None),
    Case(rq.sqltypes.Date(), datetime.datetime.now().time(), TypeError),
    Case(rq.sqltypes.Date(), datetime.datetime.now().timestamp(), TypeError),
    Case(rq.sqltypes.Date(), int(datetime.datetime.now().timestamp()), TypeError),
    Case(rq.sqltypes.Date(), "Ali", TypeError),
    # --- JSONType ---
    Case(rq.sqltypes.JSON(), 4.5, None),
    Case(rq.sqltypes.JSON(), 4456746532, None),
    Case(rq.sqltypes.JSON(), "Ali", None),
    Case(rq.sqltypes.JSON(), {"key": "value"}, None),
    Case(rq.sqltypes.JSON(), [2, 3], None),
    Case(rq.sqltypes.JSON(), b"bytes", TypeError),
    Case(rq.sqltypes.JSON(), (1, 2), TypeError),
    Case(rq.sqltypes.JSON(), datetime.datetime.now(), TypeError),
    # --- JSONBinaryType ---
    Case(rq.sqltypes.JSONBinary(), 4.5, None),
    Case(rq.sqltypes.JSONBinary(), 4456746532, None),
    Case(rq.sqltypes.JSONBinary(), "Ali", None),
    Case(rq.sqltypes.JSONBinary(), {"key": "value"}, None),
    Case(rq.sqltypes.JSONBinary(), [2, 3], None),
    Case(rq.sqltypes.JSONBinary(), b"bytes", TypeError),
    Case(rq.sqltypes.JSONBinary(), (1, 2), TypeError),
    Case(rq.sqltypes.JSONBinary(), datetime.datetime.now(), TypeError),
    # --- DecimalType ---
    Case(rq.sqltypes.Decimal(None), 4.5, None),
    Case(rq.sqltypes.Decimal((10, 4)), "5.6", None),
    Case(rq.sqltypes.Decimal(), 4, None),
    Case(rq.sqltypes.Decimal(), "9e-10", None),
    Case(rq.sqltypes.Decimal(), decimal.Decimal("1.0"), None),
    Case(rq.sqltypes.Decimal(), "Ali", ValueError),
    # --- UUIDType ---
    Case(rq.sqltypes.UUID(), uuid.uuid4(), None),
    Case(rq.sqltypes.UUID(), uuid.uuid4().hex, TypeError),
    Case(rq.sqltypes.UUID(), uuid.uuid4().int, TypeError),
    # --- INETType ---
    Case(rq.sqltypes.INET(), "1.2.3.4", None),
    Case(rq.sqltypes.INET(), "1.2.3.4/23", None),
    Case(rq.sqltypes.INET(), "127.0.0.1", None),
    Case(rq.sqltypes.INET(), "127.0.0.1:8080", ValueError),
    Case(rq.sqltypes.INET(), "EA:31:7D:14:D8:40", ValueError),
    Case(rq.sqltypes.INET(), "invalid", ValueError),
    Case(rq.sqltypes.INET(), uuid.uuid4().hex, ValueError),
    Case(rq.sqltypes.INET(), 45, TypeError),
    # --- MacAddressType ---
    Case(rq.sqltypes.MacAddress(), "EA:31:7D:14:D8:40", None),
    Case(rq.sqltypes.MacAddress(), "1.2.3.4", ValueError),
    Case(rq.sqltypes.MacAddress(), "invalid", ValueError),
    Case(rq.sqltypes.MacAddress(), uuid.uuid4().hex, ValueError),
    Case(rq.sqltypes.MacAddress(), 45, TypeError),
    # --- EnumType ---
    Case(rq.sqltypes.Enum("USER", ("a", "b")), "string", None),
    Case(rq.sqltypes.Enum("USER", ("a", "b")), Enum.FIELD, None),
    Case(rq.sqltypes.Enum("USER", ("a", "b")), StrEnum.FIELD, None),
    Case(rq.sqltypes.Enum("USER", ("a", "b")), IntEnum.FIELD, TypeError),
    Case(rq.sqltypes.Enum("USER", ("a", "b")), Enum, TypeError),
    Case(rq.sqltypes.Enum("USER", ("a", "b")), 4.5, TypeError),
    Case(rq.sqltypes.Enum("USER", ("a", "b")), 45, TypeError),
    # --- BigIntegerType ---
    Case(rq.sqltypes.BigInteger(), 1638479230, None),
    Case(rq.sqltypes.BigInteger(), -1638479230, None),
    Case(rq.sqltypes.BigInteger(), 4.5, TypeError),
    Case(rq.sqltypes.BigInteger(), "Ali", TypeError),
    Case(rq.sqltypes.BigInteger(), 9223372036854775807, None),
    Case(rq.sqltypes.BigInteger(), 9223372036854775807 + 1, OverflowError),
    # --- IntegerType ---
    Case(rq.sqltypes.Integer(), 37, None),
    Case(rq.sqltypes.Integer(), -37, None),
    Case(rq.sqltypes.Integer(), 4.5, TypeError),
    Case(rq.sqltypes.Integer(), "Ali", TypeError),
    Case(rq.sqltypes.Integer(), 2147483647, None),
    Case(rq.sqltypes.Integer(), 2147483647 + 1, OverflowError),
    # --- SmallIntegerType ---
    Case(rq.sqltypes.SmallInteger(), 647, None),
    Case(rq.sqltypes.SmallInteger(), -647, None),
    Case(rq.sqltypes.SmallInteger(), 4.5, TypeError),
    Case(rq.sqltypes.SmallInteger(), "Ali", TypeError),
    Case(rq.sqltypes.SmallInteger(), 32767, None),
    Case(rq.sqltypes.SmallInteger(), 32767 + 1, OverflowError),
    # --- TinyIntegerType ---
    Case(rq.sqltypes.TinyInteger(), 10, None),
    Case(rq.sqltypes.TinyInteger(), -10, None),
    Case(rq.sqltypes.TinyInteger(), 4.5, TypeError),
    Case(rq.sqltypes.TinyInteger(), "Ali", TypeError),
    Case(rq.sqltypes.TinyInteger(), 127, None),
    Case(rq.sqltypes.TinyInteger(), 127 + 1, OverflowError),
    # --- BigUnsignedType ---
    Case(rq.sqltypes.BigUnsigned(), 89438302, None),
    Case(rq.sqltypes.BigUnsigned(), -1, OverflowError),
    Case(rq.sqltypes.BigUnsigned(), 4.5, TypeError),
    Case(rq.sqltypes.BigUnsigned(), "Ali", TypeError),
    Case(rq.sqltypes.BigUnsigned(), 18446744073709551615, None),
    Case(rq.sqltypes.BigUnsigned(), 18446744073709551615 + 1, OverflowError),
    # --- UnsignedType ---
    Case(rq.sqltypes.Unsigned(), 89438302, None),
    Case(rq.sqltypes.Unsigned(), -1, OverflowError),
    Case(rq.sqltypes.Unsigned(), 4.5, TypeError),
    Case(rq.sqltypes.Unsigned(), "Ali", TypeError),
    Case(rq.sqltypes.Unsigned(), 4294967295, None),
    Case(rq.sqltypes.Unsigned(), 4294967295 + 1, OverflowError),
    # --- SmallUnsignedType ---
    Case(rq.sqltypes.SmallUnsigned(), 978, None),
    Case(rq.sqltypes.SmallUnsigned(), -1, OverflowError),
    Case(rq.sqltypes.SmallUnsigned(), 4.5, TypeError),
    Case(rq.sqltypes.SmallUnsigned(), "Ali", TypeError),
    Case(rq.sqltypes.SmallUnsigned(), 65535, None),
    Case(rq.sqltypes.SmallUnsigned(), 65535 + 1, OverflowError),
    # --- TinyUnsignedType ---
    Case(rq.sqltypes.TinyUnsigned(), 20, None),
    Case(rq.sqltypes.TinyUnsigned(), -1, OverflowError),
    Case(rq.sqltypes.TinyUnsigned(), 4.5, TypeError),
    Case(rq.sqltypes.TinyUnsigned(), "Ali", TypeError),
    Case(rq.sqltypes.TinyUnsigned(), 255, None),
    Case(rq.sqltypes.TinyUnsigned(), 255 + 1, OverflowError),
    # --- FloatType ---
    Case(rq.sqltypes.Float(), 20.0, None),
    Case(rq.sqltypes.Float(), 20, None),
    Case(rq.sqltypes.Float(), 4.5, None),
    Case(rq.sqltypes.Float(), "Ali", TypeError),
    # --- DoubleType ---
    Case(rq.sqltypes.Double(), 20.0, None),
    Case(rq.sqltypes.Double(), 20, None),
    Case(rq.sqltypes.Double(), 4.5, None),
    Case(rq.sqltypes.Double(), "Ali", TypeError),
    # --- TextType ---
    Case(rq.sqltypes.Text(), "A", None),
    Case(rq.sqltypes.Text(), "A" * 10000, None),
    Case(rq.sqltypes.Text(), b"B", TypeError),
    Case(rq.sqltypes.Text(), 1, TypeError),
    Case(rq.sqltypes.Text(), 1.4, TypeError),
    # --- StringType ---
    Case(rq.sqltypes.String(), "A", None),
    Case(rq.sqltypes.String(), "A" * 10000, None),
    Case(rq.sqltypes.String(), b"B", TypeError),
    Case(rq.sqltypes.String(), 1, TypeError),
    Case(rq.sqltypes.String(), 1.4, TypeError),
    # --- CharType ---
    Case(rq.sqltypes.Char(), "A", None),
    Case(rq.sqltypes.Char(), "A" * 10000, None),
    Case(rq.sqltypes.Char(), b"B", TypeError),
    Case(rq.sqltypes.Char(), 1, TypeError),
    Case(rq.sqltypes.Char(), 1.4, TypeError),
    # --- ArrayType ---
    Case(rq.sqltypes.Array(rq.sqltypes.Integer()), [1, 2], None),
    Case(rq.sqltypes.Array(rq.sqltypes.Integer()), [1, 2], None),
    Case(rq.sqltypes.Array(rq.sqltypes.Integer()), (1, 2, 5), None),
    Case(rq.sqltypes.Array(rq.sqltypes.Float()), [1.3, 2.7], None),
    Case(rq.sqltypes.Array(rq.sqltypes.Integer()), [1, "A"], TypeError),
    Case(
        rq.sqltypes.Array(rq.sqltypes.Integer()),
        [1, 2, 3, 4, 5, 6, 7, 8, "A", 10],
        TypeError,
    ),
    Case(rq.sqltypes.Array(rq.sqltypes.Text()), ["B", "A"], TypeError),
    # --- VectorType ---
    Case(rq.sqltypes.Vector(6), [1.4, 2.1], None),
    Case(rq.sqltypes.Vector(5), [1, 2], None),
    Case(rq.sqltypes.Vector(4), (1, 2, 5.3), None),
    Case(rq.sqltypes.Vector(3), [1.3, 2.7], None),
    Case(rq.sqltypes.Vector(2), [1, "A"], TypeError),
    Case(rq.sqltypes.Vector(1), [1, 2, 3, 4, 5, 6, 7, 8, "A", 10], TypeError),
    Case(rq.sqltypes.Vector(), ["B", "A"], TypeError),
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
