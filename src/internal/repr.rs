/// A helper to generate __repr__ methods.
#[derive(Default, Clone, Debug)]
pub struct ReprFormatter {
    buf: String,
    started: bool,
}

#[derive(Debug)]
pub struct ReprVec {
    key: &'static str,
    optional: bool,
    items: String,
}

#[derive(Debug)]
pub struct ReprNestedVec {
    index: usize,
    items: String,
}

impl ReprFormatter {
    #[inline]
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            buf: String::from("<") + name.as_ref(),
            started: false,
        }
    }

    #[inline]
    pub fn new_with_pyref<T: pyo3::PyClass>(pyref: &pyo3::PyRef<'_, T>) -> Self {
        Self::new(crate::internal::get_type_name(pyref.py(), pyref.as_ptr()))
    }

    #[inline]
    pub fn vec(&self, key: &'static str, optional: bool) -> ReprVec {
        ReprVec {
            key,
            optional,
            items: String::new(),
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    pub fn finish(&mut self) -> String {
        self.take().buf + ">"
    }

    #[inline]
    pub fn write(&mut self, part: impl AsRef<str>) -> &mut Self {
        if self.started {
            self.buf.push(' ');
            self.buf.push_str(part.as_ref());
        } else {
            self.buf.push(' ');
            self.buf.push_str(part.as_ref());
            self.started = true;
        }

        self
    }

    #[inline]
    pub fn pair(&mut self, key: &'static str, value: impl AsRef<str>) -> &mut Self {
        if key.is_empty() {
            self.write(value)
        } else {
            self.write(format!("{}={}", key, value.as_ref()))
        }
    }

    #[inline]
    pub fn quote(&mut self, key: &'static str, value: impl AsRef<str>) -> &mut Self {
        if key.is_empty() {
            self.write(format!("'{}'", value.as_ref()))
        } else {
            self.write(format!("{}='{}'", key, value.as_ref()))
        }
    }

    #[inline]
    pub fn iden(&mut self, key: &'static str, value: &sea_query::DynIden) -> &mut Self {
        self.quote(key, value.to_string())
    }

    #[inline]
    pub fn debug(&mut self, key: &'static str, value: impl std::fmt::Debug) -> &mut Self {
        self.pair(key, format!("{value:?}"))
    }

    #[inline]
    pub fn display(&mut self, key: &'static str, value: impl std::fmt::Display) -> &mut Self {
        self.pair(key, value.to_string())
    }

    #[inline]
    pub fn map<T, F, R>(&mut self, key: &'static str, value: T, func: F) -> &mut Self
    where
        R: AsRef<str>,
        F: FnOnce(T) -> R,
    {
        self.pair(key, func(value))
    }

    #[inline]
    pub fn optional_pair(
        &mut self,
        key: &'static str,
        value: Option<impl AsRef<str>>,
    ) -> &mut Self {
        match value {
            Some(x) => self.pair(key, x),
            None => self,
        }
    }

    #[inline]
    pub fn optional_quote(
        &mut self,
        key: &'static str,
        value: Option<impl AsRef<str>>,
    ) -> &mut Self {
        match value {
            Some(x) => self.quote(key, x),
            None => self,
        }
    }

    #[inline]
    pub fn optional_iden(
        &mut self,
        key: &'static str,
        value: Option<&sea_query::DynIden>,
    ) -> &mut Self {
        match value {
            Some(x) => self.iden(key, x),
            None => self,
        }
    }

    #[inline]
    pub fn optional_debug(
        &mut self,
        key: &'static str,
        value: Option<impl std::fmt::Debug>,
    ) -> &mut Self {
        match value {
            Some(x) => self.debug(key, x),
            None => self,
        }
    }

    #[inline]
    pub fn optional_display(
        &mut self,
        key: &'static str,
        value: Option<impl std::fmt::Display>,
    ) -> &mut Self {
        match value {
            Some(x) => self.display(key, x),
            None => self,
        }
    }

    #[inline]
    pub fn optional_boolean(&mut self, key: &'static str, value: bool) -> &mut Self {
        if value {
            self.pair(key, "true")
        } else {
            self
        }
    }

    #[inline]
    pub fn optional_map<T, F, R>(
        &mut self,
        key: &'static str,
        value: Option<T>,
        func: F,
    ) -> &mut Self
    where
        R: AsRef<str>,
        F: FnOnce(T) -> R,
    {
        match value {
            Some(x) => self.map(key, x, func),
            None => self,
        }
    }
}

impl ReprVec {
    #[inline]
    pub fn vec(&self, index: usize) -> ReprNestedVec {
        ReprNestedVec {
            index,
            items: String::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, index: usize, item: impl AsRef<str>) -> &mut Self {
        if index > 0 {
            self.items.push_str(", ");
        }
        self.items.push_str(item.as_ref());
        self
    }

    #[inline]
    pub fn quote(&mut self, index: usize, item: impl AsRef<str>) -> &mut Self {
        if index > 0 {
            self.items.push_str(", ");
        }
        self.items.push('\'');
        self.items.push_str(item.as_ref());
        self.items.push('\'');
        self
    }

    #[inline]
    pub fn quote_iter<I, D>(&mut self, iterator: I) -> &mut Self
    where
        D: AsRef<str>,
        I: Iterator<Item = D>,
    {
        for (index, item) in iterator.enumerate() {
            self.quote(index, item);
        }
        self
    }

    #[inline]
    pub fn display_iter<I, D>(&mut self, iterator: I) -> &mut Self
    where
        D: std::fmt::Display,
        I: Iterator<Item = D>,
    {
        for (index, item) in iterator.enumerate() {
            self.push(index, item.to_string());
        }
        self
    }

    pub fn finish(&mut self, fmt: &mut ReprFormatter) {
        if self.optional && self.items.is_empty() {
            return;
        }

        self.items.insert(0, '[');
        self.items.push(']');
        fmt.pair(self.key, std::mem::take(&mut self.items));
    }
}

impl ReprNestedVec {
    #[inline]
    pub fn push(&mut self, index: usize, item: impl AsRef<str>) -> &mut Self {
        if index > 0 {
            self.items.push_str(", ");
        }
        self.items.push_str(item.as_ref());
        self
    }

    #[inline]
    pub fn quote(&mut self, index: usize, item: impl AsRef<str>) -> &mut Self {
        if index > 0 {
            self.items.push_str(", ");
        }
        self.items.push('\'');
        self.items.push_str(item.as_ref());
        self.items.push('\'');
        self
    }

    #[inline]
    pub fn quote_iter<I, D>(&mut self, iterator: I) -> &mut Self
    where
        D: AsRef<str>,
        I: Iterator<Item = D>,
    {
        for (index, item) in iterator.enumerate() {
            self.quote(index, item);
        }
        self
    }

    #[inline]
    pub fn display_iter<I, D>(&mut self, iterator: I) -> &mut Self
    where
        D: std::fmt::Display,
        I: Iterator<Item = D>,
    {
        for (index, item) in iterator.enumerate() {
            self.push(index, item.to_string());
        }
        self
    }

    pub fn finish(&mut self, fmtvec: &mut ReprVec) {
        self.items.insert(0, '[');
        self.items.push(']');
        fmtvec.push(self.index, std::mem::take(&mut self.items));
    }
}
