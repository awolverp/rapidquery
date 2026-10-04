//! Allocation-conscious string replacement that writes directly to a [`fmt::Write`].
// Assisted-by: Grok

use std::fmt::Write;
use std::fmt::{self};

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
pub fn replace_to<P, W>(haystack: &str, from: P, to: &str, out: &mut W) -> fmt::Result
where
    P: ReplacePattern,
    W: Write,
{
    // Single ASCII byte → single ASCII byte: still go through the generic loop.
    // `str::find(char)` already uses memchr, which beats a hand-rolled byte scan
    // when matches are sparse, and the generic loop is correct for every other case.
    let _ = from.as_ascii_byte();

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

/// Replace multiple single ASCII bytes in one pass over `haystack`.
///
/// Equivalent to applying each pair as a simultaneous substitution against the
/// *original* input (not a chain of sequential [`str::replace`] calls). If the
/// same byte appears more than once, the last pair wins.
///
/// # Constraints
/// - Each key must be an ASCII byte (`0..=127`). Non-ASCII keys are skipped so
///   that splits cannot land inside a multi-byte UTF-8 scalar.
/// - Replacement values may be any string, including empty (deletion).
///
/// # Complexity
/// O(n + k) where n is the input length and k is the number of pairs.
pub fn replace_ascii_to<W: Write>(haystack: &str, replacements: &[(u8, &str)], out: &mut W) -> fmt::Result {
    if replacements.is_empty() {
        return out.write_str(haystack);
    }

    // table[b] == 0 → no replacement
    // table[b] == i + 1 → replacements[i]
    // 512 bytes on the stack, no heap, last write wins.
    let mut table = [0u16; 256];
    for (i, &(byte, _)) in replacements.iter().enumerate() {
        if !byte.is_ascii() {
            continue;
        }
        table[byte as usize] = i as u16 + 1;
    }

    let bytes = haystack.as_bytes();
    let mut last_end = 0;

    for (i, &b) in bytes.iter().enumerate() {
        let slot = table[b as usize];
        if slot == 0 {
            continue;
        }

        write_utf8_prefix(out, bytes, last_end, i)?;
        out.write_str(replacements[slot as usize - 1].1)?;
        last_end = i + 1;
    }

    write_utf8_prefix(out, bytes, last_end, bytes.len())
}

/// Write `bytes[from..to]` as a string.
///
/// # Safety invariant
/// `bytes` is the raw view of a valid `&str`, and `from..to` only starts/ends on
/// ASCII bytes or the slice ends — hence a char boundary.
#[inline]
fn write_utf8_prefix<W: Write>(out: &mut W, bytes: &[u8], from: usize, to: usize) -> fmt::Result {
    if from == to {
        return Ok(());
    }
    // SAFETY: see invariant above. Callers only advance `from` past ASCII matches
    // or to `bytes.len()`.
    let chunk = unsafe { bytes.get_unchecked(from..to) };
    out.write_str(unsafe { std::str::from_utf8_unchecked(chunk) })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace(haystack: &str, from: impl ReplacePattern, to: &str) -> String {
        let mut out = String::with_capacity(haystack.len());
        replace_to(haystack, from, to, &mut out).unwrap();
        out
    }

    fn replace_ascii(s: &str, replacements: &[(u8, &str)]) -> String {
        let mut out = String::with_capacity(s.len());
        replace_ascii_to(s, replacements, &mut out).unwrap();
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
    fn quote_and_nul() {
        let input = "O'Brien\0's \0dog's\0 name";
        let expected = input.replace('\'', "''").replace('\0', "");
        let got = replace_ascii(input, &[(b'\'', "''"), (b'\0', "")]);
        assert_eq!(got, expected);
    }

    #[test]
    fn single_replacement_matches_naive() {
        let input = "hello world";
        assert_eq!(replace_ascii(input, &[(b'o', "0")]), input.replace('o', "0"));
    }

    #[test]
    fn no_matches_ascii() {
        assert_eq!(replace_ascii("hello world", &[(b'z', "Z"), (b'x', "X")]), "hello world");
    }

    #[test]
    fn empty_input_ascii() {
        assert_eq!(replace_ascii("", &[(b'a', "b")]), "");
    }

    #[test]
    fn empty_replacements_list() {
        assert_eq!(replace_ascii("unchanged string", &[]), "unchanged string");
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

        let got = replace_ascii(
            input,
            &[
                (b'-', "_DASH_"),
                (b'.', "_DOT_"),
                (b',', "_COMMA_"),
                (b';', "_SEMI_"),
                (b':', "_COLON_"),
            ],
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn unicode_content_preserved() {
        let input = "café\0naïve'test";
        let expected = input.replace('\'', "''").replace('\0', "");
        assert_eq!(replace_ascii(input, &[(b'\'', "''"), (b'\0', "")]), expected);
    }

    #[test]
    fn last_duplicate_wins() {
        assert_eq!(replace_ascii("abc", &[(b'a', "X"), (b'a', "Y")]), "Ybc");
    }

    #[test]
    fn non_ascii_keys_are_skipped() {
        // 0xC3 is a UTF-8 lead byte in "é". Treating it as a split point would be UB.
        assert_eq!(replace_ascii("café", &[(0xC3, "X")]), "café");
    }

    #[test]
    fn matches_std_replace_for_char() {
        let samples = ["", "a", "aaa", "café", "hello\0world", "🇺🇸"];
        for s in samples {
            assert_eq!(replace(s, 'a', "AA"), s.replace('a', "AA"));
            assert_eq!(replace(s, "l", "_"), s.replace("l", "_"));
        }
    }
}
