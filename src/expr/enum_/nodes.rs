use super::BinaryOp;
use super::CastKind;
use super::ExprInner;
use super::Ident;
use super::IsKind;
use super::TrimWhere;
use super::UnaryOp;
use super::UnarySpacing;
use crate::dialect::PyDialect;
use crate::dialect::flags;
use crate::internal::repr::ReprFormatter;
use crate::sql::PrepareResult;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;
use crate::sql::write_separated;

#[inline]
fn unsupported(message: &'static str) -> crate::sql::PrepareError {
    rapidquery_macros::pyerr!(PyRuntimeError, message).into()
}

/// Row value. SQL spelling depends on [`TupleSyntax`].
#[derive(Clone, Debug, PartialEq)]
pub struct Tuple {
    pub items: Vec<ExprInner>,
}

impl Tuple {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("TupleExpr");
        for item in &self.items {
            fmt.positional_formatter(item.python_repr(py)?);
        }
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Tuple {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        let arity = self.items.len();
        match dialect.0.tuple() {
            flags::Tuple::Unsupported => {
                return Err(unsupported("tuple values are not supported by this dialect"));
            }
            flags::Tuple::Row => {
                out.push_str("ROW(");
            }
            flags::Tuple::RowRejectEmpty => {
                if arity == 0 {
                    return Err(unsupported("empty tuple is not supported by this dialect"));
                }
                out.push_str("ROW(");
            }
            flags::Tuple::Struct => {
                if arity == 0 {
                    return Err(unsupported("empty tuple is not supported by this dialect"));
                }
                out.push_str("struct(");
            }
            flags::Tuple::Parentheses => {
                if arity < 2 {
                    return Err(unsupported("a tuple needs at least two elements on this dialect"));
                }
                out.push_char('(');
            }
        }
        write_separated(dialect, out, &self.items, ", ")?;
        out.push_char(')');
        Ok(())
    }
}

/// Unary operation (`NOT x`, `-x`, `~x`).
///
/// `NOT` binds between comparisons and `AND`. `+`, `-`, and `~` bind tighter
/// than multiplication. Nested equal-precedence operands are parenthesized.
/// `~` is `BITNOT(expr)` where the dialect's bitwise profile says so.
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<ExprInner>,
}

