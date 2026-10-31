use std::fmt;
use std::fmt::Write;

/// A value rendered straight into a [`ReprFormatter`] buffer.
///
/// Omitted values write nothing and do not emit a separator. [`Option`] is
/// omitted when empty; [`IfTrue`] / [`IfFalse`] are omitted on the other polarity.
pub trait WriteRepr {
    fn omit(&self) -> bool {
        false
    }

    fn write_repr(self, buf: &mut String);
}

impl WriteRepr for &str {
    fn write_repr(self, buf: &mut String) {
        write!(buf, "{self:?}").unwrap();
    }
}

impl WriteRepr for String {
    fn write_repr(self, buf: &mut String) {
        write!(buf, "{self:?}").unwrap();
    }
}

impl WriteRepr for arcstr::ArcStr {
    fn write_repr(self, buf: &mut String) {
        write!(buf, "{self:?}").unwrap();
    }
}

impl WriteRepr for char {
    fn write_repr(self, buf: &mut String) {
        write!(buf, "{self:?}").unwrap();
    }
}

impl WriteRepr for bool {
    fn write_repr(self, buf: &mut String) {
        buf.push_str(if self { "True" } else { "False" });
    }
}

/// Text inserted with no quoting or escaping.
pub struct Raw<'a>(pub &'a str);

impl WriteRepr for Raw<'_> {
    fn write_repr(self, buf: &mut String) {
        buf.push_str(self.0);
    }
}

/// Python `True`, written only when the flag is set.
pub struct IfTrue(pub bool);

impl WriteRepr for IfTrue {
    fn omit(&self) -> bool {
        !self.0
    }

    fn write_repr(self, buf: &mut String) {
        buf.push_str("True");
    }
}

/// Python `False`, written only when the flag is clear.
pub struct IfFalse(pub bool);

impl WriteRepr for IfFalse {
    fn omit(&self) -> bool {
        self.0
    }

    fn write_repr(self, buf: &mut String) {
        buf.push_str("False");
    }
}

impl WriteRepr for ReprFormatter {
    fn write_repr(mut self, buf: &mut String) {
        self.close();
        buf.push_str(&self.buf);
    }
}

impl<T: WriteRepr> WriteRepr for Option<T> {
    fn omit(&self) -> bool {
        match self {
            Some(value) => value.omit(),
            None => true,
        }
    }

    fn write_repr(self, buf: &mut String) {
        if let Some(value) = self {
            value.write_repr(buf);
        }
    }
}

/// Buffer for a Python `__repr__` of the form `Name(...)` or `[...]`.
///
/// Arguments are written in place. A nested formatter is copied into the parent
/// once, after it is closed. Bracketed groups are a [`ReprGroup`] borrow of this
/// buffer; dropping the group writes the closer.
///
/// ```
/// use rapidquery::internal::repr::ReprFormatter;
///
/// let mut fmt = ReprFormatter::new_call("Ident");
/// fmt.positional("id");
/// fmt.close();
/// assert_eq!(fmt.as_str(), "Ident(\"id\")");
/// ```
pub struct ReprFormatter {
    buf: String,
    close_token: char,
    is_separated: bool,
    is_closed: bool,
}

impl ReprFormatter {
    #[inline]
    fn separate(&mut self) {
        debug_assert!(!self.is_closed);

        if self.is_separated {
            self.buf.push_str(", ");
        } else {
            self.is_separated = true;
        }
    }

    #[inline]
    fn push_key(&mut self, key: &str) {
        self.buf.push_str(key);
        self.buf.push('=');
    }

    #[inline]
    fn write_value(&mut self, value: impl WriteRepr) -> bool {
        if value.omit() {
            return false;
        }

        self.separate();
        value.write_repr(&mut self.buf);
        true
    }

    fn open_group(&mut self, open: char, close: char) -> ReprGroup<'_> {
        self.separate();
        self.buf.push(open);
        self.is_separated = false;

        ReprGroup {
            fmt: self,
            close_token: close,
            finished: false,
        }
    }
}

impl ReprFormatter {
    pub fn new_call(name: &str) -> Self {
        let mut buf = String::with_capacity(name.len() + 32);
        buf.push_str(name);
        buf.push('(');

        Self {
            buf,
            close_token: ')',
            is_separated: false,
            is_closed: false,
        }
    }

    #[inline]
    pub fn new_list() -> Self {
        Self {
            buf: String::from("["),
            close_token: ']',
            is_separated: false,
            is_closed: false,
        }
    }

    pub fn positional(&mut self, value: impl WriteRepr) -> &mut Self {
        self.write_value(value);
        self
    }

    pub fn positional_debug(&mut self, value: &impl fmt::Debug) -> &mut Self {
        self.separate();
        write!(self.buf, "{value:?}").unwrap();
        self
    }

    pub fn positional_raw(&mut self, value: &str) -> &mut Self {
        self.positional(Raw(value))
    }

