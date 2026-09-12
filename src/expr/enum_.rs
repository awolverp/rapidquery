//! Engine expression tree.
//!
//! Python classes are empty views over [`ExprEnum`] (Serializer-style).
//! Query-shaped variants are omitted until `Query` exists.

use std::fmt::Write as _;
use std::hash::{Hash, Hasher};

use arcstr::ArcStr;
use pyo3::types::PyAnyMethods;

use crate::data_type::{PyDataType, serializer_repr};
use crate::dialect::SqlDialect;
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};

use super::func::Func;
use super::ident::Ident;
use super::value::Value;

/// How duplicate values are treated inside function argument lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DuplicateTreatment {
    /// Consider only unique values.
    Distinct,
    /// Retain all duplicate values (the default).
    All,
}

impl DuplicateTreatment {
    pub(crate) fn as_sql(self) -> &'static str {
        match self {
            Self::Distinct => "DISTINCT",
            Self::All => "ALL",
        }
    }
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
    BitNot,
}

impl UnaryOp {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "NOT" | "not" | "Not" => Some(Self::Not),
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "~" => Some(Self::BitNot),
            _ => None,
        }
    }

    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Not => "NOT",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::BitNot => "~",
        }
    }
}

/// Binary operator. Unknown spellings become [`BinaryOp::Custom`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
    Xor,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    StringConcat,
    Custom(ArcStr),
}

impl BinaryOp {
    pub fn parse(s: &str) -> Self {
        match s {
            "+" => Self::Add,
            "-" => Self::Sub,
            "*" => Self::Mul,
            "/" => Self::Div,
            "%" => Self::Mod,
            "=" => Self::Eq,
            "<>" | "!=" => Self::NotEq,
            "<" => Self::Lt,
            "<=" => Self::LtEq,
            ">" => Self::Gt,
            ">=" => Self::GtEq,
            "AND" | "and" | "And" => Self::And,
            "OR" | "or" | "Or" => Self::Or,
            "XOR" | "xor" | "Xor" => Self::Xor,
            "&" => Self::BitAnd,
            "|" => Self::BitOr,
            "^" => Self::BitXor,
            "<<" => Self::LShift,
            ">>" => Self::RShift,
            "||" => Self::StringConcat,
            other => Self::Custom(ArcStr::from(other)),
        }
    }

    pub fn as_sql(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Eq => "=",
            Self::NotEq => "<>",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::LShift => "<<",
            Self::RShift => ">>",
            Self::StringConcat => "||",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Right-hand side of `IS [NOT] …`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsKind {
    Null,
    True,
    False,
    Unknown,
}

impl IsKind {
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("NULL") {
            Some(Self::Null)
        } else if s.eq_ignore_ascii_case("TRUE") {
            Some(Self::True)
        } else if s.eq_ignore_ascii_case("FALSE") {
            Some(Self::False)
        } else if s.eq_ignore_ascii_case("UNKNOWN") {
            Some(Self::Unknown)
        } else {
            None
        }
    }

    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Null => "NULL",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Pattern-matching operator family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LikeKind {
    Like,
    ILike,
    SimilarTo,
    RLike,
}

impl LikeKind {
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("LIKE") {
            Some(Self::Like)
        } else if s.eq_ignore_ascii_case("ILIKE") {
            Some(Self::ILike)
        } else if s.eq_ignore_ascii_case("SIMILAR TO") {
            Some(Self::SimilarTo)
        } else if s.eq_ignore_ascii_case("RLIKE") {
            Some(Self::RLike)
        } else {
            None
        }
    }

    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Like => "LIKE",
            Self::ILike => "ILIKE",
            Self::SimilarTo => "SIMILAR TO",
            Self::RLike => "RLIKE",
        }
    }
}

