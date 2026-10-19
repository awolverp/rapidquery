use super::BinaryOp;
use super::CastKind;
use super::ExprInner;
use super::Ident;
use super::IsKind;
use super::TrimWhere;
use super::UnaryOp;
use super::UnarySpacing;
use crate::dialect::PyDialect;
use crate::dialect::config;
use crate::internal::repr;
use crate::internal::sql::PrepareResult;
use crate::internal::sql::SqlWriter;
use crate::internal::sql::WriteSql;
use crate::internal::sql::write_separated;

#[inline]
fn unsupported(message: &'static str) -> crate::internal::sql::PrepareError {
    rapidquery_macros::pyerr!(PyRuntimeError, message).into()
}

/// Row value. The dialect chooses `ROW`, `struct`, or parentheses.
///
/// An empty row is legal only for dialects that accept it. A string stored as
/// an item is a value expression, not a column name.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::Tuple;
/// use std::str::FromStr;
///
/// let row = Tuple {
///     items: vec![ExprInner::Ident(Ident::from_str("id").unwrap())],
/// };
/// assert_eq!(row.items.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Tuple {
    pub items: Vec<ExprInner>,
}

impl Tuple {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("TupleExpr");

        for item in &self.items {
            fmt.positional(item.python_repr(py)?);
        }
        fmt.close();

        Ok(fmt)
    }
}

impl WriteSql for Tuple {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        let arity = self.items.len();

        if dialect.0.tuple_syntax != config::TupleSyntax::Unsupported {
            if arity == 0 && !dialect.0.tuple_arity.contains(config::TupleArity::EMPTY) {
                return Err(unsupported("empty tuple is not supported by this dialect"));
            }
            if arity == 1 && !dialect.0.tuple_arity.contains(config::TupleArity::ONE) {
                return Err(unsupported("a tuple needs at least two elements on this dialect"));
            }
        }
        match dialect.0.tuple_syntax {
            config::TupleSyntax::Unsupported => {
                return Err(unsupported("tuple values are not supported by this dialect"));
            }
            config::TupleSyntax::Row => {
                out.push_str("ROW(");
            }
            config::TupleSyntax::Struct => {
                out.push_str("struct(");
            }
            config::TupleSyntax::Parentheses => {
                out.push_char('(');
            }
        }
        write_separated(dialect, out, &self.items, ", ")?;
        out.push_char(')');
        Ok(())
    }
}

/// Prefix operator: `NOT`, `+`, `-`, or `~`.
///
/// `NOT` is written with a space. The others sit against the operand. Dialects
/// that have no prefix `~` render it as a function.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::Unary;
/// use rapidquery::expr::enum_::UnaryOp;
/// use std::str::FromStr;
///
/// let expr = Unary {
///     op: UnaryOp::Not,
///     expr: Box::new(ExprInner::Ident(Ident::from_str("active").unwrap())),
/// };
/// assert_eq!(expr.op.as_sql(), "NOT");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Unary {
    pub op: UnaryOp,
    pub expr: Box<ExprInner>,
}

impl Unary {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("UnaryExpr");

        fmt.positional(self.op.as_sql())
            .positional(self.expr.python_repr(py)?)
            .close();

        Ok(fmt)
    }
}

impl WriteSql for Unary {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        if self.op == UnaryOp::BitNot {
            if !dialect.0.bitwise_operations.contains(config::BitwiseOperations::NOT) {
                return Err(unsupported("bitwise NOT is not supported by this dialect"));
            }

