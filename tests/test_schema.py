import pytest

import rapidquery as rq

from .mixin import TableNameInitMixin


class TestAlterTable(TableNameInitMixin):
    table_name_instance = rq.AlterTable

    options = [
        rq.AlterTableAddColumnOption(rq.Column("premium", rq.sqltypes.Boolean())),
        rq.AlterTableAddForeignKeyOption(rq.ForeignKey(["id"], ["fonts.id"])),
        rq.AlterTableDropColumnOption("id"),
        rq.AlterTableDropForeignKeyOption("fk_12"),
        rq.AlterTableModifyColumnOption(
            rq.Column("name", rq.sqltypes.Char(64), nullable=True)
        ),
        rq.AlterTableRenameColumnOption("id", "user_id"),
    ]

    def test_init(self):
        rq.AlterTable("users", self.options)
        rq.AlterTable("users", [])

        with pytest.raises(TypeError):
            rq.AlterTable("users", [1])  # type: ignore

        with pytest.raises(TypeError):
            rq.AlterTable("users", ["a"])  # type: ignore

    def test_add_option(self):
        stmt = rq.AlterTable("users", [])

        for op in self.options:
            stmt.add_option(op)

    def test_properties(self):
        stmt = rq.AlterTable("users", self.options)

        assert stmt.name == rq.TableName("users")
        assert stmt.options == self.options

        stmt.name = "public.fonts"
        assert stmt.name == rq.TableName("fonts", "public")

        stmt.options = []
        assert stmt.options == []


class TestDropIndex(TableNameInitMixin):
    def table_name_instance(self, tb):
        return rq.DropIndex("idx_1", tb)

    def test_init(self):
        stmt = rq.DropIndex("idx_name", "users", if_exists=True)
        assert "IF EXISTS" in stmt.to_sql("postgres")

    def test_properties(self):
        stmt = rq.DropIndex("idx_1", "fonts")

        assert stmt.name == "idx_1"
        assert stmt.table == rq.TableName("fonts")
        assert not stmt.if_exists

        stmt.name = "idx_2"
        assert stmt.name == "idx_2"

        stmt.table = "users"
        assert stmt.table == rq.TableName("users")

        stmt.if_exists = True
        assert stmt.if_exists


class TestDropTable(TableNameInitMixin):
    table_name_instance = rq.DropTable

    def test_init(self):
        rq.DropTable("users", if_exists=True, cascade=True, restrict=True)

    def test_properties(self):
        stmt = rq.DropTable("fonts")

        assert stmt.name == rq.TableName("fonts")
        assert not stmt.if_exists
        assert not stmt.cascade
        assert not stmt.restrict

        stmt.name = "public.users"
        assert stmt.name == rq.TableName("users", "public")

        stmt.if_exists = True
        stmt.cascade = True
        stmt.restrict = True
        assert stmt.if_exists
        assert stmt.cascade
        assert stmt.restrict


class TestIndex(TableNameInitMixin):
    def table_name_instance(self, tb):
        return rq.Index(None, ["id"], tb)

    def test_init(self):
        stmt = rq.Index(
            "idx_glyph_aspect", ["aspect"], "glyph", if_not_exists=True
        ).to_sql("sqlite")
        assert (
            stmt
            == 'CREATE INDEX IF NOT EXISTS "idx_glyph_aspect" ON "glyph" ("aspect")'
        )

        stmt = rq.Index(
            "idx_glyph_aspect", [rq.IndexColumn("aspect", "ASC", 128)], "glyph"
        ).to_sql("mysql")
        assert stmt == "CREATE INDEX `idx_glyph_aspect` ON `glyph` (`aspect` (128) ASC)"

        stmt = rq.Index(
            "idx_font_name_include_language",
            ["name"],
            "fonts",
            include=["language"],
            where=rq.Expr.col("aspect").in_([3, 4]),
        ).to_sql("postgresql")
        assert (
            stmt
            == 'CREATE INDEX "idx_font_name_include_language" ON "fonts" ("name") INCLUDE ("language")'
        )

        stmt = rq.Index(
            "idx_name",
            ["font_id"],
            "fonts",
            primary=False,
            nulls_not_distinct=True,
            unique=True,
            index_type="BTREE",
            where=None,
            include=["language"],
        ).to_sql("postgres")
        assert (
            stmt
            == 'CREATE UNIQUE INDEX "idx_name" ON "fonts" USING BTREE ("font_id") INCLUDE ("language") NULLS NOT DISTINCT'
        )


