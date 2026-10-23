use std::str::FromStr;

use super::*;

#[inline]
fn bool_flags(format: config::BoolFormat) -> config::DialectConfig {
    config::DialectConfig {
        bool_format: format,
        ..Default::default()
    }
}

#[inline]
fn string_flags(format: config::StringFormat, escape: config::StringEscape) -> config::DialectConfig {
    config::DialectConfig {
        string_format: format,
        string_escape: escape,
        ..Default::default()
    }
}

#[inline]
fn ident_flags(format: config::IdentifierFormat, escape: config::IdentifierEscape) -> config::DialectConfig {
    config::DialectConfig {
        identifier_format: format,
        identifier_escape: escape,
        ..Default::default()
    }
}

fn write_str(flags: &config::DialectConfig, raw: &str) -> Result<String, errors::FormatError> {
    let mut out = String::new();
    flags.write_string_literal(&mut out, raw)?;
    Ok(out)
}

fn escape(flags: &config::DialectConfig, raw: &str) -> String {
    let mut out = String::new();
    flags.escape_string(&mut out, raw).unwrap();
    out
}

fn quote_ident(flags: &config::DialectConfig, ident: &str) -> String {
    let mut out = String::new();
    flags.write_identifier(&mut out, ident).unwrap();
    out
}

/// A writer that fails on the Nth `write_str` / `write_char` (1-based).
struct FailAfter {
    remaining: usize,
}

impl std::fmt::Write for FailAfter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if s.is_empty() {
            return Ok(());
        }
        if self.remaining == 0 {
            return Err(std::fmt::Error);
        }
        self.remaining -= 1;
        Ok(())
    }
}

/// Standard-SQL string lexer: quotes doubled, no backslash escapes.
/// Returns the unescaped body if `sql` is exactly one literal.
fn parse_quote_doubled(sql: &str) -> Option<String> {
    let rest = sql.strip_prefix('\'')?;
    let mut body = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            if chars.peek() == Some(&'\'') {
                chars.next();
                body.push('\'');
            } else {
                return chars.next().is_none().then_some(body);
            }
        } else {
            body.push(ch);
        }
    }
    None
}

/// MySQL-style backslash lexer (NO_BACKSLASH_ESCAPES off). Unknown escapes
/// keep the escaped byte. Returns the body if the literal closes exactly once.
fn parse_backslash(sql: &str) -> Option<String> {
    let rest = sql.strip_prefix('\'')?;
    let bytes = rest.as_bytes();
    let mut i = 0;
    let mut body = Vec::new();
    while i < bytes.len() {
        match bytes[i] {
            b'\'' => {
                i += 1;
                return (i == bytes.len()).then(|| String::from_utf8(body).ok()).flatten();
            }
            b'\\' => {
                if i + 1 >= bytes.len() {
                    return None;
                }
                body.push(bytes[i + 1]);
                i += 2;
            }
            b => {
                body.push(b);
                i += 1;
            }
        }
    }
    None
}

fn assert_no_early_close_quote(escaped: &str) {
    let mut chars = escaped.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' {
            assert_eq!(
                chars.next(),
                Some('\''),
                "odd quote in quote-doubled escape: {escaped:?}"
            );
        }
    }
}

const BREAKOUTS: &[&str] = &[
    "'",
    "''",
    "' OR '1'='1",
    "' OR 1=1 --",
    "'; DROP TABLE users; --",
    "\\",
    "\\'",
    "\\\\",
    "'''",
    "admin'--",
    "1' AND SLEEP(1)--",
    "\" OR \"1\"=\"1",
    "') OR ('1'='1",
    "\n' OR 1=1 --",
    "\r\n",
    "foo\tbar",
    "%' OR 1=1 --",
    "_",
    "/*",
    "*/",
    "--",
    "#",
    "\u{1a}",
    "\u{7f}",
    "\u{01}",
    "café",
    "I ❤️ SQL",
    "\u{202e}1=1 RO",
    "’",  // U+2019, not a SQL quote
    "＇", // U+FF0C fullwidth is not the quote; fullwidth apostrophe is U+FF07
    "＇",
    "\u{ff07}",
];

#[test]
fn write_boolean_keyword_and_integer() {
    let keyword = bool_flags(config::BoolFormat::Keyword);
    let integer = bool_flags(config::BoolFormat::Integer);

    let mut out = String::new();
    keyword.write_boolean(&mut out, true).unwrap();
    keyword.write_boolean(&mut out, false).unwrap();
    assert_eq!(out, "TRUEFALSE");

    out.clear();
    integer.write_boolean(&mut out, true).unwrap();
    integer.write_boolean(&mut out, false).unwrap();
    assert_eq!(out, "10");
}

