//! Assisted-by: Claude

pub trait ReplacePattern: Copy {
    fn find_in(self, haystack: &str) -> Option<(usize, usize)>;

    fn pattern_len_hint(self) -> Option<usize>;
}

impl ReplacePattern for char {
    #[inline]
    fn find_in(self, haystack: &str) -> Option<(usize, usize)> {
        haystack.find(self).map(|i| (i, self.len_utf8()))
    }

    #[inline]
    fn pattern_len_hint(self) -> Option<usize> {
        Some(self.len_utf8())
    }
}

impl ReplacePattern for &str {
    #[inline]
    fn find_in(self, haystack: &str) -> Option<(usize, usize)> {
        haystack.find(self).map(|i| (i, self.len()))
    }

    #[inline]
    fn pattern_len_hint(self) -> Option<usize> {
        Some(self.len())
    }
}

pub fn str_replace_to<P, W>(haystack: &str, from: P, to: &str, w: &mut W) -> std::fmt::Result
where
    P: ReplacePattern,
    W: std::fmt::Write,
{
    let from_ascii_byte: Option<u8> = match from.pattern_len_hint() {
        Some(1) => from.find_in(haystack).and_then(|(start, len)| {
            if len == 1 {
                haystack.as_bytes().get(start).copied()
            } else {
                None
            }
        }),
        _ => None,
    };

    if let (Some(from_byte), [to_byte]) = (from_ascii_byte, to.as_bytes()) {
        return replace_ascii_to_writer(haystack.as_bytes(), from_byte, *to_byte, w);
    }

    let mut last_end = 0usize;
    let mut search_start = 0usize;
    while let Some((rel_start, len)) = from.find_in(&haystack[search_start..]) {
        let start = search_start + rel_start;
        w.write_str(&haystack[last_end..start])?;
        w.write_str(to)?;
        last_end = start + len;
        search_start = last_end;
        if len == 0 {
            break;
        }
    }
    w.write_str(&haystack[last_end..])?;
    Ok(())
}

#[inline]
fn replace_ascii_to_writer<W: std::fmt::Write>(
    haystack: &[u8],
    from_byte: u8,
    to_byte: u8,
    w: &mut W,
) -> std::fmt::Result {
    let mut last_end = 0usize;
    for (i, &b) in haystack.iter().enumerate() {
        if b == from_byte {
            let chunk = unsafe { haystack.get_unchecked(last_end..i) };
            w.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })?;
            w.write_char(to_byte as char)?;
            last_end = i + 1;
        }
    }
    let chunk = unsafe { haystack.get_unchecked(last_end..haystack.len()) };
    w.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })?;
    Ok(())
}

use std::fmt::{self, Write};