/// SQL expression tree.
///
/// TODO: `Exists`, `InSubquery`, `Subquery`, and `Quantified` (`ANY` /
/// `ALL` / `SOME`) once `Query` exists.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprEnum {
    Ident(Ident),
    Value(Value),
    Tuple(Vec<ExprEnum>),
    Unary {
        op: UnaryOp,
        expr: Box<ExprEnum>,
    },
    Binary {
        left: Box<ExprEnum>,
        op: BinaryOp,
        right: Box<ExprEnum>,
    },
    Function(Func),
    Is {
        expr: Box<ExprEnum>,
        negated: bool,
        kind: IsKind,
    },
    IsDistinctFrom {
        left: Box<ExprEnum>,
        right: Box<ExprEnum>,
        negated: bool,
    },
    InList {
        expr: Box<ExprEnum>,
        list: Vec<ExprEnum>,
        negated: bool,
    },
    Between {
        expr: Box<ExprEnum>,
        negated: bool,
        low: Box<ExprEnum>,
        high: Box<ExprEnum>,
    },
    Like {
        kind: LikeKind,
        negated: bool,
        expr: Box<ExprEnum>,
        pattern: Box<ExprEnum>,
        escape: Option<char>,
    },
    Cast {
        expr: Box<ExprEnum>,
        data_type: PyDataType,
    },
    Case {
        operand: Option<Box<ExprEnum>>,
        when_then: Vec<(ExprEnum, ExprEnum)>,
        else_result: Option<Box<ExprEnum>>,
    },
    Custom {
        sql: String,
        args: Vec<ExprEnum>,
    },
}

impl ExprEnum {
    /// Lower is tighter binding.
    fn precedence(&self) -> u8 {
        match self {
            Self::Ident(_)
            | Self::Value(_)
            | Self::Tuple(_)
            | Self::Function(_)
            | Self::Cast { .. }
            | Self::Case { .. }
            | Self::Custom { .. } => 0,
            Self::Unary { .. } => 1,
            Self::Binary { op, .. } => match op {
                BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 2,
                BinaryOp::Add | BinaryOp::Sub => 3,
                BinaryOp::BitAnd
                | BinaryOp::BitOr
                | BinaryOp::BitXor
                | BinaryOp::LShift
                | BinaryOp::RShift
                | BinaryOp::StringConcat => 4,
                BinaryOp::Eq
                | BinaryOp::NotEq
                | BinaryOp::Lt
                | BinaryOp::LtEq
                | BinaryOp::Gt
                | BinaryOp::GtEq
                | BinaryOp::Custom(_) => 5,
                BinaryOp::And => 6,
                BinaryOp::Xor => 7,
                BinaryOp::Or => 8,
            },
            Self::Is { .. }
            | Self::IsDistinctFrom { .. }
            | Self::InList { .. }
            | Self::Between { .. }
            | Self::Like { .. } => 5,
        }
    }

    fn write_operand<W: SqlWriter>(
        &self,
        dialect: &SqlDialect,
        out: &mut W,
        parent_prec: u8,
        paren_if_equal: bool,
    ) -> PrepareResult<()> {
        let child_prec = self.precedence();
        let paren = child_prec > parent_prec || (paren_if_equal && child_prec == parent_prec);
        if paren {
            out.push_char('(');
        }
        self.write_sql(dialect, out)?;
        if paren {
            out.push_char(')');
        }
        Ok(())
    }

    pub(crate) fn py_eq(
        &self,
        py: pyo3::Python<'_>,
        other: &Self,
    ) -> pyo3::PyResult<bool> {
        match (self, other) {
            (Self::Value(a), Self::Value(b)) => {
                if a.serializer != b.serializer {
                    return Ok(false);
                }
                a.object.bind(py).eq(b.object.bind(py))
            }
            (a, b) => Ok(a == b),
        }
    }