#[test]
fn write_boolean_does_not_emit_quotes_or_lowercase() {
    // MySQL may accept the integers; neither spelling is a string, so a
    // caller that interpolates them into a quoted context would be wrong.
    for value in [false, true] {
        let mut keyword = String::new();
        let mut integer = String::new();
        bool_flags(config::BoolFormat::Keyword)
            .write_boolean(&mut keyword, value)
            .unwrap();

        bool_flags(config::BoolFormat::Integer)
            .write_boolean(&mut integer, value)
            .unwrap();

        assert!(!keyword.contains('\''));
        assert!(!integer.contains('\''));
        assert_eq!(keyword, keyword.to_ascii_uppercase());
        assert!(integer == "0" || integer == "1");
    }
}

#[test]
fn write_boolean_propagates_writer_errors() {
    let mut writer = FailAfter { remaining: 0 };
    let err = bool_flags(config::BoolFormat::Keyword)
        .write_boolean(&mut writer, true)
        .unwrap_err();

    assert_eq!(err, errors::FormatError::Write);
}

#[test]
fn escape_string_quote_doubles_and_leaves_the_rest() {
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::Quote);
    assert_eq!(escape(&flags, ""), "");
    assert_eq!(escape(&flags, "plain"), "plain");
    assert_eq!(escape(&flags, "'"), "''");
    assert_eq!(escape(&flags, "''"), "''''");
    assert_eq!(escape(&flags, "a'b'c"), "a''b''c");
    // Backslash is data under standard SQL. Leaving it alone is correct for
    // Postgres and wrong for MySQL with backslash escapes enabled.
    assert_eq!(escape(&flags, "\\'"), "\\''");
    assert_eq!(escape(&flags, "\0"), "\0");
    assert_eq!(escape(&flags, "\n\r\t"), "\n\r\t");
    assert_eq!(escape(&flags, "100%_"), "100%_");
    assert_eq!(escape(&flags, "雪's"), "雪''s");
}

#[test]
fn escape_string_backslash_hex_covers_c0_and_del() {
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::BackslashHex);
    assert_eq!(escape(&flags, "\\"), "\\\\");
    assert_eq!(escape(&flags, "'"), "\\'");
    assert_eq!(escape(&flags, "\n\r\t"), "\\n\\r\\t");
    assert_eq!(escape(&flags, "\0"), "\\x00");
    assert_eq!(escape(&flags, "\u{1}"), "\\x01");
    assert_eq!(escape(&flags, "\u{1a}"), "\\x1A");
    assert_eq!(escape(&flags, "\u{1f}"), "\\x1F");
    assert_eq!(escape(&flags, "\u{7f}"), "\\x7F");
    // Space, DEL-adjacent printable, and non-ASCII stay raw. Splitting a
    // multibyte char here would corrupt the literal (and the unsafe span
    // writer assumes ASCII split points).
    assert_eq!(escape(&flags, " ~"), " ~");
    assert_eq!(escape(&flags, "雪's\\"), "雪\\'s\\\\");
    assert_eq!(escape(&flags, "\"quote"), "\"quote");
}

#[test]
fn escape_string_mysql_keeps_tab_and_uses_z_for_sub() {
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::BackslashMysql);

    assert_eq!(escape(&flags, "\\'\""), "\\\\\\'\\\"");
    assert_eq!(escape(&flags, "\0"), "\\0");
    assert_eq!(escape(&flags, "\n\r"), "\\n\\r");
    assert_eq!(escape(&flags, "\t"), "\t");
    assert_eq!(escape(&flags, "\u{1a}"), "\\Z");
    // Other C0 bytes pass through. A strict ANSI client may reject them;
    // MySQL's NO_BACKSLASH_ESCAPES-off lexer does not treat them as quotes.
    assert_eq!(escape(&flags, "\u{01}"), "\u{01}");
    assert_eq!(escape(&flags, "\u{7f}"), "\u{7f}");
    assert_eq!(escape(&flags, "it\'s"), "it\\'s");
}