class TestIndexColumn:
    def test_new(self):
        rq.IndexColumn("name")
        rq.IndexColumn("name", "ASC")
        rq.IndexColumn("name", prefix=12)
        rq.IndexColumn("name", "DESC", prefix=1)
        rq.IndexColumn("name", None, None)

        # lowercase orders
        rq.IndexColumn("name", "asc")  # type: ignore
        rq.IndexColumn("name", "desc")  # type: ignore

        with pytest.raises(Exception):
            rq.IndexColumn("name", "invalid")  # type: ignore

        val = rq.IndexColumn("name", "desc", prefix=1)  # type: ignore
        assert val.name == "name"
        assert val.order == "DESC"
        assert val.prefix == 1


class TestRenameTable(TableNameInitMixin):
    def table_name_instance(self, tb):
        return rq.RenameTable(tb, tb)

    def test_properties(self):
        stmt = rq.RenameTable("t1", "t2")

        assert stmt.from_name == rq.TableName("t1")
        assert stmt.to_name == rq.TableName("t2")

        stmt.from_name = "pub.users"
        stmt.to_name = "arc.users"
        assert stmt.from_name == rq.TableName("users", "pub")
        assert stmt.to_name == rq.TableName("users", "arc")


class TestTable:
    def test_init(self):
        rq.Table("users")
        rq.Table("public.users")
        rq.Table("db.public.users")
        rq.Table(rq.TableName("users"))
        rq.Table(rq.TableName("users", "public"))
        rq.Table(rq.TableName("users", "public", "db"))

        table = rq.Table(
            "pub.users",
            rq.Column(
                "id", rq.sqltypes.Integer(), primary_key=True, auto_increment=True
            ),
            rq.Column(
                "username", rq.sqltypes.Integer(), unique_key=True, nullable=False
            ),
            rq.Column(
                "font_id", rq.sqltypes.Integer(), unique_key=True, nullable=False
            ),
            rq.ForeignKey(["font_id"], ["fonts.id"], on_delete="CASCADE"),
            rq.Index("idx_name", ["id", "font_id"]),
            rq.Expr.col("username").like("A%"),
            if_not_exists=True,
            extra="WOW",
        )
        assert table.name == rq.TableName("users", "pub")
        assert len(table.columns) == 3
        assert len(table.foreign_keys) == 1
        assert len(table.indexes) == 1
        assert len(table.checks) == 1
        assert all(isinstance(x, rq.Column) for x in table.columns)
        assert all(isinstance(x, rq.ForeignKey) for x in table.foreign_keys)
        assert all(isinstance(x, rq.Index) for x in table.indexes)
        assert all(isinstance(x, rq.Expr) for x in table.checks)
        assert table.if_not_exists
        assert not table.temporary
        assert table.extra == "WOW"

        stmt = table.to_sql("postgres")
        assert stmt.count(";") == 1
        assert "IF NOT EXISTS" in stmt
        assert '"id"' in stmt
        assert '"username"' in stmt
        assert '"font_id"' in stmt
        assert "WOW" in stmt

    def test_args(self):
        with pytest.raises(TypeError):
            rq.Table(
                "pub.users",
                rq.Column(
                    "id", rq.sqltypes.Integer(), primary_key=True, auto_increment=True
                ),
                complex(),  # type: ignore
            )

        class Col(rq.Column):
            pass

        class Fk(rq.ForeignKey):
            pass

        class Idx(rq.Index):
            pass

        table = rq.Table(
            "pub.users",
            Col("id", rq.sqltypes.Integer(), primary_key=True, auto_increment=True),
            Fk(["font_id"], ["fonts.id"], on_delete="CASCADE"),
            Idx("idx_name", ["id", "font_id"]),
        )

        assert table.name == rq.TableName("users", "pub")
        assert len(table.columns) == 1
        assert len(table.foreign_keys) == 1
        assert len(table.indexes) == 1
        assert all(isinstance(x, Col) for x in table.columns)
        assert all(isinstance(x, Fk) for x in table.foreign_keys)
        assert all(isinstance(x, Idx) for x in table.indexes)

    def test_multiple_primary(self):
        stmt = rq.Table(
            "pub.users",
            rq.Column("id", rq.sqltypes.Integer(), primary_key=True),
            rq.Column("name", rq.sqltypes.Integer(), primary_key=True),
        )
        assert stmt.to_sql("postgres").count("PRIMARY KEY") == 2


class TestTruncateTable:
    def table_name_instance(self, tb):
        return rq.TruncateTable(tb)

    def test_properties(self):
        stmt = rq.TruncateTable("t1")

        assert stmt.name == rq.TableName("t1")

        stmt.name = "pub.users"
        assert stmt.name == rq.TableName("users", "pub")
