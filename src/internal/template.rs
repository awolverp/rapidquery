//! Allocation-conscious string replacement that writes directly to a [`fmt::Write`].

// FIXME: Use `std::str::pattern::Pattern` when it's stable
/// A search needle that can be located inside a UTF-8 haystack.
pub trait ReplacePattern: Copy {
    /// First match of this pattern in `haystack`.
    ///
    /// Returns `(byte_offset, match_len_in_bytes)`.
    fn find(self, haystack: &str) -> Option<(usize, usize)>;

    /// If this pattern is exactly one ASCII byte, return that byte.
    ///
    /// Used to prove that match boundaries are UTF-8 char boundaries without
    /// inspecting the haystack.
    fn as_ascii_byte(self) -> Option<u8> {
        None
    }
}

impl ReplacePattern for char {
    #[inline]
    fn find(self, haystack: &str) -> Option<(usize, usize)> {
        haystack.find(self).map(|i| (i, self.len_utf8()))
    }

    #[inline]
    fn as_ascii_byte(self) -> Option<u8> {
        self.is_ascii().then_some(self as u8)
    }
}

impl ReplacePattern for &str {
    #[inline]
    fn find(self, haystack: &str) -> Option<(usize, usize)> {
        // An empty needle matches at every byte offset and would loop forever
        // (or insert between every character). Treat it as "no match".
        if self.is_empty() {
            return None;
        }
        haystack.find(self).map(|i| (i, self.len()))
    }

    #[inline]
    fn as_ascii_byte(self) -> Option<u8> {
        let bytes = self.as_bytes();
        match bytes {
            [b] if b.is_ascii() => Some(*b),
            _ => None,
        }
    }
}
/// Replace every occurrence of `from` with `to`, writing the result to `out`.
///
/// Matches are non-overlapping and searched left to right, same as [`str::replace`].
/// Empty `&str` needles are ignored (no infinite loop, input is copied as-is).
#[inline]
pub fn replace_to<P, W>(haystack: &str, from: P, to: &str, out: &mut W) -> std::fmt::Result
where
    P: ReplacePattern,
    W: std::fmt::Write,
{
    // Single ASCII byte → single ASCII byte: still go through the generic loop.
    // `str::find(char)` already uses memchr, which beats a hand-rolled byte scan
    // when matches are sparse, and the generic loop is correct for every other case.

    let mut last_end = 0;
    let mut search_at = 0;

    while let Some((rel, len)) = from.find(&haystack[search_at..]) {
        let start = search_at + rel;
        out.write_str(&haystack[last_end..start])?;
        out.write_str(to)?;
        last_end = start + len;
        search_at = last_end;

        // Defensive: a zero-width match would otherwise spin.
        if len == 0 {
            break;
        }
    }

    out.write_str(&haystack[last_end..])
}