/// Replaces multiple single ASCII bytes in one pass over the string,
/// writing directly to w with no intermediate allocation.
///
/// Equivalent to chaining .replace(byte_i as char, to_i) for each pair,
/// but scans the input once instead of N times.
///
/// # Constraints
/// - Each from must be a single ASCII byte (0..=127).
/// - to can be any string, including empty (to delete).
/// - If a byte appears more than once in replacements, the last one wins.
///
/// # Complexity
/// O(n + k): n = input length, k = number of replacement pairs
/// (lookup table built once, then O(1) per input byte).
pub fn replace_many_ascii_to<W: Write>(
    s: &str,
    replacements: &[(u8, &str)],
    w: &mut W,
) -> fmt::Result {
    debug_assert!(
        replacements.iter().all(|(b, _)| b.is_ascii()),
        "all from values must be ASCII bytes"
    );

    // Fixed-size 256-entry lookup table: table[b] = Some(index into replacements) or None.
    // Lives on the stack, avoids a HashMap allocation.
    let mut table: [Option<u16>; 256] = [None; 256];
    for (idx, &(byte, _)) in replacements.iter().enumerate() {
        assert!(idx <= u16::MAX as usize, "too many replacement pairs");
        table[byte as usize] = Some(idx as u16);
    }

    let bytes = s.as_bytes();
    let mut last_end = 0usize;

    for (i, &b) in bytes.iter().enumerate() {
        if let Some(idx) = table[b as usize] {
            let chunk = unsafe { bytes.get_unchecked(last_end..i) };
            w.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })?;
            w.write_str(replacements[idx as usize].1)?;
            last_end = i + 1;
        }
    }

    let chunk = unsafe { bytes.get_unchecked(last_end..bytes.len()) };
    w.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace_many_ascii(s: &str, replacements: &[(u8, &str)]) -> String {
        let mut out = String::with_capacity(s.len());
        replace_many_ascii_to(s, replacements, &mut out)
            .expect("String::write_str should never fail");
        out
    }

    #[test]
    fn str_pattern() {
        let mut s = String::new();
        str_replace_to("hello world", "o", "0", &mut s).unwrap();
        assert_eq!(s, "hell0 w0rld");
    }

    #[test]
    fn char_pattern() {
        let mut s = String::new();
        str_replace_to("banana", 'a', "AA", &mut s).unwrap();
        assert_eq!(s, "bAAnAAnAA");
    }
    #[test]
    fn ascii_fast_path() {
        let mut s = String::new();
        str_replace_to("banana", 'a', "_", &mut s).unwrap();
        assert_eq!(s, "b_n_n_");
    }

    #[test]
    fn no_match() {
        let mut s = String::new();
        str_replace_to("hello", "z", "x", &mut s).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn unicode() {
        let mut s = String::new();
        str_replace_to("café café", "é", "e", &mut s).unwrap();
        assert_eq!(s, "cafe cafe");
    }

    #[test]
    fn multi_char_string_pattern() {
        let mut s = String::new();
        str_replace_to("aaaa", "aa", "b", &mut s).unwrap();
        assert_eq!(s, "bb");
    }

    #[test]
    fn quote_and_nul() {
        let input = "O'Brien\0's \0dog's\0 name";
        let expected = input.replace('\'', "''").replace('\0', "");
        let got = replace_many_ascii(input, &[(b'\'', "''"), (b'\0', "")]);
        assert_eq!(got, expected);
    }

    #[test]
    fn single_replacement_matches_naive() {
        let input = "hello world";
        let expected = input.replace('o', "0");
        let got = replace_many_ascii(input, &[(b'o', "0")]);
        assert_eq!(got, expected);
    }

    #[test]
    fn no_matches() {
        let input = "hello world";
        let got = replace_many_ascii(input, &[(b'z', "Z"), (b'x', "X")]);
        assert_eq!(got, input);
    }

    #[test]
    fn empty_input() {
        let got = replace_many_ascii("", &[(b'a', "b")]);
        assert_eq!(got, "");
    }

    #[test]
    fn empty_replacements_list() {
        let input = "unchanged string";
        let got = replace_many_ascii(input, &[]);
        assert_eq!(got, input);
    }

    #[test]
    fn many_pairs_at_once() {
        let input = "a-b_c.d,e;f:g";
        let expected = input
            .replace('-', "_DASH_")
            .replace('.', "_DOT_")
            .replace(',', "_COMMA_")
            .replace(';', "_SEMI_")
            .replace(':', "_COLON_");

        // Note: chained .replace() calls apply sequentially (later calls can
        // match text produced by earlier ones), while replace_many applies
        // all pairs simultaneously against the original input. This test
        // uses a non-overlapping scenario so both give the same result:
        let got = replace_many_ascii(
            input,
            &[
                (b'-', "_DASH_"),
                (b'.', "_DOT_"),
                (b',', "_COMMA_"),
                (b';', "_SEMI_"),
                (b':', "_COLON_"),
            ],
        );
        // Since replace_many is single-pass over the original input (not
        // sequential), the simultaneous result matches the chained
        // expected here because none of the patterns overlap.
        assert_eq!(got, expected);
    }

    #[test]
    fn unicode_content_preserved() {
        let input = "café\0naïve'test";
        let expected = input.replace('\'', "''").replace('\0', "");
        let got = replace_many_ascii(input, &[(b'\'', "''"), (b'\0', "")]);
        assert_eq!(got, expected);
    }

    #[test]
    fn last_duplicate_wins() {
        // if a byte appears twice in replacements, the last one wins
        let got = replace_many_ascii("abc", &[(b'a', "X"), (b'a', "Y")]);
        assert_eq!(got, "Ybc");
    }
}