#[test]
fn escape_string_unicode_and_hex_zero_disagree_on_nul_and_quotes() {
    let unicode = string_flags(
        config::StringFormat::SingleQuote,
        config::StringEscape::BackslashUnicode,
    );

    let unicode_quote = string_flags(
        config::StringFormat::SingleQuote,
        config::StringEscape::BackslashUnicodeQuote,
    );
    let hex_zero = string_flags(
        config::StringFormat::SingleQuote,
        config::StringEscape::BackslashHexZero,
    );

    assert_eq!(escape(&unicode, "\0"), "\\0");
    assert_eq!(escape(&unicode, "\u{01}"), "\\u0001");
    assert_eq!(escape(&unicode, "\u{1a}"), "\\u001A");
    assert_eq!(escape(&unicode, "\u{7f}"), "\\u007F");
    assert_eq!(escape(&unicode, "\"'"), "\"\\'");
    assert_eq!(escape(&unicode_quote, "\"'"), "\\\"\\'");
    assert_eq!(escape(&hex_zero, "\0"), "\\0");
    assert_eq!(escape(&hex_zero, "\u{01}"), "\\x01");
    assert_eq!(escape(&hex_zero, "\u{7f}"), "\\x7F");
    assert_eq!(escape(&hex_zero, "\t\n"), "\\t\\n");
}

#[test]
fn escape_string_does_not_wrap_or_reject_nul() {
    // NUL rejection belongs to `write_string_literal`. `escape_string` is the
    // inner transform and must stay usable for already-open literals.
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::Quote);
    let escaped = escape(&flags, "a\0b");
    assert_eq!(escaped, "a\0b");
    assert!(!escaped.starts_with('\''));
}

#[test]
fn write_string_literal_prefixes_and_empty() {
    let cases = [
        (config::StringFormat::SingleQuote, "''"),
        (config::StringFormat::PrefixBigE, "E''"),
        (config::StringFormat::PrefixBigN, "N''"),
    ];
    for (format, expected) in cases {
        let flags = string_flags(format, config::StringEscape::Quote);
        assert_eq!(write_str(&flags, "").unwrap(), expected);
        assert_eq!(write_str(&flags, "a").unwrap(), expected.replace("''", "'a'"));
    }
}

#[test]
fn write_string_literal_rejects_nul_before_writing() {
    let mut out = String::from("kept");
    let flags = string_flags(config::StringFormat::PrefixBigE, config::StringEscape::BackslashHex);
    let err = flags.write_string_literal(&mut out, "a\0b").unwrap_err();
    assert_eq!(err, errors::FormatError::NulInString);
    // C drivers (historical libmysql) truncate at NUL. Refusing before any
    // write avoids a half-emitted literal that a caller might still send.
    assert_eq!(out, "kept");
    assert!(flags.write_string_literal(&mut String::new(), "\0").is_err());
}

#[test]
fn write_string_literal_quote_mode_survives_classic_payloads() {
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::Quote);
    for payload in BREAKOUTS {
        if payload.contains('\0') {
            continue;
        }
        let sql = write_str(&flags, payload).unwrap();
        let body = parse_quote_doubled(&sql).unwrap_or_else(|| panic!("quote breakout for {payload:?} -> {sql:?}"));
        assert_eq!(body, *payload);
        assert_no_early_close_quote(&sql[1..sql.len() - 1]);
    }
    assert_eq!(write_str(&flags, "' OR '1'='1").unwrap(), "''' OR ''1''=''1'");
    assert_eq!(
        write_str(&flags, "'; DROP TABLE users; --").unwrap(),
        "'''; DROP TABLE users; --'"
    );
}

#[test]
fn write_string_literal_backslash_modes_do_not_end_on_backslash_quote() {
    // The MySQL bug: escaping only `'` as `\'` (or not escaping `\`) lets
    // `\'` close the string. Every backslash mode must escape `\` first.
    let modes = [
        config::StringEscape::BackslashHex,
        config::StringEscape::BackslashMysql,
        config::StringEscape::BackslashUnicode,
        config::StringEscape::BackslashHexZero,
        config::StringEscape::BackslashUnicodeQuote,
    ];
    for mode in modes {
        let flags = string_flags(config::StringFormat::SingleQuote, mode);
        for payload in ["\\'", "\\\\'", "'", "\\", "a\\'b", "\n' OR 1=1 --"] {
            let sql = write_str(&flags, payload).unwrap();
            assert!(
                parse_backslash(&sql).is_some(),
                "{mode:?} breakout for {payload:?} -> {sql:?}"
            );
            // `\'` must not be a closer. The backslash is escaped, then the quote.
            if payload.contains('\\') {
                assert!(sql.contains("\\\\"), "{mode:?} left a raw backslash in {sql:?}");
            }
        }
    }
}

