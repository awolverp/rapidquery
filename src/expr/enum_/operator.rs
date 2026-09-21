/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
    BitNot,
}

impl UnaryOp {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "NOT" | "not" | "Not" => Some(Self::Not),
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
    Custom(arcstr::ArcStr),
}

impl BinaryOp {
    #[inline]
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