            match dialect.0.bitwise_call_syntax {
                config::BitwiseCallSyntax::Infix => {
                    out.push_char('~');
                    return self
                        .expr
                        .write_operand(dialect, out, UnaryOp::BitNot.precedence(), true);
                }
                config::BitwiseCallSyntax::BitFunction | config::BitwiseCallSyntax::ShiftFunction => {
                    out.push_str("BITNOT(");
                    self.expr.write_sql(dialect, out)?;
                    out.push_char(')');
                    return Ok(());
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

/// Infix operation, including arithmetic, comparison, and boolean operators.
///
/// `!=` is stored as `<>`. An unknown spelling is kept as a custom operator.
/// The dialect may rewrite `%`, `^`, `||`, and shifts into function calls.
///
/// ```
/// use rapidquery::expr::enum_::Binary;
/// use rapidquery::expr::enum_::BinaryOp;
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use std::str::FromStr;
///
/// let expr = Binary {
///     left: Box::new(ExprInner::Ident(Ident::from_str("age").unwrap())),
///     op: BinaryOp::Gt,
///     right: Box::new(ExprInner::Ident(Ident::from_str("n").unwrap())),
/// };
/// assert_eq!(expr.op.as_sql(), ">");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Binary {
    pub left: Box<ExprInner>,
    pub op: BinaryOp,
    pub right: Box<ExprInner>,
}

impl Binary {
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("BinaryExpr");

        fmt.positional(self.left.python_repr(py)?)
            .positional(self.op.as_sql())
            .positional(self.right.python_repr(py)?)
            .close();

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

/// Render one bitwise operator from [`flags::Bitwise`].
///
/// `infix` is the token for [`flags::BitCall::Infix`]. `bit_fn` is the name
/// for [`flags::BitCall::BitFunction`]. `shift_fn` is the name for
/// [`flags::BitCall::ShiftFunction`]. AND and OR pass the same function name
/// twice; shifts pass `BITSHIFT*` and `SHIFT*`.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn write_bit_op<W: SqlWriter>(
    bin: &Binary,
    dialect: &PyDialect,
    out: &mut W,
    op: config::BitwiseOperations,
    infix: &str,
    bit_fn: &str,
    shift_fn: &str,
    missing: &'static str,
) -> PrepareResult<()> {
    if !dialect.0.bitwise_operations.contains(op) {
        return Err(unsupported(missing));
    }
    match dialect.0.bitwise_call_syntax {
        config::BitwiseCallSyntax::Infix => write_infix(bin, dialect, out, infix, false),
        config::BitwiseCallSyntax::BitFunction => write_binary_call(dialect, out, bit_fn, &bin.left, &bin.right),
        config::BitwiseCallSyntax::ShiftFunction => write_binary_call(dialect, out, shift_fn, &bin.left, &bin.right),
    }
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
            BinaryOp::StringConcat => match dialect.0.string_concat {
                config::StringConcat::ConcatFunction => {
                    write_binary_call(dialect, out, "CONCAT", &self.left, &self.right)
                }
                config::StringConcat::Pipes => write_infix(self, dialect, out, "||", true),
                config::StringConcat::Plus => write_infix(self, dialect, out, "+", true),
            },
            BinaryOp::BitXor => match dialect.0.bitwise_xor_spelling {
                config::BitwiseXorSpelling::Unsupported => {
                    Err(unsupported("bitwise XOR is not supported by this dialect"))
                }
                config::BitwiseXorSpelling::Caret => write_infix(self, dialect, out, "^", false),
                config::BitwiseXorSpelling::Hash => write_infix(self, dialect, out, "#", false),
                config::BitwiseXorSpelling::Function => write_binary_call(dialect, out, "xor", &self.left, &self.right),
                config::BitwiseXorSpelling::BitFunction => {
                    write_binary_call(dialect, out, "BITXOR", &self.left, &self.right)
                }
            },
            BinaryOp::Mod => match dialect.0.modulo_spelling {
                config::ModuloSpelling::Percent => write_infix(self, dialect, out, "%", false),
                config::ModuloSpelling::Function => write_binary_call(dialect, out, "MOD", &self.left, &self.right),
                config::ModuloSpelling::Infix => write_infix(self, dialect, out, "MOD", false),
            },
            BinaryOp::BitAnd => write_bit_op(
                self,
                dialect,
                out,
                config::BitwiseOperations::AND,
                "&",
                "BITAND",
                "BITAND",
                "bitwise AND is not supported by this dialect",
            ),
            BinaryOp::BitOr => write_bit_op(
                self,
                dialect,
                out,
                config::BitwiseOperations::OR,
                "|",
                "BITOR",
                "BITOR",
                "bitwise OR is not supported by this dialect",
            ),
            BinaryOp::LShift => write_bit_op(
                self,
                dialect,
                out,
                config::BitwiseOperations::SHL,
                "<<",
                "BITSHIFTLEFT",
                "SHIFTLEFT",
                "bitwise shift is not supported by this dialect",
            ),
            BinaryOp::RShift => write_bit_op(
                self,
                dialect,
                out,
                config::BitwiseOperations::SHR,
                ">>",
                "BITSHIFTRIGHT",
                "SHIFTRIGHT",
                "bitwise shift is not supported by this dialect",
            ),
            _ => write_infix(self, dialect, out, self.op.as_sql(), false),
        }
    }
}

/// `expr IS [NOT] NULL`, `TRUE`, `FALSE`, or `UNKNOWN`.
///
/// `negated` selects `IS NOT`. The kind is a fixed word, not an expression.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::Is;
/// use rapidquery::expr::enum_::IsKind;
/// use std::str::FromStr;
///
/// let expr = Is {
///     expr: Box::new(ExprInner::Ident(Ident::from_str("name").unwrap())),
///     negated: false,
///     kind: IsKind::Null,
/// };
/// assert_eq!(expr.kind.as_sql(), "NULL");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Is {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub kind: IsKind,
}