#[test]
fn write_string_literal_quote_mode_is_not_safe_for_mysql_backslash() {
    // Documents the mismatch. Under a MySQL lexer, quote-doubling alone does
    // not neutralize a leading backslash: `\'` becomes `\''`, which is an
    // escaped quote plus a closer, i.e. an empty string and trailing junk.
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::Quote);
    let sql = write_str(&flags, "\\' OR 1=1 --").unwrap();
    assert_eq!(sql, "'\\'' OR 1=1 --'");
    assert!(parse_backslash(&sql).is_none());
}

#[test]
fn write_string_literal_national_and_escape_prefixes_still_close() {
    let e = string_flags(config::StringFormat::PrefixBigE, config::StringEscape::BackslashHex);
    let n = string_flags(config::StringFormat::PrefixBigN, config::StringEscape::Quote);
    assert_eq!(write_str(&e, "a'b\\").unwrap(), "E'a\\'b\\\\'");
    assert_eq!(write_str(&n, "a'b").unwrap(), "N'a''b'");
    // Prefix is outside the quotes. A payload must not be able to eat it.
    assert!(write_str(&e, "x").unwrap().starts_with("E'"));
    assert!(write_str(&n, "x").unwrap().starts_with("N'"));
}

#[test]
fn write_string_literal_passes_unicode_quotes_and_like_wildcards() {
    let flags = string_flags(config::StringFormat::SingleQuote, config::StringEscape::Quote);
    // U+2019 / U+FF07 are not the ASCII delimiter. Treating them as quotes
    // would corrupt data; not treating ASCII `'` as a quote would be SQLi.
    assert_eq!(write_str(&flags, "it’s").unwrap(), "'it’s'");
    assert_eq!(write_str(&flags, "\u{ff07}").unwrap(), "'\u{ff07}'");
    assert_eq!(write_str(&flags, "100%_").unwrap(), "'100%_'");
    // U+202E (right-to-left override) can spoof a log line. It is still data.
    let spoof = write_str(&flags, "\u{202e}--").unwrap();
    assert_eq!(spoof, "'\u{202e}--'");
}

#[test]
fn write_identifier_doubles_only_its_own_closer() {
    let dq = ident_flags(config::IdentifierFormat::DoubleQuote, config::IdentifierEscape::Double);

    let bt = ident_flags(config::IdentifierFormat::Backtick, config::IdentifierEscape::Double);
    let br = ident_flags(
        config::IdentifierFormat::SquareBracket,
        config::IdentifierEscape::Double,
    );

    assert_eq!(quote_ident(&dq, ""), "\"\"");
    assert_eq!(quote_ident(&dq, "users"), "\"users\"");
    assert_eq!(quote_ident(&dq, "a\"b"), "\"a\"\"b\"");
    assert_eq!(quote_ident(&dq, "\""), "\"\"\"\"");
    // The other delimiters are data. This is the Postgres / ANSI contract.
    assert_eq!(quote_ident(&dq, "a`b]c'"), "\"a`b]c'\"");

    assert_eq!(quote_ident(&bt, "a`b"), "`a``b`");
    assert_eq!(quote_ident(&bt, "`"), "````");
    assert_eq!(quote_ident(&bt, "a\"b"), "`a\"b`");

    assert_eq!(quote_ident(&br, "a]b"), "[a]]b]");
    assert_eq!(quote_ident(&br, "]"), "[]]]");
    assert_eq!(quote_ident(&br, "a[b"), "[a[b]");
    // SQL Server does not treat `[` as special inside brackets. Escaping it
    // would change the identifier.
    assert_eq!(quote_ident(&br, "dbo.users"), "[dbo.users]");
}

#[test]
fn write_identifier_rejects_delimiter_breakout() {
    let samples = [
        (
            ident_flags(config::IdentifierFormat::DoubleQuote, config::IdentifierEscape::Double),
            "\"",
            "\"",
            '"',
        ),
        (
            ident_flags(config::IdentifierFormat::Backtick, config::IdentifierEscape::Double),
            "`",
            "`",
            '`',
        ),
        (
            ident_flags(
                config::IdentifierFormat::SquareBracket,
                config::IdentifierEscape::Double,
            ),
            "]",
            "[",
            ']',
        ),
    ];
    for (flags, closer, start, end) in samples {
        for payload in [
            closer.to_string(),
            format!("{closer}{closer}"),
            format!("x{closer} OR 1=1 --"),
            format!("{closer}; DROP TABLE users; --"),
        ] {
            let sql = quote_ident(&flags, &payload);
            assert!(
                sql.starts_with(start) && sql.ends_with(end),
                "delimiters lost in {sql:?}"
            );
            // Doubled closer never leaves a raw closer at an odd position.
            let inner = &sql[start.len()..sql.len() - end.len_utf8()];
            let mut rest = inner;
            let needle = closer.to_string();
            while let Some(at) = rest.find(&needle) {
                assert!(
                    rest[at + needle.len()..].starts_with(&needle),
                    "undoubled closer in {sql:?}"
                );
                rest = &rest[at + needle.len() * 2..];
            }
        }
    }
}

