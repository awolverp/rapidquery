"""
RapidQuery is a powerful SQL query builder library designed for Python,
combining the simplicity of Python with the raw speed of **Rust**.
Build complex SQL queries effortlessly and efficiently, with a library that
prioritizes both performance and ease of use.
"""

from ._lib import common as common
from ._lib import mysql as mysql
from ._lib import postgres as postgres
from ._lib import query as query
from ._lib import schema as schema
from ._lib import sqlite as sqlite
from ._lib import sqltypes as sqltypes

# Export .common
Column = common.Column
ColumnRef = common.ColumnRef
Expr = common.Expr
ForeignKey = common.ForeignKey
Func = common.Func
TableName = common.TableName
Value = common.Value
all = common.all
any = common.any
not_ = common.not_

# Export .query
CaseStatement = query.CaseStatement
DeleteStatement = query.DeleteStatement
Frame = query.Frame
InsertStatement = query.InsertStatement
OnConflict = query.OnConflict
Ordering = query.Ordering
QueryStatement = query.QueryStatement
Returning = query.Returning
SelectLabel = query.SelectLabel
SelectStatement = query.SelectStatement
UpdateStatement = query.UpdateStatement
WindowStatement = query.WindowStatement
WithClause = query.WithClause
WithQuery = query.WithQuery

# Export .schema
AlterTable = schema.AlterTable
AlterTableAddColumnOption = schema.AlterTableAddColumnOption
AlterTableAddForeignKeyOption = schema.AlterTableAddForeignKeyOption
AlterTableBaseOption = schema.AlterTableBaseOption
AlterTableDropColumnOption = schema.AlterTableDropColumnOption
AlterTableDropForeignKeyOption = schema.AlterTableDropForeignKeyOption
AlterTableModifyColumnOption = schema.AlterTableModifyColumnOption
AlterTableRenameColumnOption = schema.AlterTableRenameColumnOption
DropIndex = schema.DropIndex
DropTable = schema.DropTable
Index = schema.Index
IndexColumn = schema.IndexColumn
RenameTable = schema.RenameTable
SchemaStatement = schema.SchemaStatement
Table = schema.Table
TruncateTable = schema.TruncateTable
