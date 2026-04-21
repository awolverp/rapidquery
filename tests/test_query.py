import pytest

import rapidquery as rq

from .mixin import OrderByMixin, ReturningMixin, TableNameInitMixin, WhereMixin


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
        assert '"test"' in sql

        stmt = (
            rq.CaseStatement()
            .when(rq.Expr.col("id") == 1, 1)
            .when(rq.Expr.col("name") == "ali", 1)
        )
        sql = rq.SelectStatement(stmt.label("test")).to_sql("postgres")
        assert '"id"' in sql
        assert '"name"' in sql
        assert '"test"' in sql

        with pytest.raises(TypeError):
            rq.CaseStatement().when("Ali", 1)  # type: ignore

        rq.CaseStatement().when(rq.Expr("Ali"), 1).else_(1)


class TestDeleteStatement(TableNameInitMixin, ReturningMixin, WhereMixin, OrderByMixin):
    table_name_instance = rq.DeleteStatement

    def get_statement(self):
        return rq.DeleteStatement("users")

    def test_from_table(self):
        stmt = rq.DeleteStatement("users")
        assert '"users"' in stmt.to_sql("postgres")

        stmt = stmt.from_table("fonts")
        assert '"users"' not in stmt.to_sql("postgres")
        assert '"fonts"' in stmt.to_sql("postgres")

    def test_limit(self):
        stmt = rq.DeleteStatement("users").limit(20)
        assert "LIMIT" in stmt.to_sql("postgres")


class TestInsertStatement(TableNameInitMixin, ReturningMixin):
    table_name_instance = rq.InsertStatement

    def get_statement(self):
        return rq.InsertStatement("users")

    def test_replace(self):
        stmt = rq.InsertStatement("public.users").replace().to_sql("postgres")
        assert "INSERT" not in stmt
        assert "REPLACE" in stmt

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

    def test_or_default_values(self):
        stmt = rq.InsertStatement("users").or_default_values()
        assert stmt.to_sql("postgres").count("DEFAULT") == 1

        stmt = rq.InsertStatement("users").or_default_values(4)
        assert stmt.to_sql("postgres").count("DEFAULT") == 4

        stmt = rq.InsertStatement("users").or_default_values(4).values(aspect=3.14)
        assert stmt.to_sql("postgres").count("DEFAULT") == 0

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
        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_update(image="ex"))
        )
        assert 'ON CONFLICT ("id")' in stmt.to_sql("postgres")
        assert "DO UPDATE" in stmt.to_sql("postgres")

    def test_action_where(self):
        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(
                rq.OnConflict("id")
                .do_update(image="ex")
                .action_where(rq.Expr.col("aspect").is_null())
            )
        )
        assert "WHERE" in stmt.to_sql("postgres")

    def test_target_where(self):
        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(
                rq.OnConflict("id")
                .do_update(image="ex")
                .target_where(rq.Expr.col("aspect").is_null())
            )
        )
        assert "WHERE" in stmt.to_sql("postgres")

    def test_do_update(self):
        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_update("aspect", image=rq.Expr(1) + 2))
        )
        assert "DO UPDATE" in stmt.to_sql("postgres")

    def test_do_nothing(self):
        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_nothing())
        )
        assert "DO NOTHING" in stmt.to_sql("postgres")

        stmt = (
            rq.InsertStatement("glyph")
            .values(aspect=3.1415, image="abcd")
            .on_conflict(rq.OnConflict("id").do_nothing("id"))
        )
        assert "DO NOTHING" in stmt.to_sql("postgres")


class TestSelectStatement(WhereMixin, OrderByMixin):
    table_name_instance = rq.SelectStatement

    def get_statement(self):
        return rq.SelectStatement().columns("character").from_table("characters")

    def test_init(self):
        stmt = (
            rq.SelectStatement()
            .columns("character", "fonts.name")
            .from_table("characters")
            .join(
                "fonts",
                rq.Expr.col("characters.font_id") == rq.Expr.col("fonts.id"),
                "LEFT",
            )
            .where(rq.Expr.col("size_w").in_((3, 4)))
            .where(rq.Expr.col("character").like("A%"))
            .to_sql("postgres")
        )
        assert 'LEFT JOIN "fonts" ON "characters"."font_id" = "fonts"."id"' in stmt

        stmt = rq.SelectStatement(
            rq.SelectLabel(1),
            rq.SelectLabel("hello"),
            rq.Expr.col("font"),
        ).to_sql("postgres")
        assert "SELECT 1, 'hello', \"font\"" in stmt

    def test_columns(self):
        stmt = (
            rq.SelectStatement()
            .columns("character", "size_w", "size_h")
            .from_table("characters")
            .to_sql("mysql")
        )
        assert (
            "`character` AS `character`, `size_w` AS `size_w`, `size_h` AS `size_h`"
            in stmt
        )

    def test_distinct(self):
        stmt = (
            rq.SelectStatement()
            .distinct()
            .columns("character", "size_w", "size_h")
            .from_table("characters")
            .to_sql("mysql")
        )
        assert "DISTINCT" in stmt

    def test_from_function(self):
        stmt = (
            rq.SelectStatement(rq.Expr.asterisk())
            .from_function(rq.Func.random(), "func")
            .to_sql("postgres")
        )
        assert "FROM RANDOM()" in stmt

    def test_from_subquery(self):
        rq.SelectStatement().from_subquery(
            rq.SelectStatement().columns("image", "aspect").from_table("glyph"),
            "subglyph",
        )

        stmt = rq.SelectStatement()
        with pytest.raises(ValueError):
            stmt.from_subquery(stmt, "a")