#[test]
fn write_identifier_backslash_modes() {
    let dq = ident_flags(
        config::IdentifierFormat::DoubleQuote,
        config::IdentifierEscape::Backslash,
    );
    let bt = ident_flags(config::IdentifierFormat::Backtick, config::IdentifierEscape::Backslash);
    let br = ident_flags(
        config::IdentifierFormat::SquareBracket,
        config::IdentifierEscape::Backslash,
    );

    assert_eq!(quote_ident(&dq, "a\"b"), "\"a\\\"b\"");
    assert_eq!(quote_ident(&dq, "\\"), "\"\\\"");
    // Standard SQL does not treat `\` as an identifier escape. `\"` still
    // contains a raw closer, so this mode is only safe on engines that
    // consume the backslash inside quoted identifiers (MySQL ANSI_QUOTES).
    assert_eq!(quote_ident(&dq, "\""), "\"\\\"\"");

    assert_eq!(quote_ident(&bt, "a`b"), "`a\\`b`");
    assert_eq!(quote_ident(&bt, "\\"), "`\\\\`");
    assert_eq!(quote_ident(&bt, "\n\r\t"), "`\\n\\r\\t`");
    assert_eq!(quote_ident(&bt, "\u{01}"), "`\\x01`");
    assert_eq!(quote_ident(&bt, "\u{7f}"), "`\\x7F`");
    // Apostrophe is not the closer. Escaping it would change the name.
    assert_eq!(quote_ident(&bt, "it's"), "`it's`");

    assert_eq!(quote_ident(&br, "a]b"), "[a\\]b]");
    assert_eq!(quote_ident(&br, "]"), "[\\]]");
}

#[test]
fn write_identifier_trailing_backslash_can_eat_a_backslash_closer() {
    // If the engine treats `\` as an escape, a trailing backslash escapes the
    // closing delimiter and the next source token is absorbed into the name.
    // Doubling mode does not have this hole. Callers must not pair
    // `flags::IdentifierEscape::Backslash` with an engine that ignores backslashes,
    // and must reject or double a trailing `\` on engines that honor it.
    let bt = ident_flags(config::IdentifierFormat::Backtick, config::IdentifierEscape::Backslash);

    let br = ident_flags(
        config::IdentifierFormat::SquareBracket,
        config::IdentifierEscape::Backslash,
    );

    assert_eq!(quote_ident(&bt, "foo\\"), "`foo\\\\`");
    assert_eq!(quote_ident(&br, "foo\\"), "[foo\\]");
    assert!(quote_ident(&br, "foo\\").ends_with("\\]"));
}

#[test]
fn write_identifier_comments_and_dots_stay_inside() {
    let dq = ident_flags(config::IdentifierFormat::DoubleQuote, config::IdentifierEscape::Double);

    // A newline inside a quoted identifier is legal in some engines and a
    // line-comment introducer in others if the closer is lost. Doubling keeps
    // the closer, so `--` cannot comment out the rest of the statement.
    assert_eq!(quote_ident(&dq, "a\n--\nb"), "\"a\n--\nb\"");
    assert_eq!(quote_ident(&dq, "a.b"), "\"a.b\"");
    assert_eq!(quote_ident(&dq, "users"), "\"users\"");
}

