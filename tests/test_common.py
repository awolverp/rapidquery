import pytest

import rapidquery as rq


class TestColumn:
    def test_init(self):
        rq.Column("id", rq.sqltypes.Integer())

        with pytest.raises(TypeError):
            rq.Column("id", rq.sqltypes.Integer)  # type: ignore

        with pytest.raises(TypeError):
            rq.Column("id", "INTEGER")  # type: ignore

        rq.Column("id", rq.sqltypes.String(), default="")
        rq.Column("id", rq.sqltypes.String(), default=None)
        rq.Column("id", rq.sqltypes.String(), default=rq.Expr.val(232))
        rq.Column("id", rq.sqltypes.String(), default=None, generated=232)

        with pytest.raises(TypeError):
            rq.Column("id", rq.sqltypes.String(), default=3243)

        col = rq.Column(
            "id",
            rq.sqltypes.Float(),
            primary_key=True,
            auto_increment=True,
            nullable=False,
            extra="HELLO",
            comment="WOW",
            default=4.3,
            generated=98,
            stored_generated=True,
        )
        assert col.name == "id"
        assert type(col.type) is rq.sqltypes.Float
        assert col.primary_key
        assert col.auto_increment
        assert col.nullable is False
        assert col.extra == "HELLO"
        assert col.comment == "WOW"
        assert type(col.default) is rq.Expr
        assert type(col.generated) is rq.Expr
        assert col.stored_generated

    def test_adapt(self):
        col = rq.Column("name", rq.sqltypes.String())
        col.adapt("Ali")
        col.adapt(None)

        with pytest.raises(TypeError):
            col.adapt(4)  # type: ignore

    def test_column_ref_property(self):
        col = rq.Column("name", rq.sqltypes.String())
        rq.AlterTableDropColumnOption(col)


class TestColumnRef:
    def test_new(self):
        ref_1 = rq.ColumnRef("id")
        assert ref_1.name == "id"
        assert ref_1.table is None
        assert ref_1.schema is None

        ref_2 = rq.ColumnRef("id", "characters", "public")
        assert ref_2.name == "id"
        assert ref_2.table == "characters"
        assert ref_2.schema == "public"

        ref_3 = rq.ColumnRef.parse("public.characters.id")
        assert ref_3.name == "id"
        assert ref_3.table == "characters"
        assert ref_3.schema == "public"

        assert ref_2 == ref_3
        assert ref_1 != ref_2

        asterisk_1 = rq.ColumnRef("*")
        assert asterisk_1.name == "*"
        assert asterisk_1.table is None
        assert asterisk_1.schema is None

        asterisk_2 = rq.ColumnRef("*", "characters")
        assert asterisk_2.name == "*"
        assert asterisk_2.table == "characters"
        assert asterisk_2.schema is None

        asterisk_3 = rq.ColumnRef.parse("characters.*")
        assert asterisk_3.name == "*"
        assert asterisk_3.table == "characters"
        assert asterisk_3.schema is None

        assert asterisk_2 == asterisk_3
        assert asterisk_1 != asterisk_2

    def test_copy_with(self):
        ref_1 = rq.ColumnRef("id")
        assert ref_1.name == "id"
        assert ref_1.table is None
        assert ref_1.schema is None

        ref_2 = ref_1.copy_with(table="characters", schema="public")
        assert ref_2.name == "id"
        assert ref_2.table == "characters"
        assert ref_2.schema == "public"

        ref_3 = ref_2.copy_with(name="*")
        assert ref_3.name == "*"
        assert ref_3.table == "characters"
        assert ref_3.schema == "public"

    def test_try_from(self):
        class Prop:
            @property
            def __column_ref__(self):
                return "characters.id"

        class Var:
            __column_ref__ = "characters.id"

        rq.Expr.col("characters.id")
        rq.Expr.col(rq.ColumnRef("id", "characters"))
        rq.Expr.col(Prop())
        rq.Expr.col(Var)


class SelectStatementChild(rq.SelectStatement):
    pass


