import pytest

import rapidquery as rq


def _supported(**kwargs: int) -> rq.dialect.Dialect:
    features = rq.dialect.OPT_INDEX_FEATURES_SUPPORTED
    extra = kwargs.pop("features", 0)
    index_type = kwargs.pop("index_type", rq.dialect.OPT_INDEX_TYPE_UNSUPPORTED)
    return rq.dialect.Dialect(index=rq.dialect.IndexConfig(features=features | extra, index_type=index_type))


class TestIndexColumn:
    def test_new(self):
        column = rq.ddl.IndexColumn("id")
        assert column.column.to_list() == ["id"]
        assert column.order == "ASC"
        assert repr(column) == 'IndexColumn(Ident("id"))'

        column = rq.ddl.IndexColumn("id", order="Desc")
        assert column.order == "DESC"
        assert repr(column) == 'IndexColumn(Ident("id"), order="DESC")'

        with pytest.raises(ValueError):
            _ = rq.ddl.IndexColumn("id", order="invalid")


class TestCreateIndex:
    def test_new(self):
        column = rq.ddl.IndexColumn("id")
        stmt = rq.ddl.CreateIndex("idx", "users", [column])
        assert stmt.name == "idx"
        assert stmt.unique is False
        assert stmt.if_not_exists is False
        assert stmt.table_name.to_list() == ["users"]
        assert stmt.columns[0].order == "ASC"
        assert stmt.predicate is None
        assert stmt.include == []
        assert stmt.index_type is None
        assert repr(stmt) == 'CreateIndex("idx", Ident("users"), [IndexColumn(Ident("id"))])'

        stmt = rq.ddl.CreateIndex(
            "idx",
            "datasets",
            [column],
            unique=True,
            if_not_exists=True,
            include=["name"],
            predicate=rq.Ident("active"),
            index_type="btree",
        )
        assert stmt.table_name.to_list() == ["datasets"]
        assert stmt.unique is True
        assert stmt.if_not_exists is True
        assert stmt.include == ["name"]
        assert stmt.predicate.to_list() == ["active"]
        assert stmt.index_type == "btree"
        assert "unique=True" in repr(stmt)
        assert "if_not_exists=True" in repr(stmt)

    def test_invalid_construction(self):
        column = rq.ddl.IndexColumn("id")

        with pytest.raises(ValueError, match="empty"):
            _ = rq.ddl.CreateIndex("", "users", [column])
        with pytest.raises(ValueError, match="column"):
            _ = rq.ddl.CreateIndex("idx", "users", [])
        with pytest.raises(ValueError, match="INCLUDE"):
            _ = rq.ddl.CreateIndex("idx", "users", [column], include=[""])
        with pytest.raises(ValueError, match="index type"):
            _ = rq.ddl.CreateIndex("idx", "users", [column], index_type="")

    def test_unsupported(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")])

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.Dialect())
        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.BigQuery)

    def test_supported(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")])
        assert stmt.to_sql(_supported()) == 'CREATE INDEX "idx" ON "users" ("id")'

    def test_unique(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")], unique=True)

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(_supported())

        rendered = stmt.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_UNIQUE))
        assert rendered == 'CREATE UNIQUE INDEX "idx" ON "users" ("id")'

    def test_if_not_exists(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")], if_not_exists=True)
        assert stmt.to_sql(_supported()) == 'CREATE INDEX "idx" ON "users" ("id")'

        rendered = stmt.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_IF_NOT_EXISTS))
        assert rendered == 'CREATE INDEX IF NOT EXISTS "idx" ON "users" ("id")'

    def test_column_order(self):
        asc = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id", order="ASC")])
        desc = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id", order="DESC")])

        assert asc.to_sql(_supported()) == 'CREATE INDEX "idx" ON "users" ("id")'
        with pytest.raises(RuntimeError, match="supported"):
            _ = desc.to_sql(_supported())

        rendered = asc.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_COLUMN_ORDER))
        assert rendered == 'CREATE INDEX "idx" ON "users" ("id" ASC)'
        rendered = desc.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_COLUMN_ORDER))
        assert rendered == 'CREATE INDEX "idx" ON "users" ("id" DESC)'

        both = rq.ddl.CreateIndex(
            "idx",
            "users",
            [rq.ddl.IndexColumn("id"), rq.ddl.IndexColumn("email", order="DESC")],
        )
        rendered = both.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_COLUMN_ORDER))
        assert rendered == 'CREATE INDEX "idx" ON "users" ("id" ASC, "email" DESC)'

    def test_expression(self):
        column = rq.ddl.IndexColumn(rq.expr.Function("lower", rq.Ident("email")))
        stmt = rq.ddl.CreateIndex("idx", "users", [column])

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(_supported())

        rendered = stmt.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_EXPRESSIONS))
        assert rendered == 'CREATE INDEX "idx" ON "users" (("lower"("email")))'

    def test_predicate(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")], predicate=rq.Ident("active"))

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(_supported())

        rendered = stmt.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_PARTIAL))
        assert rendered == 'CREATE INDEX "idx" ON "users" ("id") WHERE "active"'

    def test_include(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")], include=["name", "email"])

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(_supported())

        rendered = stmt.to_sql(_supported(features=rq.dialect.OPT_INDEX_FEATURES_INCLUDE))
        assert rendered == 'CREATE INDEX "idx" ON "users" ("id") INCLUDE ("name", "email")'

    def test_index_type_position(self):
        stmt = rq.ddl.CreateIndex("idx", "users", [rq.ddl.IndexColumn("id")], index_type="btree")

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(_supported())

        before = stmt.to_sql(_supported(index_type=rq.dialect.OPT_INDEX_TYPE_BEFORE_COLUMNS))
        assert before == 'CREATE INDEX "idx" ON "users" USING btree ("id")'

        after = stmt.to_sql(_supported(index_type=rq.dialect.OPT_INDEX_TYPE_AFTER_COLUMNS))
        assert after == 'CREATE INDEX "idx" ON "users" ("id") USING btree'

    def test_postgresql(self):
        stmt = rq.ddl.CreateIndex(
            "idx",
            "users",
            [rq.ddl.IndexColumn(rq.expr.Function("lower", rq.Ident("email")))],
            unique=True,
            if_not_exists=True,
            include=["name"],
            predicate=rq.Ident("active"),
            index_type="btree",
        )
        assert stmt.to_sql(rq.dialect.PostgreSql) == (
            'CREATE UNIQUE INDEX IF NOT EXISTS "idx" ON "users" USING btree '
            '(("lower"("email")) ASC) INCLUDE ("name") WHERE "active"'
        )

    def test_mysql(self):
        stmt = rq.ddl.CreateIndex(
            "idx",
            "users",
            [rq.ddl.IndexColumn("id", order="DESC")],
            index_type="BTREE",
        )
        assert stmt.to_sql(rq.dialect.MySql) == "CREATE INDEX `idx` ON `users` (`id` DESC) USING BTREE"

    def test_duckdb(self):
        stmt = rq.ddl.CreateIndex("idx", "t", [rq.ddl.IndexColumn("id")], index_type="ART")
        assert stmt.to_sql(rq.dialect.DuckDb) == 'CREATE INDEX "idx" ON "t" ("id") USING ART'

    def test_sqlite(self):
        stmt = rq.ddl.CreateIndex("idx", "t", [rq.ddl.IndexColumn("id")], predicate=rq.Ident("active"))
        assert stmt.to_sql(rq.dialect.Sqlite) == 'CREATE INDEX "idx" ON "t" ("id" ASC) WHERE "active"'

    def test_mssql(self):
        stmt = rq.ddl.CreateIndex(
            "idx",
            "t",
            [rq.ddl.IndexColumn("id")],
            include=["name"],
            predicate=rq.Ident("active"),
        )
        assert stmt.to_sql(rq.dialect.MsSql) == ("CREATE INDEX [idx] ON [t] ([id] ASC) INCLUDE ([name]) WHERE [active]")


def _drop(**kwargs: int) -> rq.dialect.Dialect:
    features = kwargs.pop("features", 0)
    drop_features = rq.dialect.OPT_INDEX_DROP_FEATURES_SUPPORTED | kwargs.pop("drop_features", 0)
    drop_table = kwargs.pop("drop_table", rq.dialect.OPT_INDEX_DROP_TABLE_NAME_ONLY)
    return rq.dialect.Dialect(
        index=rq.dialect.IndexConfig(features=features, drop_features=drop_features, drop_table=drop_table)
    )


class TestDropIndex:
    def test_new(self):
        stmt = rq.ddl.DropIndex("idx")
        assert stmt.name == "idx"
        assert stmt.table_name is None
        assert stmt.if_exists is False
        assert stmt.behavior is None
        assert repr(stmt) == 'DropIndex("idx")'

        stmt = rq.ddl.DropIndex("idx", table_name="users", if_exists=True, behavior="Restrict")
        assert stmt.table_name.to_list() == ["users"]
        assert stmt.if_exists is True
        assert stmt.behavior == "RESTRICT"
        assert 'table_name=Ident("users")' in repr(stmt)
        assert "if_exists=True" in repr(stmt)
        assert 'behavior="RESTRICT"' in repr(stmt)

    def test_invalid_construction(self):
        with pytest.raises(ValueError, match="empty"):
            _ = rq.ddl.DropIndex("")
        with pytest.raises(ValueError):
            _ = rq.ddl.DropIndex("idx", behavior="invalid")

    def test_unsupported(self):
        stmt = rq.ddl.DropIndex("idx")

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.Dialect())
        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.BigQuery)
        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.Databricks)

        created = rq.ddl.CreateIndex("idx", "t", [rq.ddl.IndexColumn("id")])
        assert created.to_sql(rq.dialect.Databricks) == "CREATE INDEX `idx` ON `t` (`id`)"

    def test_if_exists(self):
        stmt = rq.ddl.DropIndex("idx", if_exists=True)
        assert stmt.to_sql(_drop()) == 'DROP INDEX "idx"'

        rendered = stmt.to_sql(_drop(features=rq.dialect.OPT_INDEX_FEATURES_IF_NOT_EXISTS))
        assert rendered == 'DROP INDEX IF EXISTS "idx"'

    def test_table_name(self):
        named = rq.ddl.DropIndex("idx", table_name="users")
        missing = rq.ddl.DropIndex("idx")

        assert missing.to_sql(_drop()) == 'DROP INDEX "idx"'

        on = rq.dialect.OPT_INDEX_DROP_TABLE_ON

        with pytest.raises(ValueError, match="table_name"):
            _ = missing.to_sql(_drop(drop_table=on))

        assert named.to_sql(_drop(drop_table=on)) == 'DROP INDEX "idx" ON "users"'

        qualified = rq.dialect.OPT_INDEX_DROP_TABLE_QUALIFIED
        with pytest.raises(ValueError, match="table_name"):
            _ = missing.to_sql(_drop(drop_table=qualified))

        assert named.to_sql(_drop(drop_table=qualified)) == 'DROP INDEX "users"."idx"'

    def test_behavior(self):
        cascade = rq.ddl.DropIndex("idx", behavior="CASCADE")
        restrict = rq.ddl.DropIndex("idx", behavior="RESTRICT")

        with pytest.raises(RuntimeError, match="supported"):
            _ = cascade.to_sql(_drop())
        with pytest.raises(RuntimeError, match="supported"):
            _ = restrict.to_sql(_drop())

        assert (
            cascade.to_sql(_drop(drop_features=rq.dialect.OPT_INDEX_DROP_FEATURES_CASCADE))
            == 'DROP INDEX "idx" CASCADE'
        )
        assert (
            restrict.to_sql(_drop(drop_features=rq.dialect.OPT_INDEX_DROP_FEATURES_RESTRICT))
            == 'DROP INDEX "idx" RESTRICT'
        )

    def test_postgresql(self):
        stmt = rq.ddl.DropIndex("idx", if_exists=True, behavior="CASCADE")
        assert stmt.to_sql(rq.dialect.PostgreSql) == 'DROP INDEX IF EXISTS "idx" CASCADE'

    def test_mysql(self):
        stmt = rq.ddl.DropIndex("idx", table_name="users", if_exists=True)
        assert stmt.to_sql(rq.dialect.MySql) == "DROP INDEX `idx` ON `users`"

    def test_mssql(self):
        stmt = rq.ddl.DropIndex("idx", table_name="t", if_exists=True)
        assert stmt.to_sql(rq.dialect.MsSql) == "DROP INDEX [idx] ON [t]"

    def test_snowflake(self):
        stmt = rq.ddl.DropIndex("idx", table_name="customers", if_exists=True)
        assert stmt.to_sql(rq.dialect.Snowflake) == 'DROP INDEX IF EXISTS "customers"."idx"'

    def test_sqlite(self):
        stmt = rq.ddl.DropIndex("idx", if_exists=True)
        assert stmt.to_sql(rq.dialect.Sqlite) == 'DROP INDEX IF EXISTS "idx"'