impl Is {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("IsExpr");

        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.kind.as_sql())
            .keyword_true("negated", self.negated)
            .close();

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

/// Null-safe comparison, `IS [NOT] DISTINCT FROM`.
///
/// Either side may be `NULL` without making the predicate unknown. The dialect
/// writes the standard form, a null-safe operator, or rejects the node.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::IsDistinctFrom;
/// use std::str::FromStr;
///
/// let expr = IsDistinctFrom {
///     left: Box::new(ExprInner::Ident(Ident::from_str("a").unwrap())),
///     right: Box::new(ExprInner::Ident(Ident::from_str("b").unwrap())),
///     negated: false,
/// };
/// assert!(!expr.negated);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct IsDistinctFrom {
    pub left: Box<ExprInner>,
    pub right: Box<ExprInner>,
    pub negated: bool,
}

impl IsDistinctFrom {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("IsDistinctFromExpr");

        fmt.positional(self.left.python_repr(py)?)
            .positional(self.right.python_repr(py)?)
            .keyword_true("negated", self.negated)
            .close();

        Ok(fmt)
    }
}

impl WriteSql for IsDistinctFrom {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.is_distinct_from_spelling {
            config::IsDistinctFromSpelling::Unsupported => {
                Err(unsupported("IS DISTINCT FROM is not supported by this dialect"))
            }
            config::IsDistinctFromSpelling::Standard => {
                self.left.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                if self.negated {
                    out.push_str(" IS NOT DISTINCT FROM ");
                } else {
                    out.push_str(" IS DISTINCT FROM ");
                }
                self.right.write_operand(dialect, out, Self::PRECEDENCE, true)
            }
            config::IsDistinctFromSpelling::NullSafeEqual => {
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

/// Membership test, `expr [NOT] IN (list…)`.
///
/// An empty list renders as a boolean literal and does not write `expr`.
/// `NOT IN` of an empty list is the true literal.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::InList;
/// use std::str::FromStr;
///
/// let expr = InList {
///     expr: Box::new(ExprInner::Ident(Ident::from_str("id").unwrap())),
///     list: Vec::new(),
///     negated: false,
/// };
/// assert!(expr.list.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct InList {
    pub expr: Box<ExprInner>,
    pub list: Vec<ExprInner>,
    pub negated: bool,
}

impl InList {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("InListExpr");

        fmt.positional(self.expr.python_repr(py)?);

        let mut tuple = fmt.positional_tuple();
        for item in &self.list {
            tuple.positional(item.python_repr(py)?);
        }
        tuple.close();

        fmt.keyword_true("negated", self.negated);
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

        // SQL Server ranks `AND` above `IN`/`BETWEEN`/`OR`, and those four share
        // a level evaluated left to right. The parentheses keep the predicate
        // from being parsed as part of the left operand.
        out.push_char('(');

        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT IN (");
        } else {
            out.push_str(" IN (");
        }
        write_separated(dialect, out, &self.list, ", ")?;
        out.push_char(')');

        out.push_char(')');

