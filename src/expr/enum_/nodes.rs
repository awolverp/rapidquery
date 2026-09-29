use super::{BinaryOp, CastKind, ExprInner, Ident, IsKind, TrimWhere, UnaryOp, UnarySpacing};
use crate::dialect::PySqlDialect;
use crate::dialect::inner::{
    BitXorSyntax, CollateSyntax, IsDistinctFromSyntax, ModuloSyntax, StringConcatSyntax,
    TrimSyntax, TupleSyntax,
};
use crate::sql::{PrepareResult, SqlWriter, WriteSql, write_separated};
use std::fmt::Write;

#[inline]
fn unsupported(message: &'static str) -> crate::sql::PrepareError {
    rapidquery_macros::pyerr!(PyRuntimeError, message).into()
}

fn join_reprs(py: pyo3::Python<'_>, buf: &mut String, items: &[ExprInner]) -> pyo3::PyResult<()> {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(&item.python_repr(py)?);
    }
    Ok(())
}

/// Row value. SQL spelling depends on [`TupleSyntax`].
#[derive(Clone, Debug, PartialEq)]
pub struct Tuple {
    pub items: Vec<ExprInner>,
}

impl Tuple {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = String::from("TupleExpr(");
        join_reprs(py, &mut buf, &self.items)?;
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Tuple {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        let Some(syntax) = dialect.0.tuple_syntax else {
            return Err(unsupported(
                "tuple values are not supported by this dialect",
            ));
        };

        let arity = self.items.len();
        match syntax {
            TupleSyntax::Row => {
                out.push_str("ROW(");
            }
            TupleSyntax::RowRejectEmpty => {
                if arity == 0 {
                    return Err(unsupported("empty tuple is not supported by this dialect"));
                }
                out.push_str("ROW(");
            }
            TupleSyntax::Struct => {
                if arity == 0 {
                    return Err(unsupported("empty tuple is not supported by this dialect"));
                }
                out.push_str("struct(");
            }
            TupleSyntax::Parentheses => {
                if arity < 2 {
                    return Err(unsupported(
                        "a tuple needs at least two elements on this dialect",
                    ));
                }
                out.push_char('(');
            }
        }
        write_separated(dialect, out, &self.items, ", ")?;
        out.push_char(')');
        Ok(())
    }
}

/// Unary operation (`NOT x`, `-x`, `|/x`, postfix `x!`).
///
/// Precedence 1. Nested equal-precedence operands are parenthesized.
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<ExprInner>,
}

impl Unary {
    pub(crate) const PRECEDENCE: u8 = 1;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(format!(
            "UnaryExpr({:?}, {})",
            self.op.as_sql(),
            self.expr.python_repr(py)?
        ))
    }
}

impl WriteSql for Unary {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match self.op.spacing() {
            UnarySpacing::SpacedPrefix => {
                out.push_str(self.op.as_sql());
                out.push_char(' ');
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            UnarySpacing::GluedPrefix => {
                out.push_str(self.op.as_sql());
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
        }
    }
}

/// Binary operation (`a + b`, `a AND b`, custom operators).
///
/// `!=` is stored and emitted as `<>`. Precedence follows [`BinaryOp::precedence`].
/// The right operand is parenthesized when it has equal precedence.
#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    pub left: Box<ExprInner>,
    pub op: BinaryOp,
    pub right: Box<ExprInner>,
}

impl Binary {
    #[inline]
    pub(crate) fn precedence(&self) -> u8 {
        self.op.precedence()
    }

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(format!(
            "BinaryExpr({}, {:?}, {})",
            self.left.python_repr(py)?,
            self.op.as_sql(),
            self.right.python_repr(py)?
        ))
    }
}

#[inline]
fn is_arithmetic_binary_op(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod
    )
}

/// `name(left, right)` with an unquoted function name.
#[inline]
fn write_binary_call<W: SqlWriter>(
    dialect: &PySqlDialect,
    out: &mut W,
    name: &str,
    left: &ExprInner,
    right: &ExprInner,
) -> PrepareResult<()> {
    out.push_str(name);
    out.push_char('(');
    left.write_sql(dialect, out)?;
    out.push_str(", ");
    right.write_sql(dialect, out)?;
    out.push_char(')');
    Ok(())
}

