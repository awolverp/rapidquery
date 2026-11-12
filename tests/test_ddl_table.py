import pytest

import rapidquery as rq
from rapidquery.exc import DialectError


class TestConstraints:
    def test_unique(self):
        constraint = rq.ddl.UniqueConstraint("email", name="users_email_key")
        assert constraint.name == "users_email_key"
        assert list(constraint.columns) == ["email"]
        assert repr(constraint) == 'UniqueConstraint("email", name="users_email_key")'

        with pytest.raises(ValueError, match="at least one column"):
            _ = rq.ddl.UniqueConstraint()

    def test_primary_key(self):
        constraint = rq.ddl.PrimaryKeyConstraint("id", "tenant")
        assert constraint.name is None
        assert list(constraint.columns) == ["id", "tenant"]
        assert repr(constraint) == 'PrimaryKeyConstraint("id", "tenant")'

    def test_foreign_key(self):
        constraint = rq.ddl.ForeignKeyConstraint(
            ["org_id"],
            "orgs",
            ["id"],
            name="users_org_fk",
            on_delete="cascade",
            on_update="SET NULL",
        )
        assert constraint.name == "users_org_fk"
        assert list(constraint.columns) == ["org_id"]
        assert constraint.references.to_list() == ["orgs"]
        assert list(constraint.ref_columns) == ["id"]
        assert constraint.on_delete == "CASCADE"
        assert constraint.on_update == "SET NULL"
        assert (
            repr(constraint)
            == 'ForeignKeyConstraint(["org_id"], Ident("orgs"), ["id"], name="users_org_fk", on_delete="CASCADE", on_update="SET NULL")'
        )

        with pytest.raises(ValueError, match='invalid value: "BOOM"'):
            _ = rq.ddl.ForeignKeyConstraint(["org_id"], "orgs", ["x"], on_delete="BOOM")

    def test_check(self):
        constraint = rq.ddl.CheckConstraint(rq.Ident("active"), name="users_active_check")
        assert constraint.name == "users_active_check"
        assert repr(constraint.expression) == 'Ident("active")'
        assert repr(constraint) == 'CheckConstraint(Ident("active"), name="users_active_check")'