#[test]
fn write_bits_spellings_and_unsupported() {
    let mut out = String::new();
    let err = config::DialectConfig {
        bit_format: config::BitFormat::Unsupported,
        ..Default::default()
    }
    .write_bits(&mut out, "101")
    .unwrap_err();

    assert_eq!(err, errors::FormatError::Unsupported("BitSerializer"));
    assert!(out.is_empty(), "unsupported must not write a partial literal");

    let cases = [
        (config::BitFormat::PrefixBigB, "B'101'"),
        (config::BitFormat::PrefixSmallB, "b'101'"),
        (config::BitFormat::PostfixBigB, "'101'B"),
    ];
    for (format, expected) in cases {
        out.clear();
        config::DialectConfig {
            bit_format: format,
            ..Default::default()
        }
        .write_bits(&mut out, "101")
        .unwrap();

        assert_eq!(out, expected);

        out.clear();
        config::DialectConfig {
            bit_format: format,
            ..Default::default()
        }
        .write_bits(&mut out, "")
        .unwrap();

        assert_eq!(out, expected.replace("101", ""));
    }
}

#[test]
fn write_bits_embeds_input_unchanged() {
    // Release builds do not re-check the alphabet. A `'` in `bits` would
    // close the literal (`B'''; DROP`), so the debug assert is the guard.
    let mut out = String::new();
    config::DialectConfig {
        bit_format: config::BitFormat::PrefixBigB,
        ..Default::default()
    }
    .write_bits(&mut out, "0")
    .unwrap();

    assert_eq!(out, "B'0'");

    out.clear();

    config::DialectConfig {
        bit_format: config::BitFormat::PostfixBigB,
        ..Default::default()
    }
    .write_bits(&mut out, "1")
    .unwrap();

    assert_eq!(out, "'1'B");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "write_bits: invalid input")]
fn write_bits_panics_on_quote_in_debug() {
    let mut out = String::new();
    let _ = config::DialectConfig {
        bit_format: config::BitFormat::PrefixBigB,
        ..Default::default()
    }
    .write_bits(&mut out, "1'; DROP TABLE t; --");
}

#[test]
fn write_bytes_every_spelling_including_empty_and_quote_byte() {
    // 0x27 is ASCII `'`. Hex spellings must emit `27`, never a raw quote.
    let quote_byte = [0x27];
    let dead = [0xDE, 0xAD, 0xBE, 0xEF];
    let cases: [(config::BytesFormat, &str, &str); 10] = [
        (config::BytesFormat::PrefixBigX, "X''", "X'27'"),
        (config::BytesFormat::PrefixZeroX, "0x", "0x27"),
        (config::BytesFormat::SlashXBytea, "'\\x'::bytea", "'\\x27'::bytea"),
        (config::BytesFormat::PrefixBigBSlashX, "B''", "B'\\x27'"),
        (config::BytesFormat::HexToRaw, "HEXTORAW('')", "HEXTORAW('27')"),
        (config::BytesFormat::Unhex, "UNHEX('')", "UNHEX('27')"),
        (
            config::BytesFormat::ToVarbyte,
            "TO_VARBYTE('', 'hex')",
            "TO_VARBYTE('27', 'hex')",
        ),
        (config::BytesFormat::PostfixXbv, "''XBV", "'27'XBV"),
        (
            config::BytesFormat::ToBinary,
            "TO_BINARY('', 'HEX')",
            "TO_BINARY('27', 'HEX')",
        ),
        (config::BytesFormat::SlashXBlob, "''::BLOB", "'\\x27'::BLOB"),
    ];
    for (format, empty, quoted) in cases {
        let flags = config::DialectConfig {
            bytes_format: format,
            ..Default::default()
        };
        let mut out = String::new();
        flags.write_bytes(&mut out, &[]).unwrap();
        assert_eq!(out, empty, "{format:?} empty");
        out.clear();
        flags.write_bytes(&mut out, &quote_byte).unwrap();
        assert_eq!(out, quoted, "{format:?} quote byte");
        assert!(
            !out.contains("''") || format == config::BytesFormat::SlashXBlob || out.starts_with("''"),
            "raw quote leaked into {out}"
        );
        out.clear();
        flags.write_bytes(&mut out, &dead).unwrap();
        assert!(
            out.contains("DEADBEEF") || out.contains("\\xDE\\xAD\\xBE\\xEF"),
            "{out}"
        );
        assert!(!out.contains("deadbeef"), "hex must be uppercase: {out}");
    }
}

#[test]
fn write_bytes_slash_x_blob_empty_is_not_a_slash_escape() {
    // Empty `'\x'::BLOB` would be a one-byte escape, not an empty blob.
    let flags = config::DialectConfig {
        bytes_format: config::BytesFormat::SlashXBlob,
        ..Default::default()
    };
    let mut out = String::new();
    flags.write_bytes(&mut out, &[]).unwrap();
    assert_eq!(out, "''::BLOB");
    out.clear();
    flags.write_bytes(&mut out, &[0x00]).unwrap();
    assert_eq!(out, "'\\x00'::BLOB");
}