    pub fn positional_list(&mut self) -> ReprGroup<'_> {
        self.open_group('[', ']')
    }

    pub fn positional_tuple(&mut self) -> ReprGroup<'_> {
        self.open_group('(', ')')
    }

    pub fn keyword(&mut self, key: &str, value: impl WriteRepr) -> &mut Self {
        if !value.omit() {
            self.separate();
            self.push_key(key);
            value.write_repr(&mut self.buf);
        }

        self
    }

    pub fn keyword_debug(&mut self, key: &str, value: &impl fmt::Debug) -> &mut Self {
        self.separate();
        self.push_key(key);
        write!(self.buf, "{value:?}").unwrap();
        self
    }

    pub fn keyword_raw(&mut self, key: &str, value: &str) -> &mut Self {
        self.keyword(key, Raw(value))
    }

    /// Write `key=True` only when `value` is set.
    pub fn keyword_true(&mut self, key: &str, value: bool) -> &mut Self {
        self.keyword(key, IfTrue(value))
    }

    /// Write `key=False` only when `value` is clear.
    pub fn keyword_false(&mut self, key: &str, value: bool) -> &mut Self {
        self.keyword(key, IfFalse(value))
    }

    /// Write the closing character. Further calls leave the text unchanged.
    pub fn close(&mut self) -> &mut Self {
        if !self.is_closed {
            self.buf.push(self.close_token);
            self.is_closed = true;
        }
        self
    }

    /// Append `suffix` after the closer. Closes first when still open.
    pub fn append_suffix(&mut self, suffix: &str) -> &mut Self {
        self.close();
        self.buf.push_str(suffix);
        self
    }

    /// Append `.method(` after the closer and reopen for more arguments.
    pub fn append_call(&mut self, method: &str) -> &mut Self {
        self.close();

        self.buf.push('.');
        self.buf.push_str(method);
        self.buf.push('(');

        self.is_closed = false;
        self.is_separated = false;
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
}

/// A `[...]` or `(...)` opened inside a [`ReprFormatter`].
///
/// Writes into the parent buffer. Drop writes the closer and marks the parent
/// argument list as separated, so the next sibling gets a comma.
pub struct ReprGroup<'a> {
    fmt: &'a mut ReprFormatter,
    close_token: char,
    finished: bool,
}

impl ReprGroup<'_> {
    fn finish(&mut self) {
        if !self.finished {
            self.fmt.buf.push(self.close_token);
            self.fmt.is_separated = true;
            self.finished = true;
        }
    }

    pub fn positional(&mut self, value: impl WriteRepr) -> &mut Self {
        self.fmt.positional(value);
        self
    }

    pub fn positional_debug(&mut self, value: &impl fmt::Debug) -> &mut Self {
        self.fmt.positional_debug(value);
        self
    }

    pub fn positional_raw(&mut self, value: &str) -> &mut Self {
        self.fmt.positional_raw(value);
        self
    }

    pub fn positional_list(&mut self) -> ReprGroup<'_> {
        self.fmt.positional_list()
    }

    pub fn positional_tuple(&mut self) -> ReprGroup<'_> {
        self.fmt.positional_tuple()
    }

    pub fn keyword(&mut self, key: &str, value: impl WriteRepr) -> &mut Self {
        self.fmt.keyword(key, value);
        self
    }

    pub fn keyword_debug(&mut self, key: &str, value: &impl fmt::Debug) -> &mut Self {
        self.fmt.keyword_debug(key, value);
        self
    }

    pub fn keyword_raw(&mut self, key: &str, value: &str) -> &mut Self {
        self.fmt.keyword_raw(key, value);
        self
    }

    pub fn keyword_true(&mut self, key: &str, value: bool) -> &mut Self {
        self.fmt.keyword_true(key, value);
        self
    }

    pub fn keyword_false(&mut self, key: &str, value: bool) -> &mut Self {
        self.fmt.keyword_false(key, value);
        self
    }

    /// Write the closer now. A later drop is a no-op.
    pub fn close(mut self) {
        self.finish();
    }
}

impl Drop for ReprGroup<'_> {
    fn drop(&mut self) {
        self.finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_debug_escape_and_bools() {
        let mut fmt = ReprFormatter::new_call("LikeExpr");

        fmt.positional("a\"b\\c")
            .keyword_false("case_sensitive", false)
            .keyword_true("negated", true)
            .keyword_true("distinct", false)
            .keyword("escape", None::<&str>)
            .keyword("where", Some("LEADING"));

        assert_eq!(
            fmt.into_string(),
            "LikeExpr(\"a\\\"b\\\\c\", case_sensitive=False, negated=True, where=\"LEADING\")"
        );
    }

    #[test]
    fn list_embeds_formatter_and_tuple() {
        let mut ident = ReprFormatter::new_call("Ident");
        ident.positional("id");

        let mut fmt = ReprFormatter::new_list();
        fmt.positional(ident);
        fmt.positional_tuple()
            .positional("when")
            .positional_raw("Value(1, IntegerSerializer())");

        fmt.keyword("else_", None::<&str>);

        assert_eq!(
            fmt.into_string(),
            "[Ident(\"id\"), (\"when\", Value(1, IntegerSerializer()))]"
        );
    }

    #[test]
    fn append_call_after_close() {
        let mut argument = ReprFormatter::new_call("Ident");
        argument.positional("x").close();

        let mut fmt = ReprFormatter::new_call("Function");
        fmt.positional("lag")
            .append_suffix(".ignore_nulls()")
            .append_call("filter")
            .positional(argument);

        assert_eq!(
            fmt.into_string(),
            "Function(\"lag\").ignore_nulls().filter(Ident(\"x\"))"
        );
    }
}