class TestUpdateStatement(WhereMixin, OrderByMixin):
    table_name_instance = rq.UpdateStatement

    def get_statement(self):
        return rq.UpdateStatement("users")

    def test_from_table(self):
        stmt = (
            rq.UpdateStatement("archive.users")
            .from_table("public.users")
            .to_sql("postgres")
        )
        assert "FROM" in stmt
        assert '"public"."users"' in stmt
        assert '"archive"."users"' in stmt

    def test_table(self):
        stmt = (
            rq.UpdateStatement("name_1")
            .values(aspect=1.23, image=123)
            .table("name_2")
            .to_sql("sqlite")
        )
        assert "name_1" not in stmt
        assert "name_2" in stmt

    def test_values(self):
        stmt = (
            rq.UpdateStatement("glyph")
            .values(aspect=1.23, image=123)
            .values(font_id=20)
            .to_sql("sqlite")
        )
        assert "aspect" in stmt
        assert "image" in stmt
        assert "font_id" in stmt


class TestWindowStatement:
    def test_basic_partition(self):
        """Test a basic OVER clause with PARTITION BY."""
        window = rq.WindowStatement("department").order_by(
            rq.Ordering("salary", order="DESC")
        )

        # Using it in a SelectStatement context
        stmt = rq.SelectStatement(
            rq.Expr.custom("AVG(salary)").label("svg_sal", window)
        ).from_table("employees")

        sql = stmt.to_sql("postgres")
        assert 'PARTITION BY "department"' in sql
        assert 'ORDER BY "salary" DESC' in sql
        assert "OVER" in sql

    def test_frame_rows(self):
        """Test WINDOW with ROWS framing."""
        window = (
            rq.WindowStatement()
            .order_by(rq.Ordering("id"))
            .frame("ROWS", rq.Frame.preceding(5), rq.Frame.current_row())
        )

        stmt = rq.SelectStatement(
            rq.Expr.custom("SUM(val)").label("running_sum", window)
        ).from_table("data")

        sql = stmt.to_sql("postgres")
        assert "ROWS BETWEEN 5PRECEDING AND CURRENT ROW" in sql

    def test_frame_unbounded(self):
        """Test WINDOW with UNBOUNDED boundaries."""
        window = rq.WindowStatement().frame(
            "RANGE", rq.Frame.unbounded_preceding(), rq.Frame.unbounded_following()
        )

        stmt = rq.SelectStatement(
            rq.SelectLabel(rq.Expr.custom("COUNT(*)"), "total", window)
        ).from_table("users")

        sql = stmt.to_sql("mysql")
        assert "RANGE BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING" in sql


class TestWith:
    def test_simple_cte(self):
        """Test a basic CTE using WithClause."""
        cte_query = (
            rq.SelectStatement()
            .columns("id", "name")
            .from_table("users")
            .where(rq.Expr.col("active").is_(True))
        )

        with_query = (
            rq.WithClause()
            .cte("active_users", cte_query)
            .query(rq.SelectStatement().columns("*").from_table("active_users"))
        )

        sql = with_query.to_sql("postgres")
        assert 'WITH "active_users"' in sql
        assert 'SELECT "id" AS "id", "name" AS "name" FROM "users"' in sql
        assert 'SELECT * FROM "active_users"' in sql

    def test_multiple_ctes(self):
        """Test multiple CTEs in a single WITH clause."""
        q1 = rq.SelectStatement().columns("id").from_table("table_a")
        q2 = rq.SelectStatement().columns("id").from_table("table_b")

        with_query = (
            rq.WithClause()
            .cte("cte_a", q1)
            .cte("cte_b", q2)
            .query(
                rq.SelectStatement()
                .columns("cte_a.id")
                .from_table("cte_a")
                .join("cte_b", rq.Expr.col("cte_a.id") == rq.Expr.col("cte_b.id"))
            )
        )

        sql = with_query.to_sql("postgres")
        assert 'WITH "cte_a"' in sql
        assert '"cte_b"' in sql
        assert 'JOIN "cte_b"' in sql

    def test_recursive(self):
        """Test RECURSIVE WITH clause generation."""
        # A simplified recursive CTE for sequence generation
        base_part = rq.SelectStatement(rq.Expr.val(1).label("n"))
        recursive_part = (
            rq.SelectStatement((rq.Expr.col("n") + 1).label("n"))
            .from_table("nums")
            .where(rq.Expr.col("n") < 5)
        )

        # SeaQuery/RapidQuery typically uses Union to combine parts for recursion
        union_query = base_part.union(recursive_part)

        with_query = (
            rq.WithClause()
            .recursive()
            .cte("nums", union_query, columns=["n"])
            .query(rq.SelectStatement().columns("*").from_table("nums"))
        )

        sql = with_query.to_sql("postgres")
        assert "WITH RECURSIVE" in sql
        assert '"nums" ("n") AS' in sql

    def test_materialized(self):
        """Test the MATERIALIZED hint in CTEs (Postgres specific)."""
        q = rq.SelectStatement().columns("id").from_table("very_large_table")

        # Materialized True
        sql_mat = (
            rq.WithClause()
            .cte("cached_data", q, materialized=True)
            .query(rq.SelectStatement().from_table("cached_data"))
            .to_sql("postgres")
        )
        assert "MATERIALIZED" in sql_mat

        # Materialized False (NOT MATERIALIZED)
        sql_not_mat = (
            rq.WithClause()
            .cte("raw_data", q, materialized=False)
            .query(rq.SelectStatement().from_table("raw_data"))
            .to_sql("postgres")
        )
        assert "NOT MATERIALIZED" in sql_not_mat