#[test]
fn write_bytes_crosses_hex_and_slash_chunk_boundaries() {
    let sixty_five = vec![0xABu8; 65];
    let thirty_three = vec![0xFFu8; 33];

    let mut hex = String::new();
    config::DialectConfig {
        bytes_format: config::BytesFormat::PrefixBigX,
        ..Default::default()
    }
    .write_bytes(&mut hex, &sixty_five)
    .unwrap();

    assert_eq!(hex.len(), 2 + 65 * 2 + 1);
    assert!(hex.starts_with("X'") && hex.ends_with('\''));
    assert_eq!(&hex[2..hex.len() - 1], &"AB".repeat(65));

    let mut slash = String::new();
    config::DialectConfig {
        bytes_format: config::BytesFormat::PrefixBigBSlashX,
        ..Default::default()
    }
    .write_bytes(&mut slash, &thirty_three)
    .unwrap();

    assert_eq!(slash, format!("B'{}'", "\\xFF".repeat(33)));

    let mut blob = String::new();
    config::DialectConfig {
        bytes_format: config::BytesFormat::SlashXBlob,
        ..Default::default()
    }
    .write_bytes(&mut blob, &thirty_three)
    .unwrap();

    assert_eq!(blob, format!("'{}'::BLOB", "\\xFF".repeat(33)));
}

#[test]
fn write_bytes_zero_prefix_empty_is_not_a_closed_literal() {
    // `0x` with no digits is rejected by most engines. Locked so a caller
    // that needs an empty blob picks another spelling.
    let mut out = String::new();
    config::DialectConfig {
        bytes_format: config::BytesFormat::PrefixZeroX,
        ..Default::default()
    }
    .write_bytes(&mut out, &[])
    .unwrap();

    assert_eq!(out, "0x");
}

#[test]
fn write_bytes_function_wrappers_cannot_be_closed_by_payload() {
    // The hex alphabet is generated, not copied, so a byte cannot inject a
    // quote into HEXTORAW / UNHEX / TO_BINARY / TO_VARBYTE.
    let nasty = [0x27, 0x5C, 0x00, 0x22, 0x3B];
    for format in [
        config::BytesFormat::HexToRaw,
        config::BytesFormat::Unhex,
        config::BytesFormat::ToVarbyte,
        config::BytesFormat::ToBinary,
    ] {
        let mut out = String::new();
        config::DialectConfig {
            bytes_format: format,
            ..Default::default()
        }
        .write_bytes(&mut out, &nasty)
        .unwrap();

        assert!(out.contains("275C00223B"), "{format:?} -> {out}");
        let hex = out
            .split('\'')
            .nth(1)
            .unwrap_or_else(|| panic!("missing hex literal in {out}"));

        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "{format:?} leaked non-hex into {out}"
        );
        assert_eq!(hex, "275C00223B");
    }
}

#[test]
fn writer_errors_surface_as_format_error() {
    let mut flags = string_flags(config::StringFormat::PrefixBigN, config::StringEscape::Quote);
    let mut writer = FailAfter { remaining: 0 };
    assert_eq!(
        flags.write_string_literal(&mut writer, "x").unwrap_err(),
        errors::FormatError::Write
    );

    let mut writer = FailAfter { remaining: 0 };

    flags.bit_format = config::BitFormat::PrefixBigB;
    assert!(flags.write_bits(&mut writer, "1").is_err());
}

fn column(name: &str) -> crate::expr::enum_::ExprInner {
    crate::expr::enum_::ExprInner::Ident(crate::expr::enum_::Ident::from_str(name).unwrap())
}

fn binary(op: crate::expr::enum_::BinaryOp) -> crate::expr::enum_::Binary {
    crate::expr::enum_::Binary {
        left: Box::new(column("a")),
        right: Box::new(column("b")),
        op,
    }
}

fn render(
    py: pyo3::Python<'_>,
    flags: config::DialectConfig,
    expr: &impl crate::sql::WriteSql,
) -> Result<String, crate::sql::PrepareError> {
    let dialect = PyDialect(flags);
    let mut buf = crate::sql::BufferedSql::new(py);
    expr.write_sql(&dialect, &mut buf)?;
    Ok(buf.into_string())
}

fn error_text(err: crate::sql::PrepareError) -> String {
    match err {
        crate::sql::PrepareError::Python(err) => err.to_string(),
        other => other.to_string(),
    }
}