#[derive(Debug, Clone, Copy, derive_more::Display)]
pub enum TemplateParseError {
    #[display("template is too long (maximum is 256 characters)")]
    TemplateIsTooLong,
    #[display("one of required keys not found: {_0}")]
    RequiredKeyNotFound(&'static str),
    #[display("placeholders are missing (at least {_0} placeholder is required, {_1} found)")]
    MissingPlaceholder(u8, usize),
}
impl std::error::Error for TemplateParseError {}

#[derive(Debug, Clone, Copy)]
struct TemplateMatches {
    start: u16,
    end: u16,
    key: &'static str,
}

/// A small, allocation-conscious template that expands `{key}` placeholders
/// by writing directly to a [`fmt::Write`].
///
/// The template string is stored as an [`ArcStr`]. Placeholder positions are
/// stored as plain numbers. Keys are `'static`.
#[derive(Debug, Clone)]
pub struct Template {
    tmpl: arcstr::ArcStr,
    matches: smallvec::SmallVec<[TemplateMatches; 1]>,
}

impl Template {
    /// Parses a template string, locating every `{key}` whose name appears in `keys`.
    ///
    /// # Arguments
    ///
    /// * `tmpl` - The template text.
    /// * `keys` - Static list of `(name, required)`.
    /// * `minimum` - Minimum number of placeholders that must be found.
    ///
    /// # Errors
    ///
    /// - Template longer than 256 bytes.
    /// - Fewer than `minimum` placeholders found.
    /// - A required key is missing.
    #[cold]
    pub fn parse(tmpl: &str, keys: &[(&'static str, bool)], minimum: u8) -> Result<Self, TemplateParseError> {
        debug_assert!(keys.len() <= 16, "too many keys");

        if tmpl.len() > 256 {
            return Err(TemplateParseError::TemplateIsTooLong);
        }

        let mut matches: smallvec::SmallVec<[TemplateMatches; 1]> = smallvec::SmallVec::new();
        let bytes = tmpl.as_bytes();
        let mut required_mask = 0usize;
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b'{'
                && let Some(rel_end) = tmpl[i + 1..].find('}')
            {
                let key = &tmpl[i + 1..i + 1 + rel_end];

                if let Some(idx) = keys.iter().position(|(k, _)| *k == key) {
                    let start = i;
                    let end = i + 1 + rel_end + 1;

                    matches.push(TemplateMatches {
                        start: start as u16,
                        end: end as u16,
                        key: keys[idx].0,
                    });

                    if keys[idx].1 {
                        required_mask |= 1 << idx;
                    }

                    i = end;
                    continue;
                }
            }

            i += 1;
        }

        if matches.len() < minimum as usize {
            return Err(TemplateParseError::MissingPlaceholder(minimum, matches.len()));
        }

        for (idx, (key, required)) in keys.iter().enumerate() {
            if *required && (required_mask & (1 << idx)) == 0 {
                return Err(TemplateParseError::RequiredKeyNotFound(key));
            }
        }

        Ok(Self {
            tmpl: tmpl.into(),
            matches,
        })
    }

    /// Writes the expanded template to `out`.
    ///
    /// `fill` is called for each placeholder with its key name and the writer.
    /// It is responsible for writing the replacement (or nothing).
    #[inline]
    #[cfg_attr(feature = "nightly", optimize(speed))]
    pub fn write_to<W, F>(&self, out: &mut W, mut fill: F) -> std::fmt::Result
    where
        W: std::fmt::Write,
        F: FnMut(&'static str, &mut W) -> std::fmt::Result,
    {
        let s = self.tmpl.as_str();
        let mut last: u16 = 0;

        for m in self.matches.iter() {
            if last < m.start {
                out.write_str(&s[last as usize..m.start as usize])?;
            }

            fill(m.key, out)?;
            last = m.end;
        }

        if (last as usize) < s.len() {
            out.write_str(&s[last as usize..])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    fn replace(haystack: &str, from: impl ReplacePattern, to: &str) -> String {
        let mut out = String::with_capacity(haystack.len());
        replace_to(haystack, from, to, &mut out).unwrap();
        out
    }

    #[test]
    fn str_pattern() {
        assert_eq!(replace("hello world", "o", "0"), "hell0 w0rld");
    }

    #[test]
    fn char_pattern() {
        assert_eq!(replace("banana", 'a', "AA"), "bAAnAAnAA");
    }

    #[test]
    fn ascii_char_to_ascii_str() {
        assert_eq!(replace("banana", 'a', "_"), "b_n_n_");
    }

    #[test]
    fn no_match() {
        assert_eq!(replace("hello", "z", "x"), "hello");
    }

    #[test]
    fn unicode_pattern() {
        assert_eq!(replace("café café", "é", "e"), "cafe cafe");
    }

    #[test]
    fn multi_char_string_pattern() {
        assert_eq!(replace("aaaa", "aa", "b"), "bb");
    }

    #[test]
    fn overlapping_is_non_overlapping_left_to_right() {
        assert_eq!(replace("aaa", "aa", "b"), "ba");
    }

    #[test]
    fn empty_haystack() {
        assert_eq!(replace("", "a", "b"), "");
    }

    #[test]
    fn empty_needle_is_ignored() {
        assert_eq!(replace("keep", "", "x"), "keep");
    }

    #[test]
    fn delete_via_empty_replacement() {
        assert_eq!(replace("a_b_c", "_", ""), "abc");
    }

    #[test]
    fn replacement_longer_than_match() {
        assert_eq!(replace("ab", "b", "BBB"), "aBBB");
    }

    #[test]
    fn matches_std_replace_for_char() {
        let samples = ["", "a", "aaa", "café", "hello\0world", "🇺🇸"];
        for s in samples {
            assert_eq!(replace(s, 'a', "AA"), s.replace('a', "AA"));
            assert_eq!(replace(s, "l", "_"), s.replace("l", "_"));
        }
    }

    #[test]
    fn single_required_key() {
        let t = Template::parse("{hex}::BLOB", &[("hex", true), ("raw", false)], 1).unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |key, w| {
            if key == "hex" {
                w.write_str("0xDEADBEEF")
            } else {
                Ok(())
            }
        })
        .unwrap();

        assert_eq!(out, "0xDEADBEEF::BLOB");
    }

    #[test]
    fn multiple_keys() {
        let t = Template::parse(
            "regexp_matches({expr}, {pattern})",
            &[("expr", true), ("pattern", true), ("flags", false)],
            2,
        )
        .unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |key, w| match key {
            "expr" => w.write_str("col"),
            "pattern" => w.write_str("'^foo.*'"),
            _ => Ok(()),
        })
        .unwrap();

        assert_eq!(out, "regexp_matches(col, '^foo.*')");
    }

    #[test]
    fn optional_key_omitted() {
        let t = Template::parse("func({a}, {b})", &[("a", true), ("b", false)], 1).unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |key, w| if key == "a" { w.write_str("1") } else { Ok(()) })
            .unwrap();

        assert_eq!(out, "func(1, )");
    }

    #[test]
    fn required_key_missing_at_parse_fails() {
        let err = Template::parse("no placeholders", &[("missing", true)], 0);
        assert!(matches!(err, Err(TemplateParseError::RequiredKeyNotFound("missing"))));
    }

    #[test]
    fn minimum_placeholders_not_met() {
        let err = Template::parse("only {one}", &[("one", false)], 2);
        assert!(matches!(err, Err(TemplateParseError::MissingPlaceholder(2, 1))));
    }

    #[test]
    fn unknown_placeholders_are_left_literal() {
        let t = Template::parse("hello {unknown} world", &[("known", false)], 0).unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |_, _| Ok(())).unwrap();

        assert_eq!(out, "hello {unknown} world");
    }

    #[test]
    fn repeated_keys() {
        let t = Template::parse("{x} + {x}", &[("x", true)], 2).unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |_, w| w.write_str("1")).unwrap();

        assert_eq!(out, "1 + 1");
    }

    #[test]
    fn empty_template() {
        let t = Template::parse("", &[], 0).unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |_, _| Ok(())).unwrap();

        assert_eq!(out, "");
    }

    #[test]
    fn no_placeholders() {
        let t = Template::parse("plain text", &[("unused", false)], 0).unwrap();

        let mut out = String::new();
        t.write_to(&mut out, |_, _| Ok(())).unwrap();

        assert_eq!(out, "plain text");
    }

    #[test]
    fn template_too_long() {
        let long = "a".repeat(257);
        let err = Template::parse(&long, &[], 0);
        assert!(matches!(err, Err(TemplateParseError::TemplateIsTooLong)));
    }
}