/// Infix `left SPELLING right`.
///
/// `paren_arithmetic` parenthesizes a `+ - * / %` side. Infix `||` (and MsSql
/// `+` used as concat) needs that because dialect precedence disagrees with
/// the tree: SQLite binds `||` tighter than arithmetic, and MsSql `+` is the
/// same token as addition.
#[inline]
fn write_infix<W: SqlWriter>(
    bin: &Binary,
    dialect: &PySqlDialect,
    out: &mut W,
    spelling: &str,
    paren_arithmetic: bool,
) -> PrepareResult<()> {
    let prec = bin.precedence();
    write_infix_side(dialect, out, &bin.left, prec, false, paren_arithmetic)?;
    out.push_char(' ');
    out.push_str(spelling);
    out.push_char(' ');
    write_infix_side(dialect, out, &bin.right, prec, true, paren_arithmetic)
}

#[inline]
fn write_infix_side<W: SqlWriter>(
    dialect: &PySqlDialect,
    out: &mut W,
    expr: &ExprInner,
    prec: u8,
    paren_if_equal: bool,
    paren_arithmetic: bool,
) -> PrepareResult<()> {
    if paren_arithmetic && matches!(expr, ExprInner::Binary(b) if is_arithmetic_binary_op(&b.op)) {
        out.push_char('(');
        expr.write_sql(dialect, out)?;
        out.push_char(')');
        return Ok(());
    }
    expr.write_operand(dialect, out, prec, paren_if_equal)
}

impl WriteSql for Binary {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match &self.op {
            BinaryOp::StringConcat => match dialect.0.string_concat_syntax {
                StringConcatSyntax::ConcatFunction => {
                    write_binary_call(dialect, out, "CONCAT", &self.left, &self.right)
                }
                StringConcatSyntax::Pipes => write_infix(self, dialect, out, "||", true),
                StringConcatSyntax::Plus => write_infix(self, dialect, out, "+", true),
            },
            BinaryOp::BitXor => match dialect.0.bit_xor_syntax {
                None => Err(unsupported("bitwise XOR is not supported by this dialect")),
                Some(BitXorSyntax::Caret) => write_infix(self, dialect, out, "^", false),
                Some(BitXorSyntax::Hash) => write_infix(self, dialect, out, "#", false),
                Some(BitXorSyntax::XORFunction) => {
                    write_binary_call(dialect, out, "xor", &self.left, &self.right)
                }
                Some(BitXorSyntax::BitXORFunction) => {
                    write_binary_call(dialect, out, "BITXOR", &self.left, &self.right)
                }
            },
            BinaryOp::Mod => match dialect.0.modulo_syntax {
                ModuloSyntax::Percent => write_infix(self, dialect, out, "%", false),
                ModuloSyntax::Function => {
                    write_binary_call(dialect, out, "MOD", &self.left, &self.right)
                }
                ModuloSyntax::Infix => write_infix(self, dialect, out, "MOD", false),
            },
            _ => write_infix(self, dialect, out, self.op.as_sql(), false),
        }
    }
}

/// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
#[derive(Clone, Debug, PartialEq)]
pub struct Is {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub kind: IsKind,
}

impl Is {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "IsExpr({}, {:?}",
            self.expr.python_repr(py)?,
            self.kind.as_sql()
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Is {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" IS NOT ");
        } else {
            out.push_str(" IS ");
        }
        out.push_str(self.kind.as_sql());
        Ok(())
    }
}

/// `left IS [NOT] DISTINCT FROM right`.
#[derive(Clone, Debug, PartialEq)]
pub struct IsDistinctFrom {
    pub left: Box<ExprInner>,
    pub right: Box<ExprInner>,
    pub negated: bool,
}

