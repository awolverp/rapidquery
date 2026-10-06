/// Buffer for a Python `__repr__` of the form `Name(...)` or `[...]`.
///
/// Arguments are written in place. A nested formatter is copied into the parent
/// once, after it is closed.
///
/// ```
/// use rapidquery::internal::repr::ReprFormatter;
///
/// let mut fmt = ReprFormatter::new_call("Ident");
/// fmt.positional_debug("id");
/// fmt.close();
/// assert_eq!(fmt.as_str(), "Ident(\"id\")");
/// ```
pub struct ReprFormatter {
    buf: String,
    separated: bool,
    closer: Closer,
    closed: bool,
}

#[derive(Clone, Copy)]
enum Closer {
    Paren,
    Bracket,
}

impl ReprFormatter {
    /// `name(`.
    pub fn new_call(name: &str) -> Self {
        let mut buf = String::with_capacity(name.len() + 32);
        buf.push_str(name);
        buf.push('(');
        Self {
            buf,
            separated: false,
            closer: Closer::Paren,
            closed: false,
        }
    }

    /// `[`.
    pub fn new_list() -> Self {
        Self {
            buf: String::from("["),
            separated: false,
            closer: Closer::Bracket,
            closed: false,
        }
    }

    pub fn positional_debug(&mut self, value: impl std::fmt::Debug) -> &mut Self {
        self.separate();
        self.push_debug(&value);
        self
    }

    /// Paste `value` with no extra quotes and no separator logic inside it.
    pub fn positional_raw(&mut self, value: &str) -> &mut Self {
        self.separate();
        self.buf.push_str(value);
        self
    }

    pub fn positional_formatter(&mut self, child: ReprFormatter) -> &mut Self {
        self.separate();
        self.push_formatter(child);
        self
    }

    /// Positional `[...]`. Items are written with the same argument methods.
    pub fn positional_list<R>(&mut self, write_items: impl FnOnce(&mut Self) -> R) -> R {
        self.group('[', ']', write_items)
    }

    /// Positional `(...)`.
    pub fn positional_tuple<R>(&mut self, write_items: impl FnOnce(&mut Self) -> R) -> R {
        self.group('(', ')', write_items)
    }

    pub fn keyword_debug(&mut self, key: &str, value: impl std::fmt::Debug) -> &mut Self {
        self.separate();
        self.push_key(key);
        self.push_debug(&value);
        self
    }

    pub fn keyword_formatter(&mut self, key: &str, child: ReprFormatter) -> &mut Self {
        self.separate();
        self.push_key(key);
        self.push_formatter(child);
        self
    }

    pub fn keyword_optional_debug<T: std::fmt::Debug>(&mut self, key: &str, value: Option<T>) -> &mut Self {
        if let Some(value) = value {
            self.keyword_debug(key, value);
        }
        self
    }

    pub fn keyword_optional_formatter(&mut self, key: &str, value: Option<ReprFormatter>) -> &mut Self {
        if let Some(child) = value {
            self.keyword_formatter(key, child);
        }
        self
    }

    /// `key=True` when `value` is set. Omitted when it is clear.
    pub fn keyword_if_true(&mut self, key: &str, value: bool) -> &mut Self {
        if value {
            self.separate();
            self.push_key(key);
            self.buf.push_str("True");
        }
        self
    }

    /// `key=False` when `value` is clear. Omitted when it is set.
    pub fn keyword_if_false(&mut self, key: &str, value: bool) -> &mut Self {
        if !value {
            self.separate();
            self.push_key(key);
            self.buf.push_str("False");
        }
        self
    }

    /// Write the closing `)` or `]`. Further calls leave the text unchanged.
    pub fn close(&mut self) -> &mut Self {
        if !self.closed {
            self.buf.push(match self.closer {
                Closer::Paren => ')',
                Closer::Bracket => ']',
            });
            self.closed = true;
        }
        self
    }

    /// Append `suffix` after the closer. Closes first when still open.
    pub fn append_suffix(&mut self, suffix: &str) -> &mut Self {
        self.close();
        self.buf.push_str(suffix);
        self
    }

    /// Append `.method(argument)` after the closer. `argument` is already closed.
    pub fn append_call(&mut self, method: &str, argument: &ReprFormatter) -> &mut Self {
        debug_assert!(argument.closed);
        self.close();
        self.buf.push('.');
        self.buf.push_str(method);
        self.buf.push('(');
        self.buf.reserve(argument.buf.len());
        self.buf.push_str(&argument.buf);
        self.buf.push(')');
        self
    }

    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Move the buffer out. Closes first when still open.
    pub fn into_string(mut self) -> String {
        self.close();
        self.buf
    }

    fn separate(&mut self) {
        debug_assert!(!self.closed);
        if self.separated {
            self.buf.push_str(", ");
        } else {
            self.separated = true;
        }
    }

    fn push_key(&mut self, key: &str) {
        self.buf.push_str(key);
        self.buf.push('=');
    }

    fn push_debug(&mut self, value: &impl std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.buf, "{value:?}").unwrap();
    }

    fn push_formatter(&mut self, mut child: ReprFormatter) {
        child.close();
        self.buf.reserve(child.buf.len());
        self.buf.push_str(&child.buf);
    }

    fn group<R>(&mut self, open: char, close: char, write_items: impl FnOnce(&mut Self) -> R) -> R {
        self.separate();
        self.buf.push(open);
        self.separated = false;
        let result = write_items(self);
        self.buf.push(close);
        self.separated = true;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::ReprFormatter;

    #[test]
    fn call_debug_escape_and_bools() {
        let mut fmt = ReprFormatter::new_call("LikeExpr");
        fmt.positional_debug("a\"b\\c");
        fmt.keyword_if_false("case_sensitive", false);
        fmt.keyword_if_true("negated", true);
        fmt.keyword_if_true("distinct", false);
        fmt.keyword_optional_debug::<&str>("escape", None);
        fmt.keyword_optional_debug("where", Some("LEADING"));
        assert_eq!(
            fmt.into_string(),
            "LikeExpr(\"a\\\"b\\\\c\", case_sensitive=False, negated=True, where=\"LEADING\")"
        );
    }

    #[test]
    fn list_embeds_formatter_and_tuple() {
        let mut ident = ReprFormatter::new_call("Ident");
        ident.positional_debug("id");

        let mut fmt = ReprFormatter::new_list();
        fmt.positional_formatter(ident);
        fmt.positional_tuple(|pair| {
            pair.positional_debug("when");
            pair.positional_raw("Value(1, IntegerSerializer())");
        });
        fmt.keyword_optional_formatter("else_", None);
        assert_eq!(
            fmt.into_string(),
            "[Ident(\"id\"), (\"when\", Value(1, IntegerSerializer()))]"
        );
    }

    #[test]
    fn append_call_after_close() {
        let mut argument = ReprFormatter::new_call("Ident");
        argument.positional_debug("x");
        argument.close();

        let mut fmt = ReprFormatter::new_call("Function");
        fmt.positional_debug("lag");
        fmt.append_suffix(".ignore_nulls()");
        fmt.append_call("filter", &argument);
        assert_eq!(
            fmt.into_string(),
            "Function(\"lag\").ignore_nulls().filter(Ident(\"x\"))"
        );
    }
}
