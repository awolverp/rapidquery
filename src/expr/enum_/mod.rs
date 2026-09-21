mod func;
mod ident;
mod operator;
mod value;

pub use func::*;
pub use ident::*;
pub use operator::*;
pub use value::*;

use crate::dialect::{LikeAnySyntax, SqlDialect};
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<ExprEnum>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    pub left: Box<ExprEnum>,
    pub op: BinaryOp,
    pub right: Box<ExprEnum>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Is {
    pub expr: Box<ExprEnum>,
    pub negated: bool,
    pub kind: IsKind,
}

/// `expr IS [NOT] [form] NORMALIZED`.
#[derive(Clone, Debug, PartialEq)]
pub struct IsNormalized {
    pub expr: Box<ExprEnum>,
    pub negated: bool,
    pub form: Option<NormalizationForm>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IsDistinctFrom {
    pub left: Box<ExprEnum>,
    pub right: Box<ExprEnum>,
    pub negated: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InList {
    pub expr: Box<ExprEnum>,
    pub list: Vec<ExprEnum>,
    pub negated: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Between {
    pub expr: Box<ExprEnum>,
    pub negated: bool,
    pub low: Box<ExprEnum>,
    pub high: Box<ExprEnum>,
}

/// Pattern side of [`Like`]: one expression, or a non-empty ANY list.
#[derive(Clone, Debug, PartialEq)]
pub enum LikePattern {
    /// `col LIKE p`
    One(Box<ExprEnum>),
    /// `col LIKE ANY …` — never empty.
    Any(Vec<ExprEnum>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Like {
    pub case_sensitive: bool,
    pub negated: bool,
    pub expr: Box<ExprEnum>,
    pub pattern: LikePattern,
    pub escape: Option<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimilarTo {
    pub negated: bool,
    pub expr: Box<ExprEnum>,
    pub pattern: Box<ExprEnum>,
    pub escape: Option<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RLike {
    pub negated: bool,
    pub expr: Box<ExprEnum>,
    pub pattern: Box<ExprEnum>,
    pub escape: Option<char>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cast {
    pub expr: Box<ExprEnum>,
    pub data_type: crate::data_type::DataTypeName,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    pub operand: Option<Box<ExprEnum>>,
    pub when: Vec<(ExprEnum, ExprEnum)>,
    pub else_result: Option<Box<ExprEnum>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    pub sql: arcstr::ArcStr,
    pub args: Vec<ExprEnum>,
}

/// SQL expression tree.
///
/// TODO: `Exists`, `InSubquery`, `Subquery`, and `Quantified` (`ANY` /
/// `ALL` / `SOME`) once `Query` exists.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprEnum {
    /// Represents a SQL identifier
    Ident(Ident),
    /// A Python object plus the Serializer used to emit it as SQL.
    Value(Value),
    /// Represents a SQL tuple
    Tuple(Vec<ExprEnum>),
    /// Unary operation
    Unary(Unary),
    /// Binary operation
    Binary(Binary),
    /// Function
    Function(Function),
    /// IS [NOT] <kind>
    Is(Is),
    /// IS [NOT] [form] NORMALIZED
    IsNormalized(IsNormalized),
    /// <left> IS [NOT] DISTINCT FROM <right>
    IsDistinctFrom(IsDistinctFrom),
    /// <expr> [NOT] IN <list>
    InList(InList),
    /// <expr> [NOT] BETWEEN <low> AND <high>
    Between(Between),
    Like(Like),
    SimilarTo(SimilarTo),
    RLike(RLike),
    Cast(Cast),
    Case(Case),
    Custom(Custom),
}

fn join_reprs(py: pyo3::Python<'_>, buf: &mut String, items: &[ExprEnum]) -> pyo3::PyResult<()> {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&item.python_repr(py)?);
    }
    Ok(())
}

impl From<Ident> for ExprEnum {
    #[inline]
    fn from(value: Ident) -> Self {
        Self::Ident(value)
    }
}

impl From<Value> for ExprEnum {
    #[inline]
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

impl From<Unary> for ExprEnum {
    #[inline]
    fn from(value: Unary) -> Self {
        Self::Unary(value)
    }
}

impl From<Binary> for ExprEnum {
    #[inline]
    fn from(value: Binary) -> Self {
        Self::Binary(value)
    }
}

impl From<Function> for ExprEnum {
    #[inline]
    fn from(value: Function) -> Self {
        Self::Function(value)
    }
}

impl From<Is> for ExprEnum {
    #[inline]
    fn from(value: Is) -> Self {
        Self::Is(value)
    }
}

impl From<IsNormalized> for ExprEnum {
    #[inline]
    fn from(value: IsNormalized) -> Self {
        Self::IsNormalized(value)
    }
}

impl From<IsDistinctFrom> for ExprEnum {
    #[inline]
    fn from(value: IsDistinctFrom) -> Self {
        Self::IsDistinctFrom(value)
    }
}

impl From<InList> for ExprEnum {
    #[inline]
    fn from(value: InList) -> Self {
        Self::InList(value)
    }
}

impl From<Between> for ExprEnum {
    #[inline]
    fn from(value: Between) -> Self {
        Self::Between(value)
    }
}

impl From<Like> for ExprEnum {
    #[inline]
    fn from(value: Like) -> Self {
        Self::Like(value)
    }
}

impl From<SimilarTo> for ExprEnum {
    #[inline]
    fn from(value: SimilarTo) -> Self {
        Self::SimilarTo(value)
    }
}

impl From<RLike> for ExprEnum {
    #[inline]
    fn from(value: RLike) -> Self {
        Self::RLike(value)
    }
}

impl From<Cast> for ExprEnum {
    #[inline]
    fn from(value: Cast) -> Self {
        Self::Cast(value)
    }
}

impl From<Case> for ExprEnum {
    #[inline]
    fn from(value: Case) -> Self {
        Self::Case(value)
    }
}

impl From<Custom> for ExprEnum {
    #[inline]
    fn from(value: Custom) -> Self {
        Self::Custom(value)
    }
}

impl ExprEnum {
    /// `true` when this node can appear as a SQL condition (`WHERE`,
    /// `WHEN`, `FILTER`). Identifiers and bound values are not conditions.
    #[inline]
    pub(crate) fn is_condition(&self) -> bool {
        !matches!(self, Self::Ident(_) | Self::Value(_))
    }

    /// Lower is tighter binding.
    #[inline]
    fn precedence(&self) -> u8 {
        match self {
            Self::Ident(_)
            | Self::Value(_)
            | Self::Tuple(_)
            | Self::Function(_)
            | Self::Cast(_)
            | Self::Case(_)
            | Self::Custom(_) => 0,
            Self::Unary(_) => 1,
            Self::Binary(Binary { op, .. }) => match op {
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
            Self::Is(_)
            | Self::IsNormalized(_)
            | Self::IsDistinctFrom(_)
            | Self::InList(_)
            | Self::Between(_)
            | Self::Like(_)
            | Self::SimilarTo(_)
            | Self::RLike(_) => 5,
        }
    }

    #[inline]
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

    #[inline]
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        match self {
            Self::Ident(ident) => Ok(ident.python_repr()),
            Self::Value(value) => value.python_repr(py),
            Self::Tuple(items) => {
                let mut buf = String::from("TupleExpr(");
                join_reprs(py, &mut buf, items)?;
                buf.push(')');
                Ok(buf)
            }
            Self::Unary(Unary { op, expr }) => Ok(format!(
                "UnaryExpr({:?}, {})",
                op.as_sql(),
                expr.python_repr(py)?
            )),
            Self::Binary(Binary { left, op, right }) => Ok(format!(
                "BinaryExpr({}, {:?}, {})",
                left.python_repr(py)?,
                op.as_sql(),
                right.python_repr(py)?
            )),
            Self::Function(func) => func.python_repr(py),
            Self::Is(Is {
                expr,
                negated,
                kind,
            }) => {
                let mut buf = format!("IsExpr({}, {:?}", expr.python_repr(py)?, kind.as_sql());
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::IsNormalized(IsNormalized {
                expr,
                negated,
                form,
            }) => {
                let mut buf = format!("IsNormalizedExpr({}", expr.python_repr(py)?);
                if let Some(form) = form {
                    write!(buf, ", {:?}", form.as_sql()).unwrap();
                }
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::IsDistinctFrom(IsDistinctFrom {
                left,
                right,
                negated,
            }) => {
                let mut buf = format!(
                    "IsDistinctFromExpr({}, {}",
                    left.python_repr(py)?,
                    right.python_repr(py)?
                );
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::InList(InList {
                expr,
                list,
                negated,
            }) => {
                let mut buf = format!("InListExpr({}, (", expr.python_repr(py)?);
                join_reprs(py, &mut buf, list)?;
                buf.push(')');
                if *negated {
                    buf.push_str(", negated=True");
                }
                buf.push(')');
                Ok(buf)
            }
            Self::Between(Between {
                expr,
                negated,
                low,
                high,
            }) => {
                let mut buf = format!(
                    "BetweenExpr({}, {}, {}",
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
            Self::Like(Like {
                case_sensitive,
                negated,
                expr,
                pattern,
                escape,
            }) => {
                let mut buf = format!("LikeExpr({}, ", expr.python_repr(py)?);
                match pattern {
                    LikePattern::One(p) => buf.push_str(&p.python_repr(py)?),
                    LikePattern::Any(ps) => {
                        buf.push('[');
                        join_reprs(py, &mut buf, ps)?;
                        buf.push(']');
                    }
                }
                if !*case_sensitive {
                    buf.push_str(", case_sensitive=False");
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
            Self::SimilarTo(SimilarTo {
                negated,
                expr,
                pattern,
                escape,
            }) => {
                let mut buf = format!(
                    "SimilarToExpr({}, {}",
                    expr.python_repr(py)?,
                    pattern.python_repr(py)?
                );
                if *negated {
                    buf.push_str(", negated=True");
                }
                if let Some(c) = escape {
                    write!(buf, ", escape={c:?}").unwrap();
                }
                buf.push(')');
                Ok(buf)
            }
            Self::RLike(RLike {
                negated,
                expr,
                pattern,
                escape,
            }) => {
                let mut buf = format!(
                    "RLikeExpr({}, {}",
                    expr.python_repr(py)?,
                    pattern.python_repr(py)?
                );
                if *negated {
                    buf.push_str(", negated=True");
                }
                if let Some(c) = escape {
                    write!(buf, ", escape={c:?}").unwrap();
                }
                buf.push(')');
                Ok(buf)
            }
            Self::Cast(Cast { expr, data_type }) => Ok(format!(
                "CastExpr({}, {:?})",
                expr.python_repr(py)?,
                data_type.as_str(),
            )),
            Self::Case(Case {
                operand,
                when: when_then,
                else_result,
            }) => {
                let mut buf = String::from("CaseExpr([");
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
            Self::Custom(Custom { sql, args }) => {
                let mut buf = format!("CustomExpr({sql:?}");
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

fn write_like_op<W: SqlWriter>(out: &mut W, case_sensitive: bool, negated: bool) {
    if negated {
        out.push_str("NOT ");
    }
    if case_sensitive {
        out.push_str("LIKE");
    } else {
        out.push_str("ILIKE");
    }
}

fn write_escape<W: SqlWriter>(
    dialect: &SqlDialect,
    out: &mut W,
    escape: Option<char>,
) -> PrepareResult<()> {
    let Some(c) = escape else {
        return Ok(());
    };
    out.push_str(" ESCAPE ");
    let mut buf = [0u8; 4];
    dialect
        .write_string_literal(out, c.encode_utf8(&mut buf))
        .map_err(|_| crate::sql::PrepareError::Write)?;
    Ok(())
}

fn write_like<W: SqlWriter>(
    dialect: &SqlDialect,
    out: &mut W,
    like: &Like,
    prec: u8,
) -> PrepareResult<()> {
    match &like.pattern {
        LikePattern::One(pattern) => {
            like.expr.write_operand(dialect, out, prec, false)?;
            out.push_char(' ');
            write_like_op(out, like.case_sensitive, like.negated);
            out.push_char(' ');
            pattern.write_operand(dialect, out, prec, true)?;
            write_escape(dialect, out, like.escape)
        }
        LikePattern::Any(patterns) => match dialect.like_any_syntax() {
            LikeAnySyntax::ArrayConstructor => {
                // `NOT LIKE ANY` in Postgres is a quantified NOT LIKE
                // (exists a non-match), not "matches none".
                if like.negated {
                    out.push_str("NOT (");
                }
                like.expr.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                write_like_op(out, like.case_sensitive, false);
                out.push_str(" ANY (ARRAY[");
                write_separated(dialect, out, patterns, ", ")?;
                out.push_str("])");
                write_escape(dialect, out, like.escape)?;
                if like.negated {
                    out.push_char(')');
                }
                Ok(())
            }
            LikeAnySyntax::PatternList => {
                like.expr.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                write_like_op(out, like.case_sensitive, like.negated);
                out.push_str(" ANY (");
                write_separated(dialect, out, patterns, ", ")?;
                out.push_char(')');
                write_escape(dialect, out, like.escape)
            }
            LikeAnySyntax::OrExpansion => {
                out.push_char('(');
                let sep = if like.negated { " AND " } else { " OR " };
                for (i, pattern) in patterns.iter().enumerate() {
                    if i > 0 {
                        out.push_str(sep);
                    }
                    like.expr.write_operand(dialect, out, prec, false)?;
                    out.push_char(' ');
                    write_like_op(out, like.case_sensitive, like.negated);
                    out.push_char(' ');
                    pattern.write_operand(dialect, out, prec, true)?;
                    write_escape(dialect, out, like.escape)?;
                }
                out.push_char(')');
                Ok(())
            }
        },
    }
}

impl WriteSql for ExprEnum {
    #[cfg_attr(feature = "nightly", optimize(speed))]
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
            Self::Unary(Unary { op, expr }) => match op {
                UnaryOp::PGPostfixFactorial => {
                    expr.write_operand(dialect, out, self.precedence(), true)?;
                    out.push_char('!');
                    Ok(())
                }
                UnaryOp::Not
                | UnaryOp::Hash
                | UnaryOp::AtDashAt
                | UnaryOp::DoubleAt
                | UnaryOp::QuestionDash
                | UnaryOp::QuestionPipe => {
                    out.push_str(op.as_sql());
                    out.push_char(' ');
                    expr.write_operand(dialect, out, self.precedence(), true)
                }
                UnaryOp::Plus
                | UnaryOp::Minus
                | UnaryOp::BitNot
                | UnaryOp::PGAbs
                | UnaryOp::PGSquareRoot
                | UnaryOp::PGCubeRoot
                | UnaryOp::PGPrefixFactorial => {
                    out.push_str(op.as_sql());
                    expr.write_operand(dialect, out, self.precedence(), true)
                }
            },
            Self::Binary(Binary { left, op, right }) => {
                let prec = self.precedence();
                left.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                out.push_str(op.as_sql());
                out.push_char(' ');
                right.write_operand(dialect, out, prec, true)
            }
            Self::Function(func) => func.write_sql(dialect, out),
            Self::Is(Is {
                expr,
                negated,
                kind,
            }) => {
                expr.write_operand(dialect, out, self.precedence(), false)?;
                if *negated {
                    out.push_str(" IS NOT ");
                } else {
                    out.push_str(" IS ");
                }
                out.push_str(kind.as_sql());
                Ok(())
            }
            Self::IsNormalized(IsNormalized {
                expr,
                negated,
                form,
            }) => {
                expr.write_operand(dialect, out, self.precedence(), false)?;
                if *negated {
                    out.push_str(" IS NOT ");
                } else {
                    out.push_str(" IS ");
                }
                if let Some(form) = form {
                    out.push_str(form.as_sql());
                    out.push_char(' ');
                }
                out.push_str("NORMALIZED");
                Ok(())
            }
            Self::IsDistinctFrom(IsDistinctFrom {
                left,
                right,
                negated,
            }) => {
                let prec = self.precedence();
                left.write_operand(dialect, out, prec, false)?;
                if *negated {
                    out.push_str(" IS NOT DISTINCT FROM ");
                } else {
                    out.push_str(" IS DISTINCT FROM ");
                }
                right.write_operand(dialect, out, prec, true)
            }
            Self::InList(InList {
                expr,
                list,
                negated,
            }) => {
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
            Self::Between(Between {
                expr,
                negated,
                low,
                high,
            }) => {
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
            Self::Like(like) => write_like(dialect, out, like, self.precedence()),
            Self::SimilarTo(SimilarTo {
                negated,
                expr,
                pattern,
                escape,
            }) => {
                let prec = self.precedence();
                expr.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                if *negated {
                    out.push_str("NOT SIMILAR TO ");
                } else {
                    out.push_str("SIMILAR TO ");
                }

                pattern.write_operand(dialect, out, prec, true)?;
                write_escape(dialect, out, *escape)
            }
            Self::RLike(RLike {
                negated,
                expr,
                pattern,
                escape,
            }) => {
                let prec = self.precedence();
                expr.write_operand(dialect, out, prec, false)?;
                out.push_char(' ');
                if *negated {
                    out.push_str("NOT RLIKE ");
                } else {
                    out.push_str("RLIKE ");
                }

                pattern.write_operand(dialect, out, prec, true)?;
                write_escape(dialect, out, *escape)
            }
            Self::Cast(Cast { expr, data_type }) => {
                out.push_str("CAST(");
                expr.write_sql(dialect, out)?;
                out.push_str(" AS ");
                data_type.write_sql(dialect, out)?;
                out.push_char(')');
                Ok(())
            }
            Self::Case(Case {
                operand,
                when: when_then,
                else_result,
            }) => {
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
            Self::Custom(Custom { sql, args }) => write_custom(sql, args, dialect, out),
        }
    }
}

#[cfg_attr(feature = "nightly", optimize(speed))]
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
        } else if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
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
