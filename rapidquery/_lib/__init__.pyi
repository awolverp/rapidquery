from __future__ import annotations

from . import sqltypes as sqltypes
from . import schema as schema
from . import query as query
from . import common as common

__all__ = ["delete", "insert", "returning", "update", "window"]

def delete(table: schema.Table | common.TableName | str) -> query.DeleteStatement:
    """Create a new `DeleteStatement`."""
    ...

def insert(table: schema.Table | common.TableName | str) -> query.InsertStatement:
    """Create a new `InsertStatement`."""
    ...

def returning(*args: common.Column | common.ColumnRef | str) -> query.Returning:
    """Create a new `Returning`."""
    ...

def update(table: schema.Table | common.TableName | str) -> query.UpdateStatement:
    """Create a new `PyUpdateStatement`."""
    ...

def window(
    *partition_by: common.Expr | common.Column | common.ColumnRef | str,
) -> query.WindowStatement:
    """Create a new `WindowStatement`."""
    ...