class TestCreateTable:
    def _id_column(self):
        return rq.ddl.Column("id", "INTEGER", nullable=False)

    def _table(self, *columns, **kwargs):
        return rq.ddl.CreateTable("users", self._id_column(), *columns, **kwargs)

    def test_new(self):
        stmt = self._table()
        assert stmt.name.to_list() == ["users"]
        assert len(stmt.columns) == 1
        assert stmt.constraints == ()
        assert stmt.if_not_exists is False
        assert stmt.temporary is False
        assert stmt.on_commit is None
        assert stmt.external is False
        assert stmt.comment is None
        assert stmt.location is None
        assert stmt.extra is None
        assert repr(stmt) == 'CreateTable(Ident("users"), [Column("id", "INTEGER", nullable=False)])'

        with pytest.raises(ValueError, match="at least one Column"):
            _ = rq.ddl.CreateTable("users")

    def test_base(self):
        assert self._table().to_sql(rq.dialect.PostgreSql) == 'CREATE TABLE "users" ("id" INTEGER NOT NULL)'

    def test_columns_and_name(self):
        stmt = rq.ddl.CreateTable(
            "public.users",
            self._id_column(),
            rq.ddl.Column("email", "VARCHAR", nullable=True),
        )
        assert (
            stmt.to_sql(rq.dialect.PostgreSql)
            == 'CREATE TABLE "public"."users" ("id" INTEGER NOT NULL, "email" VARCHAR NULL)'
        )

    def test_if_not_exists(self):
        stmt = self._table(if_not_exists=True)
        assert stmt.to_sql(rq.dialect.PostgreSql) == 'CREATE TABLE IF NOT EXISTS "users" ("id" INTEGER NOT NULL)'
        assert stmt.to_sql(rq.dialect.MsSql) == "CREATE TABLE [users] ([id] INTEGER NOT NULL)"

    def test_temporary(self):
        stmt = self._table(temporary=True)
        assert stmt.to_sql(rq.dialect.PostgreSql) == 'CREATE TEMPORARY TABLE "users" ("id" INTEGER NOT NULL)'
        assert stmt.to_sql(rq.dialect.Teradata) == 'CREATE VOLATILE TABLE "users" ("id" INTEGER NOT NULL)'
        assert stmt.to_sql(rq.dialect.Oracle) == 'CREATE PRIVATE TEMPORARY TABLE "users" ("id" INTEGER NOT NULL)'

        with pytest.raises(DialectError, match="temporary table"):
            _ = stmt.to_sql(rq.dialect.MsSql)

    def test_on_commit(self):
        preserve = self._table(temporary=True, on_commit="PRESERVE ROWS")
        delete = self._table(temporary=True, on_commit="delete rows")
        drop = self._table(temporary=True, on_commit="DROP")
        assert preserve.on_commit == "PRESERVE ROWS"
        assert (
            preserve.to_sql(rq.dialect.PostgreSql)
            == 'CREATE TEMPORARY TABLE "users" ("id" INTEGER NOT NULL) ON COMMIT PRESERVE ROWS'
        )
        assert (
            delete.to_sql(rq.dialect.PostgreSql)
            == 'CREATE TEMPORARY TABLE "users" ("id" INTEGER NOT NULL) ON COMMIT DELETE ROWS'
        )
        assert (
            drop.to_sql(rq.dialect.PostgreSql)
            == 'CREATE TEMPORARY TABLE "users" ("id" INTEGER NOT NULL) ON COMMIT DROP'
        )

        with pytest.raises(ValueError, match="requires a temporary table"):
            _ = self._table(on_commit="DROP")

        with pytest.raises(ValueError, match="received KEEP"):
            _ = self._table(temporary=True, on_commit="KEEP")

        with pytest.raises(DialectError, match="ON COMMIT"):
            _ = preserve.to_sql(rq.dialect.MySql)

    def test_external_comment_location_extra(self):
        stmt = self._table(external=True, comment="hi", location="s3://bucket/users", extra="TBLPROPERTIES ('a'='b')")
        assert (
            stmt.to_sql(rq.dialect.Hive)
            == "CREATE EXTERNAL TABLE `users` (`id` INTEGER NOT NULL) COMMENT 'hi' LOCATION 's3://bucket/users' TBLPROPERTIES ('a'='b')"
        )

        with pytest.raises(DialectError, match="external table"):
            _ = self._table(external=True).to_sql(rq.dialect.PostgreSql)

        commented = self._table(comment="hi")
        assert commented.to_sql(rq.dialect.PostgreSql) == 'CREATE TABLE "users" ("id" INTEGER NOT NULL)'
        assert commented.to_sql(rq.dialect.MySql) == "CREATE TABLE `users` (`id` INTEGER NOT NULL) COMMENT 'hi'"
        assert (
            self._table(comment="hi").to_sql(rq.dialect.Snowflake)
            == 'CREATE TABLE "users" ("id" INTEGER NOT NULL) COMMENT = \'hi\''
        )

        with pytest.raises(DialectError, match="LOCATION"):
            _ = self._table(location="s3://bucket/users").to_sql(rq.dialect.PostgreSql)

    def test_temporary_external(self):
        stmt = self._table(temporary=True, external=True)
        assert stmt.to_sql(rq.dialect.Hive) == "CREATE TEMPORARY EXTERNAL TABLE `users` (`id` INTEGER NOT NULL)"

    def test_constraints(self):
        stmt = self._table(
            rq.ddl.PrimaryKeyConstraint("id", name="users_pk"),
            rq.ddl.UniqueConstraint("email"),
            rq.ddl.ForeignKeyConstraint(["org_id"], "orgs", ["id"], on_delete="CASCADE"),
            rq.ddl.CheckConstraint(rq.Ident("active")),
        )
        assert stmt.constraints[0].name == "users_pk"
        assert (
            stmt.to_sql(rq.dialect.PostgreSql) == 'CREATE TABLE "users" ("id" INTEGER NOT NULL, '
            'CONSTRAINT "users_pk" PRIMARY KEY ("id"), '
            'UNIQUE ("email"), '
            'FOREIGN KEY ("org_id") REFERENCES "orgs" ("id") ON DELETE CASCADE, '
            'CHECK ("active"))'
        )

        bare = rq.dialect.Dialect()
        with pytest.raises(DialectError, match="PRIMARY KEY"):
            _ = stmt.to_sql(bare)

        plain_key = self._table(rq.ddl.ForeignKeyConstraint(["org_id"], "orgs", ["id"]))

        assert (
            plain_key.to_sql(rq.dialect.DuckDb)
            == 'CREATE TABLE "users" ("id" INTEGER NOT NULL, FOREIGN KEY ("org_id") REFERENCES "orgs" ("id"))'
        )
        foreign_key = self._table(rq.ddl.ForeignKeyConstraint(["org_id"], "orgs", ["id"], on_update="RESTRICT"))
        with pytest.raises(DialectError, match="ON UPDATE"):
            _ = foreign_key.to_sql(rq.dialect.DuckDb)

        check = self._table(rq.ddl.CheckConstraint(rq.Ident("active")))
        with pytest.raises(DialectError, match="CHECK"):
            _ = check.to_sql(rq.dialect.Redshift)

        with pytest.raises(TypeError):
            _ = self._table("id")


