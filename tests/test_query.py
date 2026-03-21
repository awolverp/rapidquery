import pytest

import rapidquery as rq


class TestCaseStatement:
    def test_init(self):
        rq.CaseStatement()

    def test_when_else(self):
        stmt = (
            rq.CaseStatement()
            .when(rq.Expr.col("id") == 1, 1)
            .when(rq.Expr.col("name") == "ali", 1)
        )
        sql = rq.SelectStatement(rq.SelectLabel(stmt, "test")).to_sql("postgres")
        assert '"id"' in sql
        assert '"name"' in sql

        with pytest.raises(TypeError):
            rq.CaseStatement().when("Ali", 1)  # type: ignore

        rq.CaseStatement().when(rq.Expr("Ali"), 1).else_(1)


class TestDeleteStatement:
    def test_init(self):
        stmt = rq.DeleteStatement("public.users")
        assert '"public"."users"' in stmt.to_sql("postgres")

    def test_clear_order_by(self):
        stmt = rq.DeleteStatement("users").order_by(rq.Ordering("id"))
        assert "ORDER BY" in stmt.to_sql("mysql")

        stmt = stmt.clear_order_by()
        assert "ORDER BY" not in stmt.to_sql("mysql")

    def test_clear_where(self):
        stmt = rq.DeleteStatement("users").where(rq.Expr.val(True))
        assert "WHERE" in stmt.to_sql("mysql")

        stmt = stmt.clear_where()
        assert "WHERE" not in stmt.to_sql("mysql")

    def test_from_table(self):
        stmt = rq.DeleteStatement("users")
        assert '"users"' in stmt.to_sql("postgres")

        stmt = stmt.from_table("fonts")
        assert '"users"' not in stmt.to_sql("postgres")
        assert '"fonts"' in stmt.to_sql("postgres")

    def test_limit(self):
        stmt = rq.DeleteStatement("users").limit(20)
        assert "LIMIT" in stmt.to_sql("postgres")

    def test_order_by(self):
        stmt = rq.DeleteStatement("users").limit(20).order_by(rq.Ordering("id", "DESC"))
        assert 'ORDER BY "id" DESC' in stmt.to_sql("postgres")

    def test_returning(self):
        stmt = rq.DeleteStatement("users").returning(rq.Returning("id"))
        assert 'RETURNING "id"' in stmt.to_sql("postgres")

        stmt = rq.DeleteStatement("users").returning(rq.Returning.all())
        assert "RETURNING *" in stmt.to_sql("postgres")

    def test_where(self):
        stmt = (
            rq.DeleteStatement("users")
            .where(rq.Expr.col("id") == 1)
            .where(rq.Expr.col("id") == 2)
        )
        assert "AND" in stmt.to_sql("postgres")


class TestInsertStatement:
    def test_init(self):
        stmt = rq.InsertStatement("public.users")
        assert '"public"."users"' in stmt.to_sql("postgres")

    def test_columns(self):
        stmt = rq.InsertStatement("users").columns("id", "name")
        assert '"id"' in stmt.to_sql("postgres")
        assert '"name"' in stmt.to_sql("postgres")

        stmt.columns("created_at")
        assert '"created_at"' in stmt.to_sql("postgres")
        assert '"id"' not in stmt.to_sql("postgres")
        assert '"name"' not in stmt.to_sql("postgres")

    def test_values(self):
        with pytest.raises(TypeError):
            rq.InsertStatement("users").values(1, a=1)  # type: ignore

        with pytest.raises(ValueError):
            rq.InsertStatement("users").values(name="ali", id=1).values(name="ali")  # type: ignore

        with pytest.raises(ValueError):
            rq.InsertStatement("users").columns("id").values(1, 2)

        with pytest.raises(ValueError):
            rq.InsertStatement("users").columns("id").values()

        stmt = rq.InsertStatement("users").columns("id").values("ali")
        assert stmt.to_sql("postgres").count("ali") == 1

        stmt = rq.InsertStatement("users").columns("id").values("ali").values("ali")
        assert stmt.to_sql("postgres").count("ali") == 2

        stmt = rq.InsertStatement("users").values(name="ali", id=1)
        assert "name" in stmt.to_sql("postgres")
        assert "id" in stmt.to_sql("postgres")
        assert "ali" in stmt.to_sql("postgres")
        assert "1" in stmt.to_sql("postgres")

        stmt = (
            rq.InsertStatement("users")
            .values(name="ali", id=1)
            .values(name="ali", id=1)
        )
        assert stmt.to_sql("postgres").count("ali") == 2
        assert stmt.to_sql("postgres").count("1") == 2

    def test_into(self):
        stmt = (
            rq.InsertStatement("name_1")
            .values(aspect=4.21, image="123")
            .into("name_2")
            .to_sql("sqlite")
        )
        assert "name_2" in stmt
        assert "name_1" not in stmt

    # TODO: test_on_conflict
    # TODO: test_returning
    # TODO: test_replace
    # TODO: test_or_default_values

    def test_select_from(self):
        valid_cols = rq.SelectStatement().columns("id", "name").from_table("fonts")
        invalid_cols = rq.SelectStatement().columns("id").from_table("fonts")

        stmt = rq.InsertStatement("users").columns("id", "name")

        with pytest.raises(ValueError):
            stmt.select_from(invalid_cols)

        stmt.select_from(valid_cols)
        assert "SELECT" in stmt.to_sql("postgres")


class TestOnConflict:
    def test_init(self):
        pass
