from collections import namedtuple
from datetime import datetime, timezone
import decimal
import pytest
import enum
import uuid

import rapidquery as rq


NamedCase = namedtuple("NamedCase", ["data", "attribute", "type", "error"])


class Enum(enum.Enum):
    FIELD = "field"

class StrEnum(enum.StrEnum):
    FIELD = "field"

class IntEnum(enum.IntEnum):
    FIELD = 1


TEST_CASES = [
    NamedCase(None, "is_null", None, False),
    NamedCase(None, "is_null", rq.FloatType(), False),
    NamedCase(True, "is_boolean", None, False),
    NamedCase(False, "is_boolean", rq.FloatType(), True),
    NamedCase(False, "is_boolean", rq.BooleanType(), False),
    NamedCase(1, "is_integer", None, False),
    NamedCase(-4, "is_integer", rq.IntegerType(), False),
    NamedCase(3e-3, "is_integer", rq.IntegerType(), True),
    NamedCase(3, "is_integer", rq.UnsignedType(), False),
    NamedCase(-1, "is_integer", rq.UnsignedType(), True),
    NamedCase(5e-3, "is_float", None, False),
    NamedCase(5e-3, "is_float", rq.DoubleType(), False),
    NamedCase(-4.5, "is_float", rq.FloatType(), False),
    NamedCase("data", "is_string", None, False),
    NamedCase("data", "is_string", rq.StringType(), False),
    NamedCase("data", "is_string", rq.IntervalType(), False),
    NamedCase("data", "is_string", rq.InetType(), False),
    NamedCase("data", "is_string", rq.MacAddressType(), False),
    NamedCase("data", "is_string", rq.CidrType(), False),
    NamedCase("data", "is_string", rq.CharType(), False),
    NamedCase(b"data", "is_bytes", None, False),
    NamedCase(b"data", "is_bytes", rq.BitType(), False),
    NamedCase({"name": "rq"}, "is_json", None, False),
    NamedCase([], "is_json", None, False),
    NamedCase(6, "is_json", rq.JsonBinaryType(), False),
    NamedCase("data", "is_json", rq.JsonBinaryType(), False),
    NamedCase(4.5, "is_json", rq.JsonBinaryType(), False),
    NamedCase({1: "rq"}, "is_json", None, True),
    NamedCase({1: "rq"}, "is_json", rq.JsonBinaryType(), True),
    NamedCase([1, 2, 3], "is_array", rq.ArrayType(rq.TinyIntegerType()), False),
    NamedCase([3, "b"], "is_array", rq.ArrayType(rq.TinyIntegerType()), True),
    NamedCase(datetime.now(), "is_datetime", None, False),
    NamedCase(datetime.now(tz=timezone.utc), "is_datetime", None, False),
    NamedCase(datetime.now(), "is_datetime", rq.DateTimeType(), False),
    NamedCase(datetime.now(tz=timezone.utc), "is_datetime", rq.TimestampType(), False),
    NamedCase(datetime.now(tz=timezone.utc), "is_datetime", rq.TimestampWithTimeZoneType(), False),
    NamedCase(datetime.now(), "is_datetime", rq.TimestampWithTimeZoneType(), False),
    NamedCase(uuid.uuid4(), "is_uuid", None, False),
    NamedCase(uuid.uuid4(), "is_uuid", rq.UuidType(), False),
    NamedCase(uuid.uuid4().hex, "is_uuid", rq.UuidType(), True),
    NamedCase(decimal.Decimal("1.2"), "is_decimal", None, False),
    NamedCase(decimal.Decimal("1.2"), "is_decimal", rq.DecimalType(), False),
    NamedCase(decimal.Decimal("1.2"), "is_decimal", rq.FloatType(), True),
    NamedCase(1.2, "is_decimal", rq.DecimalType(), False),
    NamedCase("1.2", "is_decimal", rq.DecimalType(), False),
    NamedCase([1.3, 2.1, 3], "is_vector", rq.VectorType(), False),
    NamedCase([3, "b"], "is_vector", rq.VectorType(), True),
    NamedCase("data", "is_string", rq.EnumType("a", ["a"]), False),
    NamedCase(Enum.FIELD, "is_string", rq.EnumType("a", ["a"]), False),
    NamedCase(StrEnum.FIELD, "is_string", rq.EnumType("a", ["a"]), False),
    NamedCase(IntEnum.FIELD, "is_string", rq.EnumType("a", ["a"]), True),
]


@pytest.mark.parametrize("case", TEST_CASES)
def test_adaptedvalue(case: NamedCase):
    try:
        val = rq.AdaptedValue(case.data, case.type)
    except (ValueError, TypeError, OverflowError):
        if case.error:
            return

        raise

    assert getattr(val, case.attribute)

    rq.Expr(val)  # Force AdaptedValue to adapt