impl Unary {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("UnaryExpr");
        fmt.positional_debug(self.op.as_sql());
        fmt.positional_formatter(self.expr.python_repr(py)?);
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Unary {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        if self.op == UnaryOp::BitNot {
            match dialect.0.bitwise() {
                flags::Bitwise::Infix | flags::Bitwise::InfixNoShift => {
                    out.push_char('~');
                    return self
                        .expr
                        .write_operand(dialect, out, UnaryOp::BitNot.precedence(), true);
                }
                flags::Bitwise::BitShiftFunctions | flags::Bitwise::ShiftFunctions => {
                    out.push_str("BITNOT(");
                    self.expr.write_sql(dialect, out)?;
                    out.push_char(')');
                    return Ok(());
                }
                flags::Bitwise::BitAndOnly | flags::Bitwise::Unsupported | flags::Bitwise::InfixAndShift => {
                    return Err(unsupported("bitwise NOT is not supported by this dialect"));
                }
            }
        }
        let prec = self.op.precedence();
        match self.op.spacing() {
            UnarySpacing::SpacedPrefix => {
                out.push_str(self.op.as_sql());
                out.push_char(' ');
                self.expr.write_operand(dialect, out, prec, true)
            }
            UnarySpacing::GluedPrefix => {
                out.push_str(self.op.as_sql());
                self.expr.write_operand(dialect, out, prec, true)
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
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("BinaryExpr");
        fmt.positional_formatter(self.left.python_repr(py)?);
        fmt.positional_debug(self.op.as_sql());
        fmt.positional_formatter(self.right.python_repr(py)?);
        fmt.close();
        Ok(fmt)
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
    dialect: &PyDialect,
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
    dialect: &PyDialect,
    out: &mut W,
    spelling: &str,
    paren_arithmetic: bool,
) -> PrepareResult<()> {
    let prec = bin.op.precedence();
    write_infix_side(dialect, out, &bin.left, prec, false, paren_arithmetic)?;
    out.push_char(' ');
    out.push_str(spelling);
    out.push_char(' ');
    write_infix_side(dialect, out, &bin.right, prec, true, paren_arithmetic)
}

#[inline]
fn write_infix_side<W: SqlWriter>(
    dialect: &PyDialect,
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
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match &self.op {
            BinaryOp::StringConcat => match dialect.0.string_concat() {
                flags::StringConcat::ConcatFunction => {
                    write_binary_call(dialect, out, "CONCAT", &self.left, &self.right)
                }
                flags::StringConcat::Pipes => write_infix(self, dialect, out, "||", true),
                flags::StringConcat::Plus => write_infix(self, dialect, out, "+", true),
            },
            BinaryOp::BitXor => match dialect.0.bitxor() {
                flags::BitXor::Unsupported => Err(unsupported("bitwise XOR is not supported by this dialect")),
                flags::BitXor::Caret => write_infix(self, dialect, out, "^", false),
                flags::BitXor::Hash => write_infix(self, dialect, out, "#", false),
                flags::BitXor::XORFunction => write_binary_call(dialect, out, "xor", &self.left, &self.right),
                flags::BitXor::BitXORFunction => write_binary_call(dialect, out, "BITXOR", &self.left, &self.right),
            },
            BinaryOp::Mod => match dialect.0.modulo() {
                flags::Modulo::Percent => write_infix(self, dialect, out, "%", false),
                flags::Modulo::Function => write_binary_call(dialect, out, "MOD", &self.left, &self.right),
                flags::Modulo::Infix => write_infix(self, dialect, out, "MOD", false),
            },
            BinaryOp::BitAnd => match dialect.0.bitwise() {
                flags::Bitwise::Infix | flags::Bitwise::InfixNoShift | flags::Bitwise::InfixAndShift => {
                    write_infix(self, dialect, out, "&", false)
                }
                flags::Bitwise::BitShiftFunctions | flags::Bitwise::ShiftFunctions | flags::Bitwise::BitAndOnly => {
                    write_binary_call(dialect, out, "BITAND", &self.left, &self.right)
                }
                flags::Bitwise::Unsupported => Err(unsupported("bitwise AND is not supported by this dialect")),
            },
            BinaryOp::BitOr => match dialect.0.bitwise() {
                flags::Bitwise::Infix | flags::Bitwise::InfixNoShift => write_infix(self, dialect, out, "|", false),
                flags::Bitwise::BitShiftFunctions | flags::Bitwise::ShiftFunctions => {
                    write_binary_call(dialect, out, "BITOR", &self.left, &self.right)
                }
                flags::Bitwise::BitAndOnly | flags::Bitwise::Unsupported | flags::Bitwise::InfixAndShift => {
                    Err(unsupported("bitwise OR is not supported by this dialect"))
                }
            },
            BinaryOp::LShift => match dialect.0.bitwise() {
                flags::Bitwise::Infix | flags::Bitwise::InfixAndShift => write_infix(self, dialect, out, "<<", false),
                flags::Bitwise::BitShiftFunctions => {
                    write_binary_call(dialect, out, "BITSHIFTLEFT", &self.left, &self.right)
                }
                flags::Bitwise::ShiftFunctions => write_binary_call(dialect, out, "SHIFTLEFT", &self.left, &self.right),
                flags::Bitwise::InfixNoShift | flags::Bitwise::BitAndOnly | flags::Bitwise::Unsupported => {
                    Err(unsupported("bitwise shift is not supported by this dialect"))
                }
            },
            BinaryOp::RShift => match dialect.0.bitwise() {
                flags::Bitwise::Infix | flags::Bitwise::InfixAndShift => write_infix(self, dialect, out, ">>", false),
                flags::Bitwise::BitShiftFunctions => {
                    write_binary_call(dialect, out, "BITSHIFTRIGHT", &self.left, &self.right)
                }
                flags::Bitwise::ShiftFunctions => {
                    write_binary_call(dialect, out, "SHIFTRIGHT", &self.left, &self.right)
                }
                flags::Bitwise::InfixNoShift | flags::Bitwise::BitAndOnly | flags::Bitwise::Unsupported => {
                    Err(unsupported("bitwise shift is not supported by this dialect"))
                }
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("IsExpr");
        fmt.positional_formatter(self.expr.python_repr(py)?);
        fmt.positional_debug(self.kind.as_sql());
        fmt.keyword_if_true("negated", self.negated);
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Is {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("IsDistinctFromExpr");
        fmt.positional_formatter(self.left.python_repr(py)?);
        fmt.positional_formatter(self.right.python_repr(py)?);
        fmt.keyword_if_true("negated", self.negated);
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for IsDistinctFrom {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.is_distinct_from() {
            flags::IsDistinctFrom::Unsupported => Err(unsupported("IS DISTINCT FROM is not supported by this dialect")),
            flags::IsDistinctFrom::Standard => {
                self.left.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                if self.negated {
                    out.push_str(" IS NOT DISTINCT FROM ");
                } else {
                    out.push_str(" IS DISTINCT FROM ");
                }
                self.right.write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            flags::IsDistinctFrom::NullSafeEqual => {
                // `<=>` is null-safe equal, i.e. `IS NOT DISTINCT FROM`.
                if !self.negated {
                    out.push_str("NOT (");
                }
                self.left.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(" <=> ");
                self.right.write_operand(dialect, out, Self::PRECEDENCE, true)?;
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("InListExpr");
        fmt.positional_formatter(self.expr.python_repr(py)?);
        fmt.positional_tuple(|group| -> pyo3::PyResult<()> {
            for item in &self.list {
                group.positional_formatter(item.python_repr(py)?);
            }
            Ok(())
        })?;
        fmt.keyword_if_true("negated", self.negated);
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for InList {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        if self.list.is_empty() {
            // `x IN ()` is false and `x NOT IN ()` is true, including when `x` is NULL.
            dialect.0.write_boolean(out, self.negated)?;
            return Ok(());
        }

        if dialect.0.parenthesize_in_between() {
            out.push_char('(');
        }
        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT IN (");
        } else {
            out.push_str(" IN (");
        }
        write_separated(dialect, out, &self.list, ", ")?;
        out.push_char(')');
        if dialect.0.parenthesize_in_between() {
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("BetweenExpr");
        fmt.positional_formatter(self.expr.python_repr(py)?);
        fmt.positional_formatter(self.low.python_repr(py)?);
        fmt.positional_formatter(self.high.python_repr(py)?);
        fmt.keyword_if_true("negated", self.negated);
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Between {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        if dialect.0.parenthesize_in_between() {
            out.push_char('(');
        }
        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT BETWEEN ");
        } else {
            out.push_str(" BETWEEN ");
        }
        self.low.write_operand(dialect, out, Self::PRECEDENCE, true)?;
        out.push_str(" AND ");
        self.high.write_operand(dialect, out, Self::PRECEDENCE, true)?;
        if dialect.0.parenthesize_in_between() {
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("CastExpr");
        fmt.positional_formatter(self.expr.python_repr(py)?);
        fmt.positional_debug(self.data_type_name.as_str());
        if self.kind != CastKind::Cast {
            fmt.keyword_debug("kind", self.kind.as_sql());
        }
        fmt.close();
        Ok(fmt)
    }

    fn write_call<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W, name: &str) -> PrepareResult<()> {
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
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match self.kind {
            CastKind::Cast => self.write_call(dialect, out, "CAST"),
            CastKind::TryCast => match dialect.0.try_cast() {
                flags::TryCast::Unsupported => Err(unsupported("try cast is not supported by this dialect")),
                flags::TryCast::TryCast => self.write_call(dialect, out, "TRY_CAST"),
                flags::TryCast::TryCastWord => self.write_call(dialect, out, "TRYCAST"),
                flags::TryCast::SafeCast => self.write_call(dialect, out, "SAFE_CAST"),
            },
            CastKind::DoubleColon => {
                if !dialect.0.supports_double_colon() {
                    return Err(unsupported("double-colon cast is not supported by this dialect"));
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let characters = self
            .characters
            .as_ref()
            .map(|characters| characters.python_repr(py))
            .transpose()?;
        let mut fmt = ReprFormatter::new_call("TrimExpr");
        fmt.positional_formatter(self.expr.python_repr(py)?);
        if self.where_ != TrimWhere::Both {
            fmt.keyword_debug("where", self.where_.as_sql());
        }
        fmt.keyword_optional_formatter("characters", characters);
        fmt.close();
        Ok(fmt)
    }
}

#[inline]
fn write_trim_function<W: SqlWriter>(trim: &Trim, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
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
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.trim() {
            flags::Trim::SqlFrom => {
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
            flags::Trim::Function => write_trim_function(self, dialect, out),
            flags::Trim::SpacesOnly => {
                if self.characters.is_some() {
                    return Err(unsupported("trim characters are not supported by this dialect"));
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("CollateExpr");
        fmt.positional_formatter(self.expr.python_repr(py)?);
        fmt.positional_formatter(self.collation.python_repr());
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Collate {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.collate() {
            flags::Collate::Unsupported => Err(unsupported("COLLATE is not supported by this dialect")),
            flags::Collate::InfixIdent => {
                self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(" COLLATE ");
                self.collation.write_sql(dialect, out)
            }
            flags::Collate::FunctionString => {
                let body = match self.collation.parts() {
                    [super::IdentPart::Quote(body) | super::IdentPart::Unquote(body)] if !body.is_empty() => {
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("CaseExpr");
        fmt.positional_list(|list| -> pyo3::PyResult<()> {
            for (when, then) in &self.when {
                let when = when.python_repr(py)?;
                let then = then.python_repr(py)?;
                list.positional_tuple(|pair| {
                    pair.positional_formatter(when);
                    pair.positional_formatter(then);
                });
            }
            Ok(())
        })?;
        let operand = self
            .operand
            .as_ref()
            .map(|operand| operand.python_repr(py))
            .transpose()?;
        let else_result = self
            .else_result
            .as_ref()
            .map(|else_result| else_result.python_repr(py))
            .transpose()?;
        fmt.keyword_optional_formatter("operand", operand);
        fmt.keyword_optional_formatter("else_", else_result);
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Case {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
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

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        let mut fmt = ReprFormatter::new_call("CustomExpr");
        fmt.positional_debug(self.sql.as_str());
        for arg in &self.args {
            fmt.positional_formatter(arg.python_repr(py)?);
        }
        fmt.close();
        Ok(fmt)
    }
}

impl WriteSql for Custom {
    #[cfg_attr(feature = "nightly", optimize(speed))]
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
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
