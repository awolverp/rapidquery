#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoolFormat {
    /// `TRUE` / `FALSE`.
    Keyword,
    /// `1` / `0` (MySQL, SQLite).
    Int,
}

impl BoolFormat {
    pub fn format(self, value: bool) -> &'static str {
        match (self, value) {
            (Self::Keyword, true) => "TRUE",
            (Self::Keyword, false) => "FALSE",
            (Self::Int, true) => "1",
            (Self::Int, false) => "0",
        }
    }
}

pub trait Dialect {
    // we cannot use constant variables here for boolean format
    // and identifier quotes.
    //
    // https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility

    /// Dialect name
    fn name(&self) -> &'static str;

    /// How a boolean should be rendered.
    fn bool_format(&self) -> BoolFormat {
        BoolFormat::Int
    }

    /// The start/end characters used to quote identifiers.
    fn identifier_quotes(&self) -> Option<(char, char)> {
        None
    }

    /// Escapes string literals.
    fn escape_string(&self, string: &str) -> String;

    /// (zero-copy) Quotes string literals.
    ///
    /// You should call [`Dialect::escape`] before this method.
    fn quote_string(&self, string: &mut String);

    /// Calls [`Self::escape_string`] and [`Self::quote_string`] in order.
    fn escape_and_quote_string(&self, string: &str) -> String {
        let mut result = self.escape_string(string);
        self.quote_string(&mut result);
        result
    }

    /// Render a byte slice as a literal in the configured form.
    fn encode_bytes(&self, bytes: &[u8]) -> String;

    /// Render a byte slice as a literal in the configured form.
    fn encode_bits(&self, bits: &str) -> String;
}
