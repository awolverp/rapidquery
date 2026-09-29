/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
    BitNot,
}

/// How a [`UnaryOp`] is written relative to its operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnarySpacing {
    /// Operator, space, operand (`NOT x`, `# path`).
    SpacedPrefix,
    /// Operator glued to the operand (`-1`, `|/9`).
    GluedPrefix,
}

impl UnaryOp {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("not") {
            return Some(Self::Not);
        }

        match s {
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "~" => Some(Self::BitNot),
            _ => None,
        }
    }

    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Not => "NOT",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::BitNot => "~",
        }
    }

    /// How this operator is written relative to its operand.
    #[inline]
    pub(crate) fn spacing(self) -> UnarySpacing {
        match self {
            Self::Not => UnarySpacing::SpacedPrefix,
            Self::Plus | Self::Minus | Self::BitNot => UnarySpacing::GluedPrefix,
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
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
    StringConcat,
    Custom(arcstr::ArcStr),
}

impl BinaryOp {
    #[inline]
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("and") {
            return Self::And;
        }
        if s.eq_ignore_ascii_case("or") {
            return Self::Or;
        }

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
            "&" => Self::BitAnd,
            "|" => Self::BitOr,
            "^" => Self::BitXor,
            "<<" => Self::LShift,
            ">>" => Self::RShift,
            "||" => Self::StringConcat,
            other => Self::Custom(arcstr::ArcStr::from(other)),
        }
    }

    #[inline]
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
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::LShift => "<<",
            Self::RShift => ">>",
            Self::StringConcat => "||",
            Self::Custom(s) => s.as_str(),
        }
    }

    /// Lower is tighter binding. Used by [`super::Binary`] and parenthesization.
    #[inline]
    pub fn precedence(&self) -> u8 {
        match self {
            Self::Mul | Self::Div | Self::Mod => 2,
            Self::Add | Self::Sub => 3,
            Self::BitAnd
            | Self::BitOr
            | Self::BitXor
            | Self::LShift
            | Self::RShift
            | Self::StringConcat => 4,
            Self::Eq
            | Self::NotEq
            | Self::Lt
            | Self::LtEq
            | Self::Gt
            | Self::GtEq
            | Self::Custom(_) => 5,
            Self::And => 6,
            Self::Or => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsKind {
    Null,
    True,
    False,
    Unknown,
}

impl IsKind {
    #[inline]
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

    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Null => "NULL",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// `CAST` / `TRY_CAST` / `::`. Dialect spelling of `TRY_CAST` is chosen at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastKind {
    Cast,
    TryCast,
    /// `expr::type`. Opt-in; a plain `CAST` is never rewritten to this.
    DoubleColon,
}

impl CastKind {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("CAST") {
            Some(Self::Cast)
        } else if s.eq_ignore_ascii_case("TRY_CAST") {
            Some(Self::TryCast)
        } else if s == "::" {
            Some(Self::DoubleColon)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Cast => "CAST",
            Self::TryCast => "TRY_CAST",
            Self::DoubleColon => "::",
        }
    }
}

/// `BOTH` / `LEADING` / `TRAILING` in [`super::Trim`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrimWhere {
    Both,
    Leading,
    Trailing,
}

impl TrimWhere {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("BOTH") {
            Some(Self::Both)
        } else if s.eq_ignore_ascii_case("LEADING") {
            Some(Self::Leading)
        } else if s.eq_ignore_ascii_case("TRAILING") {
            Some(Self::Trailing)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Both => "BOTH",
            Self::Leading => "LEADING",
            Self::Trailing => "TRAILING",
        }
    }
}