impl IsDistinctFrom {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "IsDistinctFromExpr({}, {}",
            self.left.python_repr(py)?,
            self.right.python_repr(py)?
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for IsDistinctFrom {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.is_distinct_from_syntax {
            None => Err(unsupported(
                "IS DISTINCT FROM is not supported by this dialect",
            )),
            Some(IsDistinctFromSyntax::Standard) => {
                self.left
                    .write_operand(dialect, out, Self::PRECEDENCE, false)?;
                if self.negated {
                    out.push_str(" IS NOT DISTINCT FROM ");
                } else {
                    out.push_str(" IS DISTINCT FROM ");
                }
                self.right
                    .write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            Some(IsDistinctFromSyntax::NullSafeEqual) => {
                // `<=>` is null-safe equal, i.e. `IS NOT DISTINCT FROM`.
                if !self.negated {
                    out.push_str("NOT (");
                }
                self.left
                    .write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(" <=> ");
                self.right
                    .write_operand(dialect, out, Self::PRECEDENCE, true)?;
                if !self.negated {
                    out.push_char(')');
                }
                Ok(())
            }
        }
    }
}

/// `expr [NOT] IN (list…)`.
///
/// An empty list is `FALSE` (`NOT IN` is `TRUE`), spelled with the dialect's
/// boolean literal. The tested expression is not written.
#[derive(Clone, Debug, PartialEq)]
pub struct InList {
    pub expr: Box<ExprInner>,
    pub list: Vec<ExprInner>,
    pub negated: bool,
}

impl InList {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("InListExpr({}, (", self.expr.python_repr(py)?);
        join_reprs(py, &mut buf, &self.list)?;
        buf.push(')');
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for InList {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        if self.list.is_empty() {
            // `x IN ()` is false and `x NOT IN ()` is true, including when `x` is NULL.
            dialect.0.format_boolean(out, self.negated)?;
            return Ok(());
        }

        if dialect.0.parenthesize_in_between {
            out.push_char('(');
        }
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT IN (");
        } else {
            out.push_str(" IN (");
        }
        write_separated(dialect, out, &self.list, ", ")?;
        out.push_char(')');
        if dialect.0.parenthesize_in_between {
            out.push_char(')');
        }
        Ok(())
    }
}

/// `expr [NOT] BETWEEN low AND high`.
///
/// No `SYMMETRIC` clause.
#[derive(Clone, Debug, PartialEq)]
pub struct Between {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub low: Box<ExprInner>,
    pub high: Box<ExprInner>,
}

impl Between {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "BetweenExpr({}, {}, {}",
            self.expr.python_repr(py)?,
            self.low.python_repr(py)?,
            self.high.python_repr(py)?
        );
        if self.negated {
            buf.push_str(", negated=True");
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Between {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        if dialect.0.parenthesize_in_between {
            out.push_char('(');
        }
        self.expr
            .write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT BETWEEN ");
        } else {
            out.push_str(" BETWEEN ");
        }
        self.low
            .write_operand(dialect, out, Self::PRECEDENCE, true)?;
        out.push_str(" AND ");
        self.high
            .write_operand(dialect, out, Self::PRECEDENCE, true)?;
        if dialect.0.parenthesize_in_between {
            out.push_char(')');
        }
        Ok(())
    }
}

/// `CAST(expr AS type)`, `TRY_CAST(expr AS type)`, or `(expr)::type`.
///
/// [`kind`](Self::kind) stays `CAST` / `TRY_CAST` / `::`. BigQuery spells
/// `TRY_CAST` as `SAFE_CAST`; Teradata spells it as `TRYCAST`.
#[derive(Clone, Debug, PartialEq)]
pub struct Cast {
    pub kind: CastKind,
    pub expr: Box<ExprInner>,
    pub data_type_name: crate::data_type::DataTypeName,
}

impl Cast {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!(
            "CastExpr({}, {:?}",
            self.expr.python_repr(py)?,
            self.data_type_name.as_str()
        );
        if self.kind != CastKind::Cast {
            write!(buf, ", kind={:?}", self.kind.as_sql()).unwrap();
        }
        buf.push(')');
        Ok(buf)
    }

    fn write_call<W: SqlWriter>(
        &self,
        dialect: &PySqlDialect,
        out: &mut W,
        name: &str,
    ) -> PrepareResult<()> {
        out.push_str(name);
        out.push_char('(');
        self.expr.write_sql(dialect, out)?;
        out.push_str(" AS ");
        self.data_type_name.write_sql(dialect, out)?;
        out.push_char(')');
        Ok(())
    }
}

