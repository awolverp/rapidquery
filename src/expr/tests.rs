use crate::data_type::PyDataType;
use crate::dialect::SqlDialect;
use super::ident::{Ident, IdentPart};
use crate::sql::{PrepareError, SqlWriter, WriteSql};

use super::enum_::{BinaryOp, DuplicateTreatment, ExprEnum, IsKind, LikeKind, UnaryOp};
use super::func::{Func, FuncArguments};

struct TestWriter(String);

impl SqlWriter for TestWriter {
    fn push_str(&mut self, s: &str) -> &mut Self {
        self.0.push_str(s);
        self
    }

    fn push_char(&mut self, c: char) -> &mut Self {
        self.0.push(c);
        self
    }

    fn push_value(
        &mut self,
        _dialect: &SqlDialect,
        _serializer: &crate::data_type::SerializerEnum,
        _object: &crate::internal::alias::PyObject,
    ) -> crate::sql::PrepareResult<()> {
        panic!("test writer does not support values");
    }
}

impl std::fmt::Write for TestWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.push_str(s);
        Ok(())
    }
}

fn quote(s: &str) -> IdentPart {
    IdentPart::quoted(s).unwrap()
}

fn ident(name: &str) -> ExprEnum {
    ExprEnum::Ident(Ident::from_one(quote(name)))
}

fn render(expr: &ExprEnum, dialect: SqlDialect) -> String {
    let mut out = TestWriter(String::new());
    expr.write_sql(&dialect, &mut out).unwrap();
    out.0
}

fn pg(expr: &ExprEnum) -> String {
    render(expr, SqlDialect::PostgreSql)
}

#[test]
fn ident_uses_dialect_quotes() {
    let expr = ident("id");
    assert_eq!(pg(&expr), "\"id\"");
    assert_eq!(render(&expr, SqlDialect::MySql), "`id`");
}

#[test]
fn binary_add_and_and() {
    let add = ExprEnum::Binary {
        left: Box::new(ident("a")),
        op: BinaryOp::Add,
        right: Box::new(ident("b")),
    };
    assert_eq!(pg(&add), "\"a\" + \"b\"");

    let and = ExprEnum::Binary {
        left: Box::new(ident("a")),
        op: BinaryOp::And,
        right: Box::new(ident("b")),
    };
    assert_eq!(pg(&and), "\"a\" AND \"b\"");
}

#[test]
fn or_parenthesizes_weaker_and_child_on_right_of_and() {
    let or = ExprEnum::Binary {
        left: Box::new(ident("a")),
        op: BinaryOp::Or,
        right: Box::new(ident("b")),
    };
    let and = ExprEnum::Binary {
        left: Box::new(or),
        op: BinaryOp::And,
        right: Box::new(ident("c")),
    };
    assert_eq!(pg(&and), "(\"a\" OR \"b\") AND \"c\"");
}

#[test]
fn unary_not_and_minus() {
    let not = ExprEnum::Unary {
        op: UnaryOp::Not,
        expr: Box::new(ident("x")),
    };
    assert_eq!(pg(&not), "NOT \"x\"");

    let minus = ExprEnum::Unary {
        op: UnaryOp::Minus,
        expr: Box::new(ident("x")),
    };
    assert_eq!(pg(&minus), "-\"x\"");
}

#[test]
fn function_distinct_and_filter_and_bare() {
    let sum = ExprEnum::Function(Func {
        name: Ident::from_one(quote("SUM")),
        args: FuncArguments::List {
            duplicate_treatment: Some(DuplicateTreatment::Distinct),
            args: vec![ident("amount")],
        },
        filter: Some(Box::new(ExprEnum::Binary {
            left: Box::new(ident("amount")),
            op: BinaryOp::Gt,
            right: Box::new(ident("zero")),
        })),
    });
    assert_eq!(
        pg(&sum),
        "\"SUM\"(DISTINCT \"amount\") FILTER (WHERE \"amount\" > \"zero\")"
    );

    let now = ExprEnum::Function(Func {
        name: Ident::from_one(quote("CURRENT_TIMESTAMP")),
        args: FuncArguments::None,
        filter: None,
    });
    assert_eq!(pg(&now), "\"CURRENT_TIMESTAMP\"");
}

#[test]
fn is_in_between_like_cast_case_tuple() {
    let is_null = ExprEnum::Is {
        expr: Box::new(ident("x")),
        negated: true,
        kind: IsKind::Null,
    };
    assert_eq!(pg(&is_null), "\"x\" IS NOT NULL");

    let in_list = ExprEnum::InList {
        expr: Box::new(ident("x")),
        list: vec![ident("a"), ident("b")],
        negated: false,
    };
    assert_eq!(pg(&in_list), "\"x\" IN (\"a\", \"b\")");

    let between = ExprEnum::Between {
        expr: Box::new(ident("x")),
        negated: false,
        low: Box::new(ident("a")),
        high: Box::new(ident("b")),
    };
    assert_eq!(pg(&between), "\"x\" BETWEEN \"a\" AND \"b\"");

    let like = ExprEnum::Like {
        kind: LikeKind::ILike,
        negated: false,
        expr: Box::new(ident("name")),
        pattern: Box::new(ident("pat")),
        escape: None,
    };
    assert_eq!(pg(&like), "\"name\" ILIKE \"pat\"");

    let cast = ExprEnum::Cast {
        expr: Box::new(ident("x")),
        data_type: PyDataType::from_name("INT"),
    };
    assert_eq!(pg(&cast), "CAST(\"x\" AS INT)");

    let case = ExprEnum::Case {
        operand: Some(Box::new(ident("x"))),
        when_then: vec![(ident("a"), ident("b"))],
        else_result: Some(Box::new(ident("c"))),
    };
    assert_eq!(pg(&case), "CASE \"x\" WHEN \"a\" THEN \"b\" ELSE \"c\" END");

    let tuple = ExprEnum::Tuple(vec![ident("a"), ident("b")]);
    assert_eq!(pg(&tuple), "(\"a\", \"b\")");
}

#[test]
fn custom_placeholders_and_braces() {
    let expr = ExprEnum::Custom {
        sql: "json_agg(DISTINCT {})".to_string(),
        args: vec![ident("col")],
    };
    assert_eq!(pg(&expr), "json_agg(DISTINCT \"col\")");

    let expr = ExprEnum::Custom {
        sql: "a {{ b }} c".to_string(),
        args: vec![],
    };
    assert_eq!(pg(&expr), "a { b } c");

    let expr = ExprEnum::Custom {
        sql: "{} {}".to_string(),
        args: vec![ident("a")],
    };
    let err = {
        let mut out = TestWriter(String::new());
        expr.write_sql(&SqlDialect::PostgreSql, &mut out)
            .unwrap_err()
    };
    assert!(matches!(
        err,
        PrepareError::CustomPlaceholders {
            expected: 2,
            got: 1
        }
    ));
}

#[test]
fn binary_op_parse_unknown_is_custom() {
    assert_eq!(BinaryOp::parse("->"), BinaryOp::Custom(arcstr::ArcStr::from("->")));
    assert_eq!(BinaryOp::parse("!=").as_sql(), "<>");
}