def _column_sql(column: rq.ddl.Column, dialect: rq.dialect.Dialect = rq.dialect.PostgreSql) -> str:
    return rq.ddl.CreateTable("users", column).to_sql(dialect)


class TestColumn:
    def test_new(self):
        col = rq.ddl.Column("id", "integer")
        assert col.name == "id"
        assert col.data_type == "INTEGER"
        assert col.primary_key is False
        assert col.unique is False
        assert col.nullable is True
        assert col.default is None
        assert col.check is None
        assert col.comment is None
        assert col.auto_increment is False
        assert col.generated is None
        assert col.generated_type is None
        assert col.extra is None
        assert repr(col) == 'Column("id", "INTEGER")'

        described = rq.ddl.Column("email", "varchar(16)", unique=True, comment="hello", extra="COLLATE nocase")
        assert described.data_type == "varchar(16)"
        assert described.unique is True
        assert described.comment == "hello"
        assert described.extra == "COLLATE nocase"
        assert repr(described) == 'Column("email", "varchar(16)", unique=True, comment="hello", extra="COLLATE nocase")'

        pk = rq.ddl.Column("id", "integer", True, True)
        assert pk.primary_key is True
        assert pk.unique is False
        assert pk.nullable is False
        assert repr(pk) == 'Column("id", "INTEGER", primary_key=True, nullable=False)'

        nullable_pk = rq.ddl.Column("id", "integer", primary_key=True, nullable=True)
        assert nullable_pk.nullable is True
        assert "nullable=" not in repr(nullable_pk)

        auto = rq.ddl.Column("id", "integer", auto_increment=True)
        assert auto.nullable is False
        assert repr(auto) == 'Column("id", "INTEGER", nullable=False, auto_increment=True)'

        for attr in ("name", "data_type", "primary_key", "nullable", "comment", "extra"):
            with pytest.raises(AttributeError):
                setattr(col, attr, "x")

        with pytest.raises(TypeError):
            _ = rq.ddl.Column(name="id", data_type="integer")  # type: ignore[call-arg]
        with pytest.raises(TypeError):
            _ = rq.ddl.Column("c", "int", False, False, None, 0, 0, None, False, None, None, "x")  # type: ignore[misc]
        with pytest.raises(TypeError):
            _ = rq.ddl.Column("c", "integer", primary_key="yes")  # type: ignore[arg-type]

    @pytest.mark.parametrize(
        ("kwargs", "expected"),
        [
            ({}, True),
            ({"primary_key": True}, False),
            ({"auto_increment": True}, False),
            ({"primary_key": True, "auto_increment": True}, False),
            ({"nullable": False}, False),
            ({"nullable": True}, True),
            ({"nullable": True, "primary_key": True}, True),
            ({"nullable": True, "auto_increment": True}, True),
        ],
    )
    def test_nullable(self, kwargs, expected):
        assert rq.ddl.Column("c", "integer", **kwargs).nullable is expected

    @pytest.mark.parametrize(
        ("given", "canonical"),
        [
            ("VIRTUAL", "VIRTUAL"),
            ("virtual", "VIRTUAL"),
            ("Persisted", "PERSISTED"),
            ("stored", "STORED"),
        ],
    )
    def test_generated_type(self, given, canonical):
        col = rq.ddl.Column("c", "integer", generated=rq.Ident("base"), generated_type=given)
        assert repr(col.generated) == 'Ident("base")'
        assert col.generated_type == canonical
        assert f'generated_type="{canonical}"' in repr(col)

    def test_generated(self):
        bare = rq.ddl.Column("c", "integer", generated=rq.Ident("base"))
        assert bare.generated_type is None
        assert repr(bare) == 'Column("c", "INTEGER", generated=Ident("base"))'

        dropped = rq.ddl.Column("c", "integer", generated_type="STORED")
        assert dropped.generated is None
        assert dropped.generated_type is None

        kept = rq.ddl.Column("c", "integer", generated=None, generated_type="stored")
        assert repr(kept.generated) == "Value(None, BooleanSerializer())"
        assert kept.generated_type == "STORED"

        with pytest.raises(ValueError, match="received nonsense"):
            _ = rq.ddl.Column("c", "integer", generated_type="nonsense")
        with pytest.raises(ValueError, match="received virtual "):
            _ = rq.ddl.Column("c", "integer", generated=rq.Ident("base"), generated_type="virtual ")

    def test_expressions(self):
        col = rq.ddl.Column("c", "integer", default=3, check=rq.Ident("active"))
        assert repr(col.default) == "Value(3, IntegerSerializer())"
        assert repr(col.check) == 'Ident("active")'
        assert repr(col) == 'Column("c", "INTEGER", default=Value(3, IntegerSerializer()), check=Ident("active"))'

        omitted = rq.ddl.Column("c", "integer")
        explicit = rq.ddl.Column("c", "integer", default=None, check=None, generated=None)
        assert omitted.default is None
        assert omitted.check is None
        assert omitted.generated is None
        assert repr(explicit.default) == "Value(None, BooleanSerializer())"
        assert "default=" not in repr(omitted)
        assert "default=" in repr(explicit)
        assert "check=" in repr(explicit)
        assert "generated=" in repr(explicit)

    def test_to_expr(self):
        plain = rq.ddl.Column("my_col", "integer")
        loaded = rq.ddl.Column("my_col", "integer", primary_key=True, default=1, comment="x")
        assert repr(plain.to_expr()) == repr(plain.__expr__) == repr(plain.__ident__) == 'Ident("my_col")'
        assert repr(loaded.to_expr()) == repr(plain.to_expr())

    def test_sql(self):
        assert _column_sql(rq.ddl.Column("c", "integer")) == 'CREATE TABLE "users" ("c" INTEGER NULL)'
        assert (
            _column_sql(rq.ddl.Column("c", "integer", nullable=False)) == 'CREATE TABLE "users" ("c" INTEGER NOT NULL)'
        )
        assert (
            _column_sql(rq.ddl.Column("id", "integer", primary_key=True, unique=True))
            == 'CREATE TABLE "users" ("id" INTEGER PRIMARY KEY NOT NULL)'
        )
        assert (
            _column_sql(rq.ddl.Column("email", "varchar", unique=True))
            == 'CREATE TABLE "users" ("email" VARCHAR UNIQUE NULL)'
        )
        assert (
            _column_sql(rq.ddl.Column("c", "integer", default=7)) == 'CREATE TABLE "users" ("c" INTEGER NULL DEFAULT 7)'
        )
        assert (
            _column_sql(rq.ddl.Column("c", "text", default="abc"))
            == """CREATE TABLE "users" ("c" TEXT NULL DEFAULT E'abc')"""
        )
        assert (
            _column_sql(rq.ddl.Column("c", "integer", check=rq.Ident("active")))
            == 'CREATE TABLE "users" ("c" INTEGER NULL CHECK ("active"))'
        )
        assert (
            _column_sql(rq.ddl.Column("c", "integer", default=None))
            == 'CREATE TABLE "users" ("c" INTEGER NULL DEFAULT NULL)'
        )
        assert (
            _column_sql(rq.ddl.Column("c", "integer", check=None))
            == 'CREATE TABLE "users" ("c" INTEGER NULL CHECK (NULL))'
        )
        assert (
            _column_sql(rq.ddl.Column("c", "integer", generated=None))
            == 'CREATE TABLE "users" ("c" INTEGER GENERATED ALWAYS AS (NULL) NULL)'
        )
        assert (
            _column_sql(rq.ddl.Column("id", "integer", primary_key=True, auto_increment=True))
            == 'CREATE TABLE "users" ("id" INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY NOT NULL)'
        )
        assert _column_sql(rq.ddl.Column("c", "text", comment="hi")) == 'CREATE TABLE "users" ("c" TEXT NULL)'

    @pytest.mark.parametrize("gen_type", [None, "VIRTUAL", "PERSISTED", "STORED"])
    def test_generated_sql(self, gen_type):
        col = rq.ddl.Column("c", "integer", generated=rq.Ident("base"), generated_type=gen_type)
        suffix = f" {gen_type}" if gen_type else ""
        assert _column_sql(col) == f'CREATE TABLE "users" ("c" INTEGER GENERATED ALWAYS AS ("base"){suffix} NULL)'
        with pytest.raises(DialectError, match="Column.generated"):
            _ = _column_sql(col, rq.dialect.Sqlite)

    def test_mysql_and_unsupported(self):
        assert (
            _column_sql(rq.ddl.Column("c", "text", comment="hi"), rq.dialect.MySql)
            == "CREATE TABLE `users` (`c` TEXT NULL COMMENT 'hi')"
        )
        assert (
            _column_sql(rq.ddl.Column("c", "text", comment="it's"), rq.dialect.MySql)
            == "CREATE TABLE `users` (`c` TEXT NULL COMMENT 'it\\'s')"
        )
        assert (
            _column_sql(rq.ddl.Column("id", "integer", primary_key=True, auto_increment=True), rq.dialect.MySql)
            == "CREATE TABLE `users` (`id` INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL)"
        )

        full = rq.ddl.Column(
            "c",
            "integer",
            primary_key=True,
            default=7,
            check=rq.Ident("c"),
            comment="my comment",
            auto_increment=True,
            generated=rq.Ident("base"),
            generated_type="stored",
            extra="ZZZ_EXTRA",
        )
        assert _column_sql(full, rq.dialect.MySql) == (
            "CREATE TABLE `users` (`c` INTEGER GENERATED ALWAYS AS (`base`) STORED "
            "PRIMARY KEY AUTO_INCREMENT NOT NULL DEFAULT 7 CHECK (`c`) COMMENT 'my comment' ZZZ_EXTRA)"
        )

        with pytest.raises(DialectError, match="Column.auto_increment"):
            _ = _column_sql(rq.ddl.Column("id", "integer", auto_increment=True), rq.dialect.DuckDb)
