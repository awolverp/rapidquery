import pytest

import rapidquery as rq


class TestCreateSchema:
    def test_new(self):
        stmt = rq.ddl.CreateSchema("name")
        assert stmt.name.to_list() == ["name"]
        assert stmt.if_not_exists is False
        assert stmt.authorization is None
        assert stmt.location is None

        stmt = rq.ddl.CreateSchema("name", if_not_exists=True, authorization="auth", location="loc")
        assert stmt.name.to_list() == ["name"]
        assert stmt.if_not_exists is True
        assert stmt.authorization == "auth"
        assert stmt.location == "loc"

    def test_unsupported(self):
        stmt = rq.ddl.CreateSchema("name")

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.Sqlite)

    def test_supported(self):
        stmt = rq.ddl.CreateSchema("name")
        assert stmt.to_sql(rq.dialect.PostgreSql) == 'CREATE SCHEMA "name"'

    def test_is_database(self):
        stmt = rq.ddl.CreateSchema("name")

        dl = rq.dialect.Dialect(schema_features=rq.dialect.OPT_SCHEMA_FEATURES_IS_DATABASE)
        assert stmt.to_sql(dl) == 'CREATE DATABASE "name"'

    def test_if_not_exists(self):
        stmt = rq.ddl.CreateSchema("name", if_not_exists=True)
        assert stmt.if_not_exists is True

        assert stmt.to_sql(rq.dialect.PostgreSql) == 'CREATE SCHEMA IF NOT EXISTS "name"'

        # Should ignore IF NOT EXISTS when a dialect doesn't support it
        assert stmt.to_sql(rq.dialect.Dialect()) == 'CREATE SCHEMA "name"'

    def test_authorization(self):
        stmt = rq.ddl.CreateSchema("name", authorization="user")
        assert stmt.authorization == "user"

        # Unsupported
        dl = rq.dialect.Dialect(schema_authorization_style=rq.dialect.OPT_SCHEMA_AUTHORIZATION_STYLE_UNSUPPORTED)
        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(dl)

        # Optional
        dl = rq.dialect.Dialect(schema_authorization_style=rq.dialect.OPT_SCHEMA_AUTHORIZATION_STYLE_OPTIONAL)
        assert stmt.to_sql(dl) == 'CREATE SCHEMA "name" AUTHORIZATION "user"'

        stmt = rq.ddl.CreateSchema("name", authorization=None)
        assert stmt.to_sql(dl) == 'CREATE SCHEMA "name"'

    def test_location(self):
        stmt = rq.ddl.CreateSchema("name", location="s3://depts/finance")
        assert stmt.location == "s3://depts/finance"

        # Unsupported
        dl = rq.dialect.Dialect(schema_location_style=rq.dialect.OPT_SCHEMA_LOCATION_STYLE_UNSUPPORTED)
        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(dl)

        # Location Keyword
        dl = rq.dialect.Dialect(
            string_format=rq.dialect.OPT_STRING_FORMAT_SINGLE_QUOTE,
            schema_location_style=rq.dialect.OPT_SCHEMA_LOCATION_STYLE_LOCATION_KEYWORD,
        )
        assert stmt.to_sql(dl) == "CREATE SCHEMA \"name\" LOCATION 's3://depts/finance'"

        # ManagedLocation Keyword
        dl = rq.dialect.Dialect(
            string_format=rq.dialect.OPT_STRING_FORMAT_SINGLE_QUOTE,
            schema_location_style=rq.dialect.OPT_SCHEMA_LOCATION_STYLE_MANAGED_LOCATION_KEYWORD,
        )
        assert stmt.to_sql(dl) == "CREATE SCHEMA \"name\" MANAGED LOCATION 's3://depts/finance'"

        # External Volume
        dl = rq.dialect.Dialect(
            string_format=rq.dialect.OPT_STRING_FORMAT_SINGLE_QUOTE,
            schema_location_style=rq.dialect.OPT_SCHEMA_LOCATION_STYLE_EXTERNAL_VOLUME,
        )
        assert stmt.to_sql(dl) == "CREATE SCHEMA \"name\" EXTERNAL_VOLUME = 's3://depts/finance'"


class TestDropSchema:
    def test_new(self):
        stmt = rq.ddl.DropSchema("name")
        assert stmt.name.to_list() == ["name"]
        assert stmt.if_exists is False
        assert stmt.behavior is None

        stmt = rq.ddl.DropSchema("name", if_exists=True, behavior="CASCADE")
        assert stmt.name.to_list() == ["name"]
        assert stmt.if_exists is True
        assert stmt.behavior == "CASCADE"

        stmt = rq.ddl.DropSchema("name", behavior="Restrict")
        assert stmt.name.to_list() == ["name"]
        assert stmt.if_exists is False
        assert stmt.behavior == "RESTRICT"

        with pytest.raises(ValueError):
            _ = rq.ddl.DropSchema("name", behavior="invalid")

    def test_unsupported(self):
        stmt = rq.ddl.DropSchema("name")

        with pytest.raises(RuntimeError, match="supported"):
            _ = stmt.to_sql(rq.dialect.Sqlite)

    def test_supported(self):
        stmt = rq.ddl.DropSchema("name")
        assert stmt.to_sql(rq.dialect.PostgreSql) == 'DROP SCHEMA "name"'

    def test_if_exists(self):
        stmt = rq.ddl.DropSchema("name", if_exists=True)
        assert stmt.if_exists is True

        assert stmt.to_sql(rq.dialect.PostgreSql) == 'DROP SCHEMA IF EXISTS "name"'

        # Should ignore IF EXISTS when a dialect doesn't support it
        assert stmt.to_sql(rq.dialect.Dialect()) == 'DROP SCHEMA "name"'

    def test_behavior(self):
        stmt = rq.ddl.DropSchema("name", behavior="CASCADE")
        assert stmt.behavior == "CASCADE"

        # Unsupported
        dl = rq.dialect.Dialect()
        with pytest.raises(RuntimeError):
            _ = stmt.to_sql(dl)

        # CASCADE Supported
        dl = rq.dialect.Dialect(schema_features=rq.dialect.OPT_SCHEMA_FEATURES_DROP_CASCADE)
        assert stmt.to_sql(dl) == 'DROP SCHEMA "name" CASCADE'

        # CASCADE Unsupported
        dl = rq.dialect.Dialect()
        with pytest.raises(RuntimeError):
            _ = stmt.to_sql(dl)

        # RESTRICT Supported
        stmt = rq.ddl.DropSchema("name", behavior="RESTRICT")
        assert stmt.behavior == "RESTRICT"

        dl = rq.dialect.Dialect(schema_features=rq.dialect.OPT_SCHEMA_FEATURES_DROP_RESTRICT)
        assert stmt.to_sql(dl) == 'DROP SCHEMA "name" RESTRICT'

        # RESTRICT Unsupported
        dl = rq.dialect.Dialect()
        with pytest.raises(RuntimeError):
            _ = stmt.to_sql(dl)