/// Infix AND and OR, no NOT, no shifts, XOR as `#`.
///
/// The old `Bitwise` enum could not name this mix. Each operator is its own bit.
#[test]
fn bitwise_and_or_with_hash_xor_rejects_not_and_shifts() {
    use crate::expr::enum_::BinaryOp;
    use crate::expr::enum_::Unary;
    use crate::expr::enum_::UnaryOp;

    pyo3::Python::attach(|py| {
        let flags = config::DialectConfig {
            bitwise_operations: config::BitwiseOperations::AND | config::BitwiseOperations::OR,
            bitwise_call_syntax: config::BitwiseCallSyntax::Infix,
            bitwise_xor_spelling: config::BitwiseXorSpelling::Hash,
            ..Default::default()
        };

        assert_eq!(render(py, flags, &binary(BinaryOp::BitAnd)).unwrap(), "\"a\" & \"b\"");
        assert_eq!(render(py, flags, &binary(BinaryOp::BitOr)).unwrap(), "\"a\" | \"b\"");
        assert_eq!(render(py, flags, &binary(BinaryOp::BitXor)).unwrap(), "\"a\" # \"b\"");

        let not = Unary {
            op: UnaryOp::BitNot,
            expr: Box::new(column("a")),
        };
        assert!(error_text(render(py, flags, &not).unwrap_err()).contains("bitwise NOT"));
        assert!(error_text(render(py, flags, &binary(BinaryOp::LShift)).unwrap_err()).contains("bitwise shift"));
        assert!(error_text(render(py, flags, &binary(BinaryOp::RShift)).unwrap_err()).contains("bitwise shift"));
    });
}

/// `ROW` plus arity 1 rejects `ROW()`. Parentheses with an empty arity mask still need two elements.
#[test]
fn tuple_arity_is_independent_of_syntax() {
    use crate::expr::enum_::Tuple;

    pyo3::Python::attach(|py| {
        let row = config::DialectConfig {
            tuple_syntax: config::TupleSyntax::Row,
            tuple_arity: config::TupleArity::ONE,
            ..Default::default()
        };

        let one = Tuple {
            items: vec![column("a")],
        };
        let two = Tuple {
            items: vec![column("a"), column("b")],
        };
        assert_eq!(render(py, row, &one).unwrap(), "ROW(\"a\")");
        assert_eq!(render(py, row, &two).unwrap(), "ROW(\"a\", \"b\")");
        assert!(error_text(render(py, row, &Tuple { items: vec![] }).unwrap_err()).contains("empty tuple"));

        let parens = config::DialectConfig {
            tuple_syntax: config::TupleSyntax::Parentheses,
            ..Default::default()
        };

        assert!(error_text(render(py, parens, &one).unwrap_err()).contains("at least two"));
        assert_eq!(render(py, parens, &two).unwrap(), "(\"a\", \"b\")");
    });
}

/// `SCALAR`, `ANY`, and empty are three different `ESCAPE` policies.
#[test]
fn like_escape_flags_combine() {
    use crate::expr::enum_::Like;
    use crate::expr::enum_::LikePattern;

    fn like(any: bool, escape: Option<char>) -> Like {
        Like {
            case_sensitive: true,
            negated: false,
            expr: Box::new(column("a")),
            pattern: if any {
                LikePattern::Any(vec![column("p")])
            } else {
                LikePattern::One(Box::new(column("p")))
            },
            escape,
        }
    }

    pyo3::Python::attach(|py| {
        let scalar = config::DialectConfig {
            like_escape: config::LikeEscape::SCALAR,
            like_any: config::LikeAny::PatternList,
            ..Default::default()
        };

        let both = config::DialectConfig {
            like_escape: config::LikeEscape::SCALAR | config::LikeEscape::ANY,
            like_any: config::LikeAny::PatternList,
            ..Default::default()
        };

        let none = config::DialectConfig {
            like_any: config::LikeAny::PatternList,
            ..Default::default()
        };

        assert_eq!(
            render(py, scalar, &like(false, Some('!'))).unwrap(),
            "\"a\" LIKE \"p\" ESCAPE '!'"
        );
        assert!(error_text(render(py, scalar, &like(true, Some('!'))).unwrap_err()).contains("LIKE ANY ESCAPE"));
        assert!(render(py, both, &like(true, Some('!'))).unwrap().contains("ESCAPE"));
        assert!(error_text(render(py, none, &like(false, Some('!'))).unwrap_err()).contains("LIKE ESCAPE"));
    });
}