    pub(crate) fn py_hash(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<u64> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::mem::discriminant(self).hash(&mut hasher);
        match self {
            Self::Ident(ident) => ident.hash(&mut hasher),
            Self::Value(value) => {
                value.object.bind(py).hash()?.hash(&mut hasher);
                value.serializer.hash(&mut hasher);
            }
            Self::Tuple(items) => {
                items.len().hash(&mut hasher);
                for item in items {
                    item.py_hash(py)?.hash(&mut hasher);
                }
            }
            Self::Unary { op, expr } => {
                op.hash(&mut hasher);
                expr.py_hash(py)?.hash(&mut hasher);
            }
            Self::Binary { left, op, right } => {
                left.py_hash(py)?.hash(&mut hasher);
                op.hash(&mut hasher);
                right.py_hash(py)?.hash(&mut hasher);
            }
            Self::Function(func) => {
                func.name.hash(&mut hasher);
                // args/filter hashed via recursive walk in python_repr-level identity
                match &func.args {
                    super::func::FuncArguments::None => 0u8.hash(&mut hasher),
                    super::func::FuncArguments::List {
                        duplicate_treatment,
                        args,
                    } => {
                        1u8.hash(&mut hasher);
                        duplicate_treatment.hash(&mut hasher);
                        args.len().hash(&mut hasher);
                        for arg in args {
                            arg.py_hash(py)?.hash(&mut hasher);
                        }
                    }
                }
                match &func.filter {
                    Some(expr) => {
                        1u8.hash(&mut hasher);
                        expr.py_hash(py)?.hash(&mut hasher);
                    }
                    None => 0u8.hash(&mut hasher),
                }
            }
            Self::Is {
                expr,
                negated,
                kind,
            } => {
                expr.py_hash(py)?.hash(&mut hasher);
                negated.hash(&mut hasher);
                kind.hash(&mut hasher);
            }
            Self::IsDistinctFrom {
                left,
                right,
                negated,
            } => {
                left.py_hash(py)?.hash(&mut hasher);
                right.py_hash(py)?.hash(&mut hasher);
                negated.hash(&mut hasher);
            }
            Self::InList {
                expr,
                list,
                negated,
            } => {
                expr.py_hash(py)?.hash(&mut hasher);
                negated.hash(&mut hasher);
                list.len().hash(&mut hasher);
                for item in list {
                    item.py_hash(py)?.hash(&mut hasher);
                }
            }
            Self::Between {
                expr,
                negated,
                low,
                high,
            } => {
                expr.py_hash(py)?.hash(&mut hasher);
                negated.hash(&mut hasher);
                low.py_hash(py)?.hash(&mut hasher);
                high.py_hash(py)?.hash(&mut hasher);
            }
            Self::Like {
                kind,
                negated,
                expr,
                pattern,
                escape,
            } => {
                kind.hash(&mut hasher);
                negated.hash(&mut hasher);
                expr.py_hash(py)?.hash(&mut hasher);
                pattern.py_hash(py)?.hash(&mut hasher);
                escape.hash(&mut hasher);
            }
            Self::Cast { expr, data_type } => {
                expr.py_hash(py)?.hash(&mut hasher);
                data_type.hash(&mut hasher);
            }
            Self::Case {
                operand,
                when_then,
                else_result,
            } => {
                match operand {
                    Some(expr) => {
                        1u8.hash(&mut hasher);
                        expr.py_hash(py)?.hash(&mut hasher);
                    }
                    None => 0u8.hash(&mut hasher),
                }
                when_then.len().hash(&mut hasher);
                for (when, then) in when_then {
                    when.py_hash(py)?.hash(&mut hasher);
                    then.py_hash(py)?.hash(&mut hasher);
                }
                match else_result {
                    Some(expr) => {
                        1u8.hash(&mut hasher);
                        expr.py_hash(py)?.hash(&mut hasher);
                    }
                    None => 0u8.hash(&mut hasher),
                }
            }
            Self::Custom { sql, args } => {
                sql.hash(&mut hasher);
                args.len().hash(&mut hasher);
                for arg in args {
                    arg.py_hash(py)?.hash(&mut hasher);
                }
            }
        }
        Ok(hasher.finish())
    }