impl WriteSql for Cast {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match self.kind {
            CastKind::Cast => self.write_call(dialect, out, "CAST"),
            CastKind::TryCast => match dialect.0.try_cast_syntax {
                Some(syntax) => self.write_call(dialect, out, syntax.as_sql()),
                None => Err(unsupported("try cast is not supported by this dialect")),
            },
            CastKind::DoubleColon => {
                if !dialect.0.supports_double_colon {
                    return Err(unsupported(
                        "double-colon cast is not supported by this dialect",
                    ));
                }
                out.push_char('(');
                self.expr.write_sql(dialect, out)?;
                out.push_str(")::");
                self.data_type_name.write_sql(dialect, out)?;
                Ok(())
            }
        }
    }
}

/// `TRIM` of [`expr`](Self::expr), optionally from one side, optionally of
/// [`characters`](Self::characters).
///
/// [`where_`](Self::where_) defaults to both ends. The SQL spelling is
/// `TRIM([LEADING|TRAILING] [chars FROM] expr)` or `TRIM` / `LTRIM` / `RTRIM`.
#[derive(Clone, Debug, PartialEq)]
pub struct Trim {
    pub expr: Box<ExprInner>,
    pub where_: TrimWhere,
    pub characters: Option<Box<ExprInner>>,
}

impl Trim {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("TrimExpr({}", self.expr.python_repr(py)?);
        if self.where_ != TrimWhere::Both {
            write!(buf, ", where={:?}", self.where_.as_sql()).unwrap();
        }
        if let Some(characters) = &self.characters {
            write!(buf, ", characters={}", characters.python_repr(py)?).unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

#[inline]
fn write_trim_function<W: SqlWriter>(
    trim: &Trim,
    dialect: &PySqlDialect,
    out: &mut W,
) -> PrepareResult<()> {
    let name = match trim.where_ {
        TrimWhere::Both => "TRIM",
        TrimWhere::Leading => "LTRIM",
        TrimWhere::Trailing => "RTRIM",
    };
    out.push_str(name);
    out.push_char('(');
    trim.expr.write_sql(dialect, out)?;
    if let Some(characters) = &trim.characters {
        out.push_str(", ");
        characters.write_sql(dialect, out)?;
    }
    out.push_char(')');
    Ok(())
}

impl WriteSql for Trim {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.trim_syntax {
            TrimSyntax::SqlFrom => {
                out.push_str("TRIM(");
                match (&self.where_, &self.characters) {
                    (TrimWhere::Both, None) => self.expr.write_sql(dialect, out)?,
                    (TrimWhere::Both, Some(characters)) => {
                        characters.write_sql(dialect, out)?;
                        out.push_str(" FROM ");
                        self.expr.write_sql(dialect, out)?;
                    }
                    (side, None) => {
                        out.push_str(side.as_sql());
                        out.push_str(" FROM ");
                        self.expr.write_sql(dialect, out)?;
                    }
                    (side, Some(characters)) => {
                        out.push_str(side.as_sql());
                        out.push_char(' ');
                        characters.write_sql(dialect, out)?;
                        out.push_str(" FROM ");
                        self.expr.write_sql(dialect, out)?;
                    }
                }
                out.push_char(')');
                Ok(())
            }
            TrimSyntax::Function => write_trim_function(self, dialect, out),
            TrimSyntax::SpacesOnly => {
                if self.characters.is_some() {
                    return Err(unsupported(
                        "trim characters are not supported by this dialect",
                    ));
                }
                write_trim_function(self, dialect, out)
            }
        }
    }
}

/// `expr COLLATE collation`.
///
/// Precedence 0, so a binary operand is parenthesized.
#[derive(Clone, Debug, PartialEq)]
pub struct Collate {
    pub expr: Box<ExprInner>,
    pub collation: Ident,
}

impl Collate {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        Ok(format!(
            "CollateExpr({}, {})",
            self.expr.python_repr(py)?,
            self.collation.python_repr()
        ))
    }
}

