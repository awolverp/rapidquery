import typing

import rapidquery as rq


class TableNameInitMixin:
    table_name_instance: typing.Callable

    def test_table_name_init(self):
        class Prop:
            @property
            def __table_name__(self):
                return "world.public.users"

        class Var:
            __table_name__ = "world.public.users"

        self.table_name_instance(rq.TableName("users", "public", "world"))
        self.table_name_instance("world.public.users")
        self.table_name_instance(Prop())
        self.table_name_instance(Var)


class ReturningMixin:
    def get_statement(self):
        raise NotImplementedError

    def test_returning(self):
        stmt = self.get_statement().returning(rq.Returning("id"))
        assert 'RETURNING "id"' in stmt.to_sql("postgres")

        stmt = self.get_statement().returning(rq.Returning.all())
        assert "RETURNING *" in stmt.to_sql("postgres")


class OrderByMixin:
    def get_statement(self):
        raise NotImplementedError

    def test_order_by(self):
        stmt = self.get_statement().order_by(rq.Ordering("id", "DESC"))
        assert 'ORDER BY "id" DESC' in stmt.to_sql("postgres")

    def test_clear_order_by(self):
        stmt = self.get_statement().order_by(rq.Ordering("id"))
        assert "ORDER BY" in stmt.to_sql("mysql")

        stmt = stmt.clear_order_by()
        assert "ORDER BY" not in stmt.to_sql("mysql")


class WhereMixin:
    def get_statement(self):
        raise NotImplementedError

    def test_clear_where(self):
        stmt = self.get_statement().where(rq.Expr.val(True))
        assert "WHERE" in stmt.to_sql("mysql")

        stmt = stmt.clear_where()
        assert "WHERE" not in stmt.to_sql("mysql")

    def test_where(self):
        stmt = (
            self.get_statement()
            .where(rq.Expr.col("id") > 10)
            .where(rq.Expr.col("name") < 20)
            .to_sql("sqlite")
        )
        assert 'WHERE "id" > 10 AND "name" < 20' in stmt