        Ok(())
    }
}

/// Inclusive range, `expr [NOT] BETWEEN low AND high`.
///
/// Both bounds are expressions. There is no `SYMMETRIC` clause. Some dialects
/// parenthesize the whole predicate.
///
/// ```
/// use rapidquery::expr::enum_::Between;
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use std::str::FromStr;
///
/// let expr = Between {
///     expr: Box::new(ExprInner::Ident(Ident::from_str("n").unwrap())),
///     negated: false,
///     low: Box::new(ExprInner::Ident(Ident::from_str("lo").unwrap())),
///     high: Box::new(ExprInner::Ident(Ident::from_str("hi").unwrap())),
/// };
/// assert!(!expr.negated);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Between {
    pub expr: Box<ExprInner>,
    pub negated: bool,
    pub low: Box<ExprInner>,
    pub high: Box<ExprInner>,
}

impl Between {
    pub(crate) const PRECEDENCE: u8 = 5;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("BetweenExpr");
        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.low.python_repr(py)?)
            .positional(self.high.python_repr(py)?)
            .keyword_true("negated", self.negated)
            .close();

        Ok(fmt)
    }
}

impl WriteSql for Between {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        // SQL Server ranks `AND` above `IN`/`BETWEEN`/`OR`, and those four share
        // a level evaluated left to right. The parentheses keep the predicate
        // from being parsed as part of the left operand.
        out.push_char('(');

        self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
        if self.negated {
            out.push_str(" NOT BETWEEN ");
        } else {
            out.push_str(" BETWEEN ");
        }
        self.low.write_operand(dialect, out, Self::PRECEDENCE, true)?;
        out.push_str(" AND ");
        self.high.write_operand(dialect, out, Self::PRECEDENCE, true)?;

        out.push_char(')');
        Ok(())
    }
}

/// Cast to a SQL type name: `CAST`, `TRY_CAST`, or `::`.
///
/// `kind` stays one of those three. The dialect rewrites `TRY_CAST` to its own
/// spelling, such as `SAFE_CAST`, or rejects it.
///
/// ```
/// use rapidquery::data_type::DataTypeName;
/// use rapidquery::expr::enum_::Cast;
/// use rapidquery::expr::enum_::CastKind;
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use std::str::FromStr;
///
/// let expr = Cast {
///     kind: CastKind::Cast,
///     expr: Box::new(ExprInner::Ident(Ident::from_str("id").unwrap())),
///     data_type_name: DataTypeName::from("int"),
/// };
/// assert_eq!(expr.data_type_name.as_str(), "INT");
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Cast {
    pub kind: CastKind,
    pub expr: Box<ExprInner>,
    pub data_type_name: crate::data_type::DataTypeName,
}

impl Cast {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("CastExpr");

        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.data_type_name.as_str());

        if self.kind != CastKind::Cast {
            fmt.keyword("kind", self.kind.as_sql());
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
            CastKind::TryCast => match dialect.0.cast_try_cast {
                config::TryCast::Unsupported => Err(unsupported("try cast is not supported by this dialect")),
                config::TryCast::TryCast => self.write_call(dialect, out, "TRY_CAST"),
                config::TryCast::TryCastWord => self.write_call(dialect, out, "TRYCAST"),
                config::TryCast::SafeCast => self.write_call(dialect, out, "SAFE_CAST"),
            },
            CastKind::DoubleColon => {
                if !dialect.0.cast_double_colon {
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

/// `TRIM` of an expression, from one side, of an optional character set.
///
/// `where_` selects both ends, the leading end, or the trailing end. `characters`
/// is `None` when the call strips spaces. The dialect picks the function name.
///
/// ```
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use rapidquery::expr::enum_::Trim;
/// use rapidquery::expr::enum_::TrimWhere;
/// use std::str::FromStr;
///
/// let expr = Trim {
///     expr: Box::new(ExprInner::Ident(Ident::from_str("s").unwrap())),
///     where_: TrimWhere::Both,
///     characters: None,
/// };
/// assert!(expr.characters.is_none());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Trim {
    pub expr: Box<ExprInner>,
    pub where_: TrimWhere,
    pub characters: Option<Box<ExprInner>>,
}

impl Trim {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let characters = self
            .characters
            .as_ref()
            .map(|characters| characters.python_repr(py))
            .transpose()?;

        let mut fmt = repr::ReprFormatter::new_call("TrimExpr");

        fmt.positional(self.expr.python_repr(py)?);

        if self.where_ != TrimWhere::Both {
            fmt.keyword("where", self.where_.as_sql());
        }

        fmt.keyword("characters", characters).close();
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
        match dialect.0.trim_spelling {
            config::TrimSpelling::SqlFrom => {
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
            config::TrimSpelling::Function => write_trim_function(self, dialect, out),
            config::TrimSpelling::SpacesOnly => {
                if self.characters.is_some() {
                    return Err(unsupported("trim characters are not supported by this dialect"));
                }
                write_trim_function(self, dialect, out)
            }
        }
    }
}

/// Expression with a collation name attached.
///
/// The collation is an identifier. Dialects that spell this as a function quote
/// that name as a string instead. Rendering fails when collation is unsupported.
///
/// ```
/// use rapidquery::expr::enum_::Collate;
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use std::str::FromStr;
///
/// let expr = Collate {
///     expr: Box::new(ExprInner::Ident(Ident::from_str("s").unwrap())),
///     collation: Ident::from_str("en_US").unwrap(),
/// };
/// assert_eq!(expr.collation.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Collate {
    pub expr: Box<ExprInner>,
    pub collation: Ident,
}

impl Collate {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("CollateExpr");
        fmt.positional(self.expr.python_repr(py)?)
            .positional(self.collation.python_repr())
            .close();

        Ok(fmt)
    }
}