impl WriteSql for Collate {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.collate_syntax {
            None => Err(unsupported("COLLATE is not supported by this dialect")),
            Some(CollateSyntax::InfixIdent) => {
                self.expr
                    .write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(" COLLATE ");
                self.collation.write_sql(dialect, out)
            }
            Some(CollateSyntax::FunctionString) => {
                let body = match self.collation.parts() {
                    [super::IdentPart::Quote(body) | super::IdentPart::Unquote(body)]
                        if !body.is_empty() =>
                    {
                        body.as_str()
                    }
                    _ => {
                        return Err(unsupported("collation must be a single name"));
                    }
                };
                out.push_str("COLLATE(");
                self.expr.write_sql(dialect, out)?;
                out.push_str(", ");
                dialect.0.write_string_literal(out, body)?;
                out.push_char(')');
                Ok(())
            }
        }
    }
}

/// `CASE [operand] WHEN … THEN … [ELSE …] END`.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    pub operand: Option<Box<ExprInner>>,
    pub when: Vec<(ExprInner, ExprInner)>,
    pub else_result: Option<Box<ExprInner>>,
}

impl Case {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = String::from("CaseExpr([");
        for (i, (when, then)) in self.when.iter().enumerate() {
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
        if let Some(operand) = &self.operand {
            write!(buf, ", operand={}", operand.python_repr(py)?).unwrap();
        }
        if let Some(else_result) = &self.else_result {
            write!(buf, ", else_={}", else_result.python_repr(py)?).unwrap();
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Case {
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        out.push_str("CASE");
        if let Some(operand) = &self.operand {
            out.push_char(' ');
            operand.write_sql(dialect, out)?;
        }
        for (when, then) in &self.when {
            out.push_str(" WHEN ");
            when.write_sql(dialect, out)?;
            out.push_str(" THEN ");
            then.write_sql(dialect, out)?;
        }
        if let Some(else_result) = &self.else_result {
            out.push_str(" ELSE ");
            else_result.write_sql(dialect, out)?;
        }
        out.push_str(" END");
        Ok(())
    }
}

/// Escape-hatch SQL template. `{}` slots are filled by argument expressions.
/// `{{` / `}}` emit literal braces. Slot count must match [`args`](Self::args).
#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    pub sql: arcstr::ArcStr,
    pub args: Vec<ExprInner>,
}

impl Custom {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
        let mut buf = format!("CustomExpr({:?}", self.sql);
        for arg in &self.args {
            buf.push_str(", ");
            buf.push_str(&arg.python_repr(py)?);
        }
        buf.push(')');
        Ok(buf)
    }
}

impl WriteSql for Custom {
    #[cfg_attr(feature = "nightly", optimize(speed))]
    fn write_sql<W: SqlWriter>(&self, dialect: &PySqlDialect, out: &mut W) -> PrepareResult<()> {
        let bytes = self.sql.as_bytes();
        let mut i = 0;
        let mut start = 0;
        let mut used = 0usize;

        while i < bytes.len() {
            if bytes[i] == b'{' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    out.push_str(&self.sql[start..i]);
                    out.push_char('{');
                    i += 2;
                    start = i;
                    continue;
                }
                if i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                    out.push_str(&self.sql[start..i]);
                    if used >= self.args.len() {
                        return Err(crate::sql::PrepareError::CustomPlaceholders {
                            expected: used + 1,
                            got: self.args.len(),
                        });
                    }
                    self.args[used].write_sql(dialect, out)?;
                    used += 1;
                    i += 2;
                    start = i;
                    continue;
                }
            } else if bytes[i] == b'}' && i + 1 < bytes.len() && bytes[i + 1] == b'}' {
                out.push_str(&self.sql[start..i]);
                out.push_char('}');
                i += 2;
                start = i;
                continue;
            }
            i += 1;
        }
        out.push_str(&self.sql[start..]);

        if used != self.args.len() {
            return Err(crate::sql::PrepareError::CustomPlaceholders {
                expected: used,
                got: self.args.len(),
            });
        }
        Ok(())
    }
}
