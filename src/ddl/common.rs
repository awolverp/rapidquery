/// Represents `ASC` and `DESC` keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::IntoStaticStr)]
#[strum(ascii_case_insensitive)]
pub enum Order {
    /// Emits `ASC`.
    #[strum(serialize = "ASC")]
    Asc,
    /// Emits `DESC`.
    #[strum(serialize = "DESC")]
    Desc,
}