    pub(crate) fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        match self {
            Self::Ident(ident) => Ok(ident.python_repr()),
            Self::Value(value) => value.python_repr(py),
            Self::Tuple(items) => {
                let mut buf = String::from("TupleExpr(");
                join_reprs(py, &mut buf, items)?;
                buf.push(')');
                Ok(buf)
            }
            Self::Unary { op, expr } => Ok(format!(
                "Unary({:?}, {})",
                op.as_sql(),
                expr.python_repr(py)?
            )),
            Self::Binary { left, op, right } => Ok(format!(
                "Binary({}, {:?}, {})",
                left.python_repr(py)?,
                op.as_sql(),
                right.python_repr(py)?
            )),
            Self::Function(func) => func.python_repr(py),
            Self::Is {
                expr,
                negated,
                kind,
            } => {
                let mut buf = format!("Is({}, {:?}", expr.python_repr(py)?, kind.as_sql());
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::IsDistinctFrom {
                left,
                right,
                negated,
            } => {
                let mut buf = format!(
                    "IsDistinctFrom({}, {}",
                    left.python_repr(py)?,
                    right.python_repr(py)?
                );
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::InList {
                expr,
                list,
                negated,
            } => {
                let mut buf = format!("InList({}, (", expr.python_repr(py)?);
                join_reprs(py, &mut buf, list)?;
                buf.push(')');
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::Between {
                expr,
                negated,
                low,
                high,
            } => {
                let mut buf = format!(
                    "Between({}, {}, {}",
                    expr.python_repr(py)?,
                    low.python_repr(py)?,
                    high.python_repr(py)?
                );
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::Like {
                kind,
                negated,
                expr,
                pattern,
                escape,
            } => {
                let mut buf = format!(
                    "Like({}, {}",
                    expr.python_repr(py)?,
                    pattern.python_repr(py)?
                );
                if *kind != LikeKind::Like {
                    write!(buf, ", kind={:?}", kind.as_sql()).unwrap();
                }
                if *negated {
                    buf.push_str(", negated=True");
                }
                if let Some(c) = escape {
                    write!(buf, ", escape={c:?}").unwrap();
                }
                buf.push(')');
                Ok(buf)
            }
            Self::Cast { expr, data_type } => Ok(format!(
                "Cast({}, DataType({:?}, {}))",
                expr.python_repr(py)?,
                data_type.name.as_str(),
                serializer_repr(&data_type.serializer)
            )),
            Self::Case {
                operand,
                when_then,
                else_result,
            } => {
                let mut buf = String::from("Case([");
                for (i, (when, then)) in when_then.iter().enumerate() {
                    if i > 0 {
                        buf.push_str(", ");
                    }
                    write!(
                        buf,
                        "({}, {})",
                        when.python_repr(py)?,
                        then.python_repr(py)?
                    )
                    .unwrap();
                }
                buf.push(']');
                if let Some(operand) = operand {
                    write!(buf, ", operand={}", operand.python_repr(py)?).unwrap();
                }
                if let Some(else_result) = else_result {
                    write!(buf, ", else_={}", else_result.python_repr(py)?).unwrap();
                }
                buf.push(')');
                Ok(buf)
            }
            Self::Custom { sql, args } => {
                let mut buf = format!("Custom({sql:?}");
                for arg in args {
                    buf.push_str(", ");
                    buf.push_str(&arg.python_repr(py)?);
                }
                buf.push(')');
                Ok(buf)
            }
        }
    }
}

fn join_reprs(
    py: pyo3::Python<'_>,
    buf: &mut String,
    items: &[ExprEnum],
) -> pyo3::PyResult<()> {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&item.python_repr(py)?);
    }
    Ok(())
}

impl WriteSql for ExprEnum {
    fn write_sql<W: SqlWriter>(&self, dialect: &SqlDialect, out: &mut W) -> PrepareResult<()> {
        match self {
            Self::Ident(ident) => ident.write_sql(dialect, out),
            Self::Value(value) => value.write_sql(dialect, out),
            Self::Tuple(items) => {
                out.push_char('(');
                write_separated(dialect, out, items, ", ")?;
                out.push_char(')');
                Ok(())
            }
            Self::Unary { op, expr } => {
                match op {
                    UnaryOp::Not => {
                        out.push_str("NOT ");
                    }
                    UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitNot => {
                        out.push_str(op.as_sql());
                    }
                }
                expr.write_operand(dialect, out, self.precedence(), true)
            }
            Self::Binary { left, op, right } => {
                let prec = self.precedence();
                left.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                out.push_str(op.as_sql());
                out.push_char(' ');
                right.write_operand(dialect, out, prec, true)
            }
            Self::Function(func) => func.write_sql(dialect, out),
            Self::Is {
                expr,
                negated,
                kind,
            } => {
                expr.write_operand(dialect, out, self.precedence(), false)?;
                if *negated {
                    out.push_str(" IS NOT ");
                } else {
                    out.push_str(" IS ");
                }
                out.push_str(kind.as_sql());
                Ok(())
            }
            Self::IsDistinctFrom {
                left,
                right,
                negated,
            } => {
                let prec = self.precedence();
                left.write_operand(dialect, out, prec, false)?;
                if *negated {
                    out.push_str(" IS NOT DISTINCT FROM ");
                } else {
                    out.push_str(" IS DISTINCT FROM ");
                }
                right.write_operand(dialect, out, prec, true)
            }
            Self::InList {
                expr,
                list,
                negated,
            } => {
                expr.write_operand(dialect, out, self.precedence(), false)?;
                if *negated {
                    out.push_str(" NOT IN (");
                } else {
                    out.push_str(" IN (");
                }
                write_separated(dialect, out, list, ", ")?;
                out.push_char(')');
                Ok(())
            }
            Self::Between {
                expr,
                negated,
                low,
                high,
            } => {
                let prec = self.precedence();
                expr.write_operand(dialect, out, prec, false)?;
                if *negated {
                    out.push_str(" NOT BETWEEN ");
                } else {
                    out.push_str(" BETWEEN ");
                }
                low.write_operand(dialect, out, prec, true)?;
                out.push_str(" AND ");
                high.write_operand(dialect, out, prec, true)
            }
            Self::Like {
                kind,
                negated,
                expr,
                pattern,
                escape,
            } => {
                let prec = self.precedence();
                expr.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                if *negated {
                    out.push_str("NOT ");
                }
                out.push_str(kind.as_sql());
                out.push_char(' ');
                pattern.write_operand(dialect, out, prec, true)?;
                if let Some(c) = escape {
                    out.push_str(" ESCAPE ");
                    let raw = c.encode_utf8(&mut [0; 4]).to_string();
                    dialect
                        .write_string_literal(out, &raw)
                        .map_err(|_| crate::sql::PrepareError::Write)?;
                }
                Ok(())
            }
            Self::Cast { expr, data_type } => {
                out.push_str("CAST(");
                expr.write_sql(dialect, out)?;
                out.push_str(" AS ");
                data_type.write_sql(dialect, out)?;
                out.push_char(')');
                Ok(())
            }
            Self::Case {
                operand,
                when_then,
                else_result,
            } => {
                out.push_str("CASE");
                if let Some(operand) = operand {
                    out.push_char(' ');
                    operand.write_sql(dialect, out)?;
                }
                for (when, then) in when_then {
                    out.push_str(" WHEN ");
                    when.write_sql(dialect, out)?;
                    out.push_str(" THEN ");
                    then.write_sql(dialect, out)?;
                }
                if let Some(else_result) = else_result {
                    out.push_str(" ELSE ");
                    else_result.write_sql(dialect, out)?;
                }
                out.push_str(" END");
                Ok(())
            }
            Self::Custom { sql, args } => write_custom(sql, args, dialect, out),
        }
    }
}

fn write_custom<W: SqlWriter>(
    sql: &str,
    args: &[ExprEnum],
    dialect: &SqlDialect,
    out: &mut W,
) -> PrepareResult<()> {
    let bytes = sql.as_bytes();
    let mut i = 0;
    let mut start = 0;
    let mut used = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                out.push_str(&sql[start..i]);
                out.push_char('{');
                i += 2;
                start = i;
                continue;
            }
            if i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                out.push_str(&sql[start..i]);
                if used >= args.len() {
                    return Err(crate::sql::PrepareError::CustomPlaceholders {
                        expected: used + 1,
                        got: args.len(),
                    });
                }
                args[used].write_sql(dialect, out)?;
                used += 1;
                i += 2;
                start = i;
                continue;
            }
        } else if bytes[i] == b'}'
            && i + 1 < bytes.len()
            && bytes[i + 1] == b'}'
        {
            out.push_str(&sql[start..i]);
            out.push_char('}');
            i += 2;
            start = i;
            continue;
        }
        i += 1;
    }
    out.push_str(&sql[start..]);
    if used != args.len() {
        return Err(crate::sql::PrepareError::CustomPlaceholders {
            expected: used,
            got: args.len(),
        });
    }
    Ok(())
}