class TestExpr:
    def test_new(self):
        rq.Expr(rq.Expr.custom("WOW"))
        rq.Expr(rq.Value(None))
        rq.Expr(rq.ColumnRef("id"))
        rq.Expr(rq.Func("NOW"))
        rq.Expr(rq.SelectStatement().columns("id"))
        rq.Expr(SelectStatementChild().columns("id"))
        rq.Expr(rq.CaseStatement().when(rq.Expr.col("aspect").in_([2, 4]), True).else_(False))

        rq.Expr((rq.Expr.custom("TUPLE"), rq.Expr.custom("TUPLE")))

        class Prop:
            @property
            def __expr__(self):
                return rq.Expr.custom("PROPERTY")

        rq.Expr(Prop())

        class ColumnRefProp:
            @property
            def __column_ref__(self):
                return "characters.id"

        rq.Expr(ColumnRefProp())

        rq.Expr(1)
        rq.Expr("a")
        rq.Expr(3.4)
        rq.Expr(None)
        rq.Expr(["a", "b"])

    def test_val(self):
        rq.Expr.val(1)
        rq.Expr.val(1.4)
        rq.Expr.val("a")
        rq.Expr.val(None)

        with pytest.raises(TypeError):
            rq.Expr.val("a", rq.sqltypes.Integer())  # type: ignore

    def test_exists(self):
        with pytest.raises(TypeError):
            rq.Expr.exists(1)  # type: ignore

        stmt = rq.SelectStatement().columns("id")
        assert "EXISTS" in rq.Expr.exists(stmt)._to_sql("postgres")

        stmt = SelectStatementChild().columns("id")
        assert "EXISTS" in rq.Expr.exists(stmt)._to_sql("postgres")

    def test_all(self):
        with pytest.raises(TypeError):
            rq.Expr.all(1)  # type: ignore

        stmt = rq.SelectStatement().columns("id")
        assert "ALL" in rq.Expr.all(stmt)._to_sql("postgres")

        stmt = SelectStatementChild().columns("id")
        assert "ALL" in rq.Expr.all(stmt)._to_sql("postgres")

    def test_any(self):
        with pytest.raises(TypeError):
            rq.Expr.any(1)  # type: ignore

        stmt = rq.SelectStatement().columns("id")
        assert "ANY" in rq.Expr.any(stmt)._to_sql("postgres")

        stmt = SelectStatementChild().columns("id")
        assert "ANY" in rq.Expr.any(stmt)._to_sql("postgres")

    def test_some(self):
        with pytest.raises(TypeError):
            rq.Expr.some(1)  # type: ignore

        stmt = rq.SelectStatement().columns("id")
        assert "SOME" in rq.Expr.some(stmt)._to_sql("postgres")

        stmt = SelectStatementChild().columns("id")
        assert "SOME" in rq.Expr.some(stmt)._to_sql("postgres")


class TestForeignKey:
    def test_init(self):
        rq.ForeignKey(
            ["font_id"],
            ["id"],
            "fonts",
            "fk_name",
            on_delete="CASCADE",
            on_update="NO ACTION",
        )
        fk = rq.ForeignKey(["font_id"], ["fonts.id"])
        assert fk.to_table.name == "fonts"

        rq.ForeignKey(["font_id", "font_name"], ["fonts.id", "fonts.name"])

        with pytest.raises(ValueError):
            rq.ForeignKey(["id_1", "id_2"], ["t1.id", "t2.id"])

        with pytest.raises(ValueError):
            rq.ForeignKey(["1", "2"], ["1"])

        with pytest.raises(ValueError):
            rq.ForeignKey(["font_id"], ["fonts.id"], on_delete="AAA")  # type: ignore

        with pytest.raises(ValueError):
            rq.ForeignKey(["font_id"], ["fonts.id"], on_update="AAA")  # type: ignore


class TestFunc:
    def test_new(self):
        rq.Func("CUSTOM", 1, 2)


class TestTableName:
    def test_new(self):
        t_1 = rq.TableName("users")
        assert t_1.name == "users"
        assert t_1.schema is None
        assert t_1.database is None
        assert t_1.alias is None

        t_2 = rq.TableName("users", "public", "world")
        assert t_2.name == "users"
        assert t_2.schema == "public"
        assert t_2.database == "world"
        assert t_2.alias is None

        t_3 = rq.TableName.parse("world.public.users")
        assert t_3.name == "users"
        assert t_3.schema == "public"
        assert t_3.database == "world"
        assert t_3.alias is None

        assert t_3 == t_2
        assert t_3 != t_1

    def test_copy_with(self):
        t_1 = rq.TableName("users")
        assert t_1.name == "users"
        assert t_1.schema is None
        assert t_1.database is None
        assert t_1.alias is None

        t_2 = t_1.copy_with(schema="public", database="world")
        assert t_2.name == "users"
        assert t_2.schema == "public"
        assert t_2.database == "world"
        assert t_2.alias is None

        t_3 = t_2.copy_with(database=None, alias="t_3")
        assert t_3.name == "users"
        assert t_3.schema == "public"
        assert t_3.database is None
        assert t_3.alias == "t_3"

    def test_try_from(self):
        class Prop:
            @property
            def __table_name__(self):
                return "world.public.users"

        class Var:
            __table_name__ = "world.public.users"

        rq.DeleteStatement(rq.TableName("users", "public", "world"))
        rq.DeleteStatement("world.public.users")
        rq.DeleteStatement(Prop())
        rq.DeleteStatement(Var)