impl WriteSql for Collate {
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> PrepareResult<()> {
        match dialect.0.collate_spelling {
            config::CollateSpelling::Unsupported => Err(unsupported("COLLATE is not supported by this dialect")),
            config::CollateSpelling::InfixIdent => {
                self.expr.write_operand(dialect, out, Self::PRECEDENCE, false)?;
                out.push_str(" COLLATE ");
                self.collation.write_sql(dialect, out)
            }
            config::CollateSpelling::FunctionString => {
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

/// `CASE` expression, searched or matched against an operand.
///
/// `operand` is `None` for `CASE WHEN`. Each `when` entry is a condition or
/// comparison value paired with its result.
///
/// ```
/// use rapidquery::expr::enum_::Case;
///
/// let expr = Case {
///     operand: None,
///     when: Vec::new(),
///     else_result: None,
/// };
/// assert!(expr.operand.is_none());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    pub operand: Option<Box<ExprInner>>,
    pub when: Vec<(ExprInner, ExprInner)>,
    pub else_result: Option<Box<ExprInner>>,
}

impl Case {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let operand = self
            .operand
            .as_ref()
            .map(|operand| operand.python_repr(py))
            .transpose()?;

        let mut fmt = repr::ReprFormatter::new_call("CaseExpr");
        fmt.positional(operand);

        for (when, then) in self.when.iter() {
            fmt.append_call("when")
                .positional(when.python_repr(py)?)
                .positional(then.python_repr(py)?);
        }

        if let Some(else_result) = &self.else_result {
            fmt.append_call("else_").positional(else_result.python_repr(py)?);
        }

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

/// SQL template. Each `{}` slot is filled by one argument expression.
///
/// `{{` and `}}` write a literal brace. The template itself is not escaped.
/// Rendering fails when the number of slots and arguments differ.
///
/// ```
/// use rapidquery::expr::enum_::Custom;
/// use rapidquery::expr::enum_::ExprInner;
/// use rapidquery::expr::enum_::Ident;
/// use std::str::FromStr;
///
/// let expr = Custom {
///     sql: arcstr::ArcStr::from("COALESCE({}, 0)"),
///     args: vec![ExprInner::Ident(Ident::from_str("n").unwrap())],
/// };
/// assert_eq!(expr.args.len(), 1);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    pub sql: arcstr::ArcStr,
    pub args: Vec<ExprInner>,
}

impl Custom {
    pub(crate) const PRECEDENCE: u8 = 0;

    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<repr::ReprFormatter> {
        let mut fmt = repr::ReprFormatter::new_call("CustomExpr");

        fmt.positional(self.sql.as_str());

        for arg in &self.args {
            fmt.positional(arg.python_repr(py)?);
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
                        return Err(crate::internal::sql::PrepareError::CustomPlaceholders {
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
            return Err(crate::internal::sql::PrepareError::CustomPlaceholders {
                expected: used,
                got: self.args.len(),
            });
        }
        Ok(())
    }
}
