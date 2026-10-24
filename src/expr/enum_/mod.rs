mod func;
mod ident;
mod like;
mod nodes;
mod operator;
mod value;

pub use func::*;
pub use ident::*;
pub use like::*;
pub use nodes::*;
pub use operator::*;
pub use value::*;

use crate::dialect::PyDialect;
use crate::internal::repr::ReprFormatter;
use crate::sql::SqlWriter;
use crate::sql::WriteSql;

/// SQL expression tree.
///
/// Concrete variants wrap the node structs in [`nodes`]. [`Ident`], [`Value`],
/// and [`Function`] live in sibling modules and already implement `python_repr`
/// / [`WriteSql`]; this enum only dispatches.
///
/// TODO: `Exists`, `InSubquery`, `Subquery`, and `Quantified` (`ANY` /
/// `ALL` / `SOME`) once `Query` exists.
#[derive(Clone, Debug, PartialEq)]
pub enum ExprInner {
    /// Identifier (`schema.table.column`, `*`).
    Ident(Ident),
    /// Bound or inlined SQL value plus its serializer.
    Value(Value),
    /// Parenthesized row `(a, b, …)`.
    Tuple(Tuple),
    /// Unary operation (`NOT x`, `-x`, postfix `x!`).
    Unary(Unary),
    /// Binary operation (`a + b`, `a AND b`).
    Binary(Binary),
    /// Function call.
    Function(Function),
    /// `expr IS [NOT] {NULL|TRUE|FALSE|UNKNOWN}`.
    Is(Is),
    /// `left IS [NOT] DISTINCT FROM right`.
    IsDistinctFrom(IsDistinctFrom),
    /// `expr [NOT] IN (list…)`.
    InList(InList),
    /// `expr [NOT] BETWEEN low AND high`.
    Between(Between),
    Like(Like),
    SimilarTo(SimilarTo),
    RLike(RLike),
    Cast(Cast),
    Trim(Trim),
    Collate(Collate),
    Case(Case),
    Custom(Custom),
}

macro_rules! impl_from_expr_inner {
    ($($ty:ty => $variant:ident),+ $(,)?) => {
        $(
            impl From<$ty> for ExprInner {
                #[inline]
                fn from(value: $ty) -> Self {
                    Self::$variant(value)
                }
            }
        )+
    };
}

impl_from_expr_inner! {
    Ident => Ident,
    Value => Value,
    Tuple => Tuple,
    Unary => Unary,
    Binary => Binary,
    Function => Function,
    Is => Is,
    IsDistinctFrom => IsDistinctFrom,
    InList => InList,
    Between => Between,
    Like => Like,
    SimilarTo => SimilarTo,
    RLike => RLike,
    Cast => Cast,
    Trim => Trim,
    Collate => Collate,
    Case => Case,
    Custom => Custom,
}

impl ExprInner {
    /// Lower is tighter binding.
    #[inline]
    fn precedence(&self) -> u8 {
        match self {
            Self::Ident(_) | Self::Value(_) | Self::Function(_) => 0,
            Self::Tuple(_) => Tuple::PRECEDENCE,
            Self::Unary(u) => u.op.precedence(),
            Self::Binary(b) => b.op.precedence(),
            Self::Is(_) => Is::PRECEDENCE,
            Self::InList(_) => InList::PRECEDENCE,
            Self::IsDistinctFrom(_) => IsDistinctFrom::PRECEDENCE,
            Self::Between(_) => Between::PRECEDENCE,
            Self::Like(_) => Like::PRECEDENCE,
            Self::SimilarTo(_) => SimilarTo::PRECEDENCE,
            Self::RLike(_) => RLike::PRECEDENCE,
            Self::Cast(_) => Cast::PRECEDENCE,
            Self::Trim(_) => Trim::PRECEDENCE,
            Self::Collate(_) => Collate::PRECEDENCE,
            Self::Case(_) => Case::PRECEDENCE,
            Self::Custom(_) => Custom::PRECEDENCE,
        }
    }

    /// Write `self` as an operand of a parent at `parent_prec`.
    ///
    /// Parenthesize when this node binds more loosely than the parent, or
    /// when `paren_if_equal` and the precedences match (right-associativity
    /// / postfix nesting).
    #[inline]
    pub(crate) fn write_operand<W: SqlWriter>(
        &self,
        dialect: &PyDialect,
        out: &mut W,
        parent_prec: u8,
        paren_if_equal: bool,
    ) -> pyo3::PyResult<()> {
        let child_prec = self.precedence();
        let paren = child_prec > parent_prec || (paren_if_equal && child_prec == parent_prec);
        if paren {
            out.write('(');
        }
        self.write_sql(dialect, out)?;
        if paren {
            out.write(')');
        }
        Ok(())
    }

    #[inline]
    pub fn python_repr(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<ReprFormatter> {
        match self {
            Self::Ident(x) => Ok(x.python_repr()),
            Self::Value(x) => x.python_repr(py),
            Self::Tuple(x) => x.python_repr(py),
            Self::Unary(x) => x.python_repr(py),
            Self::Binary(x) => x.python_repr(py),
            Self::Function(x) => x.python_repr(py),
            Self::Is(x) => x.python_repr(py),
            Self::IsDistinctFrom(x) => x.python_repr(py),
            Self::InList(x) => x.python_repr(py),
            Self::Between(x) => x.python_repr(py),
            Self::Like(x) => x.python_repr(py),
            Self::SimilarTo(x) => x.python_repr(py),
            Self::RLike(x) => x.python_repr(py),
            Self::Cast(x) => x.python_repr(py),
            Self::Trim(x) => x.python_repr(py),
            Self::Collate(x) => x.python_repr(py),
            Self::Case(x) => x.python_repr(py),
            Self::Custom(x) => x.python_repr(py),
        }
    }
}

impl WriteSql for ExprInner {
    #[cfg_attr(feature = "nightly", optimize(speed))]
    fn write_sql<W: SqlWriter>(&self, dialect: &PyDialect, out: &mut W) -> pyo3::PyResult<()> {
        match self {
            Self::Ident(x) => x.write_sql(dialect, out),
            Self::Value(x) => x.write_sql(dialect, out),
            Self::Tuple(x) => x.write_sql(dialect, out),
            Self::Unary(x) => x.write_sql(dialect, out),
            Self::Binary(x) => x.write_sql(dialect, out),
            Self::Function(x) => x.write_sql(dialect, out),
            Self::Is(x) => x.write_sql(dialect, out),
            Self::IsDistinctFrom(x) => x.write_sql(dialect, out),
            Self::InList(x) => x.write_sql(dialect, out),
            Self::Between(x) => x.write_sql(dialect, out),
            Self::Like(x) => x.write_sql(dialect, out),
            Self::SimilarTo(x) => x.write_sql(dialect, out),
            Self::RLike(x) => x.write_sql(dialect, out),
            Self::Cast(x) => x.write_sql(dialect, out),
            Self::Trim(x) => x.write_sql(dialect, out),
            Self::Collate(x) => x.write_sql(dialect, out),
            Self::Case(x) => x.write_sql(dialect, out),
            Self::Custom(x) => x.write_sql(dialect, out),
        }
    }
}
