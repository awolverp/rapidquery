/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
    BitNot,
    /// `@-@` — PostgreSQL/Redshift geometric length.
    AtDashAt,
    /// `@@` — PostgreSQL/Redshift geometric center.
    DoubleAt,
    /// `#` — PostgreSQL/Redshift geometric npoints.
    Hash,
    /// `@` — PostgreSQL absolute value.
    PGAbs,
    /// `||/` — PostgreSQL cube root.
    PGCubeRoot,
    /// `!` postfix — PostgreSQL factorial (`9!`).
    PGPostfixFactorial,
    /// `!!` — PostgreSQL prefix factorial.
    PGPrefixFactorial,
    /// `|/` — PostgreSQL square root.
    PGSquareRoot,
    /// `?-` — PostgreSQL/Redshift geometric is-horizontal.
    QuestionDash,
    /// `?|` — PostgreSQL/Redshift geometric is-vertical.
    QuestionPipe,
}

impl UnaryOp {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "NOT" | "not" | "Not" => Some(Self::Not),
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "~" => Some(Self::BitNot),
            "@-@" => Some(Self::AtDashAt),
            "@@" => Some(Self::DoubleAt),
            "#" => Some(Self::Hash),
            "@" => Some(Self::PGAbs),
            "||/" => Some(Self::PGCubeRoot),
            "!" => Some(Self::PGPostfixFactorial),
            "!!" => Some(Self::PGPrefixFactorial),
            "|/" => Some(Self::PGSquareRoot),
            "?-" => Some(Self::QuestionDash),
            "?|" => Some(Self::QuestionPipe),
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
            Self::AtDashAt => "@-@",
            Self::DoubleAt => "@@",
            Self::Hash => "#",
            Self::PGAbs => "@",
            Self::PGCubeRoot => "||/",
            Self::PGPostfixFactorial => "!",
            Self::PGPrefixFactorial => "!!",
            Self::PGSquareRoot => "|/",
            Self::QuestionDash => "?-",
            Self::QuestionPipe => "?|",
        }
    }

    /// How this operator is written relative to its operand.
    #[inline]
    pub(crate) fn spacing(self) -> UnarySpacing {
        match self {
            Self::PGPostfixFactorial => UnarySpacing::Postfix,
            Self::Not
            | Self::Hash
            | Self::AtDashAt
            | Self::DoubleAt
            | Self::QuestionDash
            | Self::QuestionPipe => UnarySpacing::SpacedPrefix,
            Self::Plus
            | Self::Minus
            | Self::BitNot
            | Self::PGAbs
            | Self::PGSquareRoot
            | Self::PGCubeRoot
            | Self::PGPrefixFactorial => UnarySpacing::GluedPrefix,
        }
    }
}

/// How a [`UnaryOp`] is written relative to its operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnarySpacing {
    /// Operand then operator, no space (`9!`).
    Postfix,
    /// Operator, space, operand (`NOT x`, `# path`).
    SpacedPrefix,
    /// Operator glued to the operand (`-1`, `|/9`).
    GluedPrefix,
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
            Self::Xor => 7,
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

/// Unicode normalization form for [`super::IsNormalized`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizationForm {
    Nfc,
    Nfd,
    Nfkc,
    Nfkd,
}

impl NormalizationForm {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("NFC") {
            Some(Self::Nfc)
        } else if s.eq_ignore_ascii_case("NFD") {
            Some(Self::Nfd)
        } else if s.eq_ignore_ascii_case("NFKC") {
            Some(Self::Nfkc)
        } else if s.eq_ignore_ascii_case("NFKD") {
            Some(Self::Nfkd)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Nfc => "NFC",
            Self::Nfd => "NFD",
            Self::Nfkc => "NFKC",
            Self::Nfkd => "NFKD",
        }
    }
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastKind {
    Cast,
    TryCast,
    SafeCast,
}

impl CastKind {
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("CAST") {
            Some(Self::Cast)
        } else if s.eq_ignore_ascii_case("TRY_CAST") {
            Some(Self::TryCast)
        } else if s.eq_ignore_ascii_case("SAFE_CAST") {
            Some(Self::SafeCast)
        } else {
            None
        }
    }

    #[inline]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Cast => "CAST",
            Self::TryCast => "TRY_CAST",
            Self::SafeCast => "SAFE_CAST",
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
