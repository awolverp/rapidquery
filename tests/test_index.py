import pytest

import rapidquery as rq


def test_index_column():
    rq.IndexColumn("name")
    rq.IndexColumn("name", "ASC")
    rq.IndexColumn("name", prefix=12)
    rq.IndexColumn("name", "DESC", prefix=1)
    rq.IndexColumn("name", None, None)

    # lowercase orders
    rq.IndexColumn("name", "asc")  # type: ignore
    rq.IndexColumn("name", "desc")  # type: ignore

    with pytest.raises(Exception):
        rq.IndexColumn("name", "invalid")

    with pytest.raises(Exception):
        rq.IndexColumn("name", "aSc")

    val = rq.IndexColumn("name", "desc", prefix=1)  # type: ignore
    assert val.name == "name"
    assert val.order == "DESC"
    assert val.prefix == 1
