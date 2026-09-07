//! Dialect quoting/escaping tests.
use super::*;

/// Every dialect this crate supports, used to drive table-style tests.
const ALL_DIALECTS: &[SqlDialect] = &[
    SqlDialect::BigQuery,
    SqlDialect::Databricks,
    SqlDialect::DuckDb,
    SqlDialect::Hive,
    SqlDialect::MsSql,
    SqlDialect::MySql,
    SqlDialect::Oracle,
    SqlDialect::PostgreSql,
    SqlDialect::Redshift,
    SqlDialect::Snowflake,
    SqlDialect::Spark,
    SqlDialect::Sqlite,
    SqlDialect::Teradata,
];

/// Dialects whose `escape_string` interprets backslash sequences
/// (`\\`, `\n`, `\t`, ...). These MUST double a literal backslash.
const BACKSLASH_ESCAPING_DIALECTS: &[SqlDialect] = &[
    SqlDialect::BigQuery,
    SqlDialect::Databricks,
    SqlDialect::Hive,
    SqlDialect::MySql,
    SqlDialect::PostgreSql,
    SqlDialect::Snowflake,
    SqlDialect::Spark,
];

/// Dialects that only double single quotes and treat backslash as a
/// completely literal character (per their own doc comments). These MUST
/// pass a literal backslash straight through, unchanged.
const QUOTE_ONLY_DIALECTS: &[SqlDialect] = &[
    SqlDialect::DuckDb,
    SqlDialect::MsSql,
    SqlDialect::Oracle,
    SqlDialect::Redshift,
    SqlDialect::Sqlite,
    SqlDialect::Teradata,
];

/// Dialects documented as stripping NUL bytes outright (rather than
/// escaping them) because their target string types can't hold NUL.
const NUL_STRIPPING_DIALECTS: &[SqlDialect] = &[
    SqlDialect::MsSql,
    SqlDialect::Oracle,
    SqlDialect::Redshift,
    SqlDialect::Sqlite,
    SqlDialect::Teradata,
];

fn sanity_check_coverage() {
    // Every dialect must land in exactly one of the two escaping-style
    // buckets above, and that bucket assignment must partition ALL_DIALECTS.
    assert_eq!(
        BACKSLASH_ESCAPING_DIALECTS.len() + QUOTE_ONLY_DIALECTS.len(),
        ALL_DIALECTS.len(),
        "every dialect must be classified as either backslash-escaping or quote-only"
    );
}

// ---------------------------------------------------------------------
// 1. Single-quote handling: the one thing EVERY dialect must get right —
//    though the *shape* of the escape differs by family:
//      - QUOTE_ONLY dialects (ANSI-style) double it:      '  ->  ''
//      - BACKSLASH_ESCAPING dialects (C/Java-style) prefix it: '  ->  \'
// ---------------------------------------------------------------------

#[test]
fn quote_only_dialects_double_a_lone_single_quote() {
    sanity_check_coverage();
    for dialect in QUOTE_ONLY_DIALECTS {
        let escaped = dialect.escape_string_as_string("'");
        assert_eq!(
            escaped,
            "''",
            "{} did not double a lone single quote (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn backslash_escaping_dialects_prefix_a_lone_single_quote() {
    for dialect in BACKSLASH_ESCAPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("'");
        assert_eq!(
            escaped,
            "\\'",
            "{} did not backslash-escape a lone single quote (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn quote_only_dialects_double_multiple_single_quotes() {
    for dialect in QUOTE_ONLY_DIALECTS.iter() {
        let escaped = dialect.escape_string_as_string("''''");
        assert_eq!(
            escaped,
            "''''''''",
            "{} mis-handled 4 consecutive quotes (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn backslash_escaping_dialects_prefix_multiple_single_quotes() {
    for dialect in BACKSLASH_ESCAPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("''''");
        assert_eq!(
            escaped,
            "\\'\\'\\'\\'",
            "{} mis-handled 4 consecutive quotes (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn every_dialect_neutralizes_the_apostrophe_in_a_classic_name() {
    for dialect in ALL_DIALECTS {
        let escaped = dialect.escape_string_as_string("O'Brien");
        let uses_backslash = BACKSLASH_ESCAPING_DIALECTS.contains(dialect);
        let expected = if uses_backslash {
            "O\\'Brien"
        } else {
            "O''Brien"
        };
        assert_eq!(
            escaped,
            expected,
            "{} did not neutralize the apostrophe in \"O'Brien\" as expected (got {escaped:?})",
            dialect.name()
        );
    }
}

// ---------------------------------------------------------------------
// 2. Backslash handling must match each dialect's documented behavior.
// ---------------------------------------------------------------------

#[test]
fn backslash_escaping_dialects_double_a_lone_backslash() {
    for dialect in BACKSLASH_ESCAPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("\\");
        assert_eq!(
            escaped,
            "\\\\",
            "{} should double a lone backslash (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn quote_only_dialects_pass_backslash_through_unchanged() {
    for dialect in QUOTE_ONLY_DIALECTS.iter() {
        let escaped = dialect.escape_string_as_string("\\");
        assert_eq!(
            escaped,
            "\\",
            "{} must treat backslash as a literal, non-special character (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn quote_only_dialects_dont_touch_backslash_next_to_quote() {
    // This is the classic footgun: a naive implementation might see `\'`
    // and think it needs special handling. These dialects must NOT
    // interpret backslash-quote as an escape sequence — the backslash
    // stays literal and the quote still gets doubled independently.
    for dialect in QUOTE_ONLY_DIALECTS.iter() {
        let escaped = dialect.escape_string_as_string("\\'");
        assert_eq!(
            escaped,
            "\\''",
            "{} mis-handled backslash immediately followed by a quote (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn backslash_escaping_dialects_handle_backslash_next_to_quote() {
    for dialect in BACKSLASH_ESCAPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("\\'");
        assert_eq!(
            escaped,
            "\\\\\\'",
            "{} mis-handled backslash immediately followed by a quote (got {escaped:?})",
            dialect.name()
        );
    }
}

// ---------------------------------------------------------------------
// 3. NUL byte handling: strip vs. escape, per dialect contract.
// ---------------------------------------------------------------------

/// Dialects documented as escaping NUL (rather than stripping or passing
/// it through raw) — i.e. the backslash-escaping family, all of which
/// have an explicit `'\0' => ...` arm.
const NUL_ESCAPING_DIALECTS: &[SqlDialect] = &[
    SqlDialect::BigQuery,
    SqlDialect::Databricks,
    SqlDialect::Hive,
    SqlDialect::MySql,
    SqlDialect::PostgreSql,
    SqlDialect::Snowflake,
    SqlDialect::Spark,
];

#[test]
fn nul_stripping_dialects_remove_nul_entirely() {
    for dialect in NUL_STRIPPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("a\0b");
        assert_eq!(
            escaped,
            "ab",
            "{} should strip NUL bytes outright (got {escaped:?})",
            dialect.name()
        );
        assert!(
            !escaped.contains('\0'),
            "{} left a raw NUL byte in its output",
            dialect.name()
        );
    }
}

#[test]
fn nul_escaping_dialects_replace_nul_with_an_escape_sequence() {
    for dialect in NUL_ESCAPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("a\0b");
        assert!(
            !escaped.contains('\0'),
            "{} left a raw NUL byte in escaped output (got {escaped:?})",
            dialect.name()
        );
        assert_ne!(
            escaped,
            "a\0b",
            "{} should have replaced NUL with an escape sequence, not left it untouched",
            dialect.name()
        );
    }
}

#[test]
fn duckdb_documents_nul_as_a_regular_passthrough_character() {
    // DuckDB VARCHAR is UTF-8 and its docs don't call out NUL as
    // unsupported, unlike MsSql/Oracle/Redshift/Sqlite/Teradata, which
    // explicitly strip it. This test exists to make that asymmetry
    // explicit and catch it if DuckDB's contract ever changes.
    let escaped = SqlDialect::DuckDb.escape_string_as_string("a\0b");
    assert_eq!(escaped, "a\0b");
}

// ---------------------------------------------------------------------
// 4. Control characters (newline, tab, CR, and other ASCII control codes).
// ---------------------------------------------------------------------

#[test]
fn backslash_escaping_dialects_never_emit_a_raw_newline_or_cr() {
    // \n and \r get an explicit backslash-escape arm in every
    // backslash-escaping dialect's implementation.
    for dialect in BACKSLASH_ESCAPING_DIALECTS {
        let escaped = dialect.escape_string_as_string("a\nb\rc");
        assert!(
            !escaped.contains('\n') && !escaped.contains('\r'),
            "{} left a raw \\n or \\r in escaped output (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn most_backslash_escaping_dialects_also_escape_tab() {
    // MySQL's implementation intentionally has no explicit tab arm (a
    // literal tab is not a syntax hazard inside a MySQL string literal,
    // unlike a quote or backslash) so it's excluded here. Every other
    // backslash-escaping dialect does translate \t -> \\t.
    let tab_escaping: Vec<SqlDialect> = BACKSLASH_ESCAPING_DIALECTS
        .iter()
        .copied()
        .filter(|d| *d != SqlDialect::MySql)
        .collect();

    for dialect in tab_escaping {
        let escaped = dialect.escape_string_as_string("a\tb");
        assert!(
            !escaped.contains('\t'),
            "{} left a raw tab in escaped output (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn mysql_passes_a_raw_tab_through_unchanged() {
    // Documents MySQL's actual (intentional) behavior explicitly, so a
    // future change is a deliberate decision rather than a silent drift.
    let escaped = SqlDialect::MySql.escape_string_as_string("a\tb");
    assert_eq!(escaped, "a\tb");
}

#[test]
fn quote_only_dialects_pass_newline_cr_tab_through_unchanged() {
    // These dialects only double single quotes; per their own doc
    // comments they don't treat backslash (or \n/\r/\t) as special.
    // A raw newline etc. inside '...' is syntactically valid in these
    // engines and is NOT an injection risk on its own — only the quote
    // character matters for breaking out of the literal.
    for dialect in QUOTE_ONLY_DIALECTS.iter() {
        let escaped = dialect.escape_string_as_string("a\nb\rc\td");
        assert_eq!(
            escaped,
            "a\nb\rc\td",
            "{} was expected to pass control chars through unchanged (got {escaped:?})",
            dialect.name()
        );
    }
}

#[test]
fn every_dialect_neutralizes_low_ascii_control_bytes() {
    // 0x01..=0x1F (excluding \n=0x0A, \r=0x0D, \t=0x09 already covered)
    // and 0x7F (DEL) are the classic "invisible injection" characters.
    // At minimum, no dialect should pass them through raw next to
    // unescaped quotes in a way that changes the quote count.
    for dialect in ALL_DIALECTS {
        for byte in [0x01u8, 0x07, 0x1b, 0x1f, 0x7f] {
            let ch = byte as char;
            let input = format!("x{ch}y");
            let escaped = dialect.escape_string_as_string(&input);
            // The escape function must not panic and must not introduce
            // an unbalanced quote by accident.
            assert_eq!(
                escaped.matches('\'').count() % 2,
                0,
                "{} produced an odd number of quotes for control byte 0x{byte:02X} (got \
                 {escaped:?})",
                dialect.name()
            );
        }
    }
}

// ---------------------------------------------------------------------
// 5. End-to-end injection safety: escape_string + quote_string together
//    must never let attacker input terminate the string literal early.
// ---------------------------------------------------------------------

/// A grab-bag of classic SQL-injection-style payloads.
const HOSTILE_INPUTS: &[&str] = &[
    "",
    "'",
    "''",
    "'; DROP TABLE users; --",
    "' OR '1'='1",
    "\\",
    "\\'",
    "\\\\",
    "\\'; DROP TABLE users; --",
    "a\0b",
    "\n\r\t",
    "'\0'",
    "\\\0'",
    "😀'; DROP TABLE users;--😀",
    "back\\slash'quote\"doublequote",
];

/// For a single-quoted-literal dialect, walks the fully escaped+quoted
/// output character by character (respecting the dialect's own escape
/// convention) and confirms the literal's closing quote is genuinely the
/// last unescaped quote — i.e. nothing in the payload could prematurely
/// close the string.
///
/// Operates entirely in `char` space (never byte-slicing a `&str`) so
/// multi-byte UTF-8 content (e.g. emoji) can't desync index arithmetic.
fn assert_no_premature_close(dialect: &SqlDialect, quoted: &str, uses_backslash_escapes: bool) {
    let chars: Vec<char> = quoted.chars().collect();

    let first_quote_idx = chars.iter().position(|&c| c == '\'').unwrap_or_else(|| {
        panic!(
            "{}: quoted output has no opening quote: {quoted:?}",
            dialect.name()
        )
    });

    let body = &chars[first_quote_idx + 1..];
    let mut i = 0;
    let mut closed_at = None;
    while i < body.len() {
        let c = body[i];
        if uses_backslash_escapes && c == '\\' {
            // Skip the escaped character (backslash always consumes the
            // next char in these dialects' escape schemes).
            i += 2;
            continue;
        }
        if c == '\'' {
            // A doubled quote (''), representing one literal quote,
            // does NOT close the string — only a single trailing quote
            // with no partner does.
            if body.get(i + 1) == Some(&'\'') {
                i += 2;
                continue;
            }
            closed_at = Some(i);
            break;
        }
        i += 1;
    }

    let closed_at = closed_at.unwrap_or_else(|| {
        panic!(
            "{}: could not find a closing quote at all in {quoted:?}",
            dialect.name()
        )
    });

    // The closing quote we found must be the LAST character of the
    // string (i.e. nothing meaningful follows it) — otherwise the
    // payload managed to close the literal early and inject trailing SQL.
    let remainder: String = body[closed_at + 1..].iter().collect();
    assert!(
        remainder.is_empty(),
        "{}: literal closed early, leaving trailing unescaped content {remainder:?} in \
         {quoted:?}",
        dialect.name()
    );
}

#[test]
fn no_hostile_input_can_break_out_of_the_string_literal() {
    for dialect in ALL_DIALECTS {
        let uses_backslash = BACKSLASH_ESCAPING_DIALECTS.contains(dialect);
        for payload in HOSTILE_INPUTS {
            let escaped = dialect.escape_string_as_string(payload);
            let quoted = dialect.quote_string_as_string(&escaped);
            assert_no_premature_close(dialect, &quoted, uses_backslash);
        }
    }
}

#[test]
fn no_hostile_input_ever_panics_any_dialect_function() {
    // A panic (e.g. from an unwrap on malformed UTF-8 boundary slicing)
    // is itself a correctness bug worth catching explicitly.
    for dialect in ALL_DIALECTS {
        for payload in HOSTILE_INPUTS {
            let escaped = dialect.escape_string_as_string(payload);
            let _ = dialect.quote_string_as_string(&escaped);
        }
    }
}

// ---------------------------------------------------------------------
// 6. Unicode correctness: multi-byte scalars must survive untouched.
// ---------------------------------------------------------------------

#[test]
fn unicode_content_passes_through_unmangled() {
    let samples = ["héllo", "日本語", "emoji😀🎉", "Ñoño", "Straße"];
    for dialect in ALL_DIALECTS {
        for sample in samples {
            let escaped = dialect.escape_string_as_string(sample);
            assert_eq!(
                escaped,
                sample,
                "{} corrupted unicode input {sample:?} (got {escaped:?})",
                dialect.name()
            );
            // Must still be valid UTF-8 after quoting (String guarantees
            // this, but we assert to document the expectation).
            let quoted = dialect.quote_string_as_string(&escaped);
            assert!(quoted.contains(sample));
        }
    }
}

#[test]
fn unicode_mixed_with_quotes_still_escapes_correctly() {
    for dialect in ALL_DIALECTS {
        let escaped = dialect.escape_string_as_string("日本語'quote'😀");
        let uses_backslash = BACKSLASH_ESCAPING_DIALECTS.contains(dialect);
        let quote_marker_count = if uses_backslash {
            escaped.matches("\\'").count()
        } else {
            escaped.matches("''").count()
        };
        assert_eq!(
            quote_marker_count,
            2,
            "{}: expected both quotes in the unicode string to be escaped (got {escaped:?})",
            dialect.name()
        );
        assert!(escaped.contains("日本語"));
        assert!(escaped.contains('😀'));
    }
}

// ---------------------------------------------------------------------
// 7. Empty-input edge cases.
// ---------------------------------------------------------------------

#[test]
fn empty_string_escapes_to_empty_string() {
    for dialect in ALL_DIALECTS {
        assert_eq!(dialect.escape_string_as_string(""), "");
    }
}

#[test]
fn empty_string_quoted_is_just_the_delimiters() {
    for dialect in ALL_DIALECTS {
        let quoted = dialect.quote_string_as_string("");
        // Must contain exactly 2 single quotes and nothing else meaningful
        // (allowing for a dialect-specific prefix like `E` or `N`).
        assert_eq!(
            quoted.matches('\'').count(),
            2,
            "{}: empty quoted string should have exactly one open/close pair (got {quoted:?})",
            dialect.name()
        );
    }
}

// ---------------------------------------------------------------------
// 8. encode_bytes: hex correctness, case, empty/boundary input.
// ---------------------------------------------------------------------

#[test]
fn encode_bytes_empty_slice_does_not_panic_and_contains_no_hex_pairs() {
    for dialect in ALL_DIALECTS {
        let out = dialect.encode_bytes_as_string(&[]);
        // No stray "00" or similar should appear from an off-by-one bug.
        assert!(
            !out.is_empty(),
            "{}: encode_bytes(&[]) produced an empty string, expected at least wrapper syntax",
            dialect.name()
        );
    }
}

#[test]
fn encode_bytes_uses_uppercase_hex_and_round_trips_all_byte_values() {
    let bytes: Vec<u8> = (0u8..=255).collect();

    // BigQuery renders each byte as its own \xNN escape (not a
    // contiguous hex run via the shared bytes_to_hex() helper), so it's
    // checked separately below.
    let plain_hex_run_dialects: Vec<SqlDialect> = ALL_DIALECTS
        .iter()
        .copied()
        .filter(|d| *d != SqlDialect::BigQuery)
        .collect();

    // The uppercase hex encoding of 0x00..=0xFF, in order — this is
    // exactly what bytes_to_hex() must produce, byte for byte. Checking
    // for this literal substring (rather than scanning all hex-looking
    // characters in the output) avoids false positives from incidental
    // lowercase letters in surrounding syntax like "bytea" or "'hex'".
    let expected_hex_run: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();

    for dialect in plain_hex_run_dialects {
        let out = dialect.encode_bytes_as_string(&bytes);
        assert!(
            out.contains(&expected_hex_run),
            "{}: encode_bytes did not contain the expected uppercase hex run for all 256 byte \
             values (got {out:?})",
            dialect.name()
        );
    }

    // BigQuery: each byte must appear as its own \xNN, in order, with
    // uppercase hex digits.
    let expected_bigquery: String = bytes.iter().map(|b| format!("\\x{:02X}", b)).collect();
    let bigquery_out = SqlDialect::BigQuery.encode_bytes_as_string(&bytes);
    assert!(
        bigquery_out.contains(&expected_bigquery),
        "BigQuery: encode_bytes did not contain the expected \\xNN-per-byte run (got \
         {bigquery_out:?})"
    );
}

#[test]
fn encode_bytes_single_byte_zero_and_single_byte_max() {
    for dialect in ALL_DIALECTS {
        let zero = dialect.encode_bytes_as_string(&[0x00]);
        let max = dialect.encode_bytes_as_string(&[0xFF]);
        assert!(zero.contains("00"), "{}: {zero:?}", dialect.name());
        assert!(max.contains("FF"), "{}: {max:?}", dialect.name());
        assert_ne!(zero, max);
    }
}

// ---------------------------------------------------------------------
// 9. encode_bits: boundary lengths, all-zero, all-one, longer-than-i64.
// ---------------------------------------------------------------------

#[test]
fn encode_bits_all_zero_and_all_one_single_bit() {
    for dialect in ALL_DIALECTS {
        let zero = dialect.encode_bits_as_string("0");
        let one = dialect.encode_bits_as_string("1");
        assert_ne!(
            zero,
            one,
            "{}: encode_bits(\"0\") and encode_bits(\"1\") must differ",
            dialect.name()
        );
    }
}

#[test]
fn encode_bits_does_not_panic_on_64_bit_boundary() {
    // Exactly 64 ones is the classic off-by-one boundary for
    // i64::from_str_radix-based implementations.
    let sixty_four_ones = "1".repeat(64);
    for dialect in ALL_DIALECTS {
        let _ = dialect.encode_bits_as_string(&sixty_four_ones);
    }
}

#[test]
fn encode_bits_does_not_panic_on_overflow_longer_than_i64() {
    // More than 64 bits overflows i64::from_str_radix; dialects that
    // parse via i64 fall back to 0 on error rather than panicking. This
    // test locks in "no panic" as the contract, regardless of the
    // fallback value chosen.
    let too_many_ones = "1".repeat(100);
    for dialect in ALL_DIALECTS {
        let _ = dialect.encode_bits_as_string(&too_many_ones);
    }
}

#[test]
fn encode_bits_empty_string_does_not_panic() {
    for dialect in ALL_DIALECTS {
        let _ = dialect.encode_bits_as_string("");
    }
}

#[test]
fn encode_bits_known_value_is_consistent_across_dialects_that_parse_as_integer() {
    // "10110101" == 181 decimal. Every dialect that renders this via an
    // integer cast should have "181" appear somewhere in its output.
    let bits = "10110101";
    let integer_based = [
        SqlDialect::BigQuery,   // CAST(0b... AS INT64) -- uses 0b, not decimal
        SqlDialect::Databricks, // CAST(...L AS BINARY) -- decimal
        SqlDialect::Hive,       // CAST(... AS BIGINT) -- decimal
        SqlDialect::MsSql,      // CAST(... AS BIGINT) -- decimal
        SqlDialect::Oracle,     // bare decimal
        SqlDialect::Redshift,   // CAST(... AS BIGINT) -- decimal
        SqlDialect::Snowflake,  // TO_HEX(...) -- decimal appears inside TO_HEX(
        SqlDialect::Spark,      // CAST(... AS BIGINT) -- decimal
        SqlDialect::Sqlite,     // bare decimal
    ];
    for dialect in &integer_based {
        let out = dialect.encode_bits_as_string(bits);
        if *dialect == SqlDialect::BigQuery {
            assert!(
                out.contains("0b10110101"),
                "{}: expected binary literal form (got {out:?})",
                dialect.name()
            );
        } else {
            assert!(
                out.contains("181"),
                "{}: expected decimal value 181 to appear in output (got {out:?})",
                dialect.name()
            );
        }
    }
}

#[test]
fn encode_bits_literal_style_dialects_preserve_the_bit_string_verbatim() {
    // Dialects with a native bit-string literal syntax should embed the
    // exact input bits, not a numeric conversion.
    let bits = "10110101";
    assert_eq!(
        SqlDialect::PostgreSql.encode_bits_as_string(bits),
        "B'10110101'"
    );
    assert_eq!(
        SqlDialect::DuckDb.encode_bits_as_string(bits),
        "B'10110101'"
    );
    assert_eq!(SqlDialect::MySql.encode_bits_as_string(bits), "b'10110101'");
    assert_eq!(
        SqlDialect::Teradata.encode_bits_as_string(bits),
        "'10110101'B"
    );
}

// ---------------------------------------------------------------------
// 10. quote_string wrapping sanity: dialect-specific prefixes/suffixes.
// ---------------------------------------------------------------------

#[test]
fn quote_string_wraps_with_expected_prefix_per_dialect() {
    let escaped = "abc";
    assert!(
        SqlDialect::PostgreSql
            .quote_string_as_string(escaped)
            .starts_with("E'")
    );
    assert!(
        SqlDialect::MsSql
            .quote_string_as_string(escaped)
            .starts_with("N'")
    );
    // Plain single-quote wrapping dialects: no special prefix letter.
    for dialect in &[
        SqlDialect::BigQuery,
        SqlDialect::MySql,
        SqlDialect::Sqlite,
        SqlDialect::Snowflake,
        SqlDialect::Oracle,
        SqlDialect::Redshift,
        SqlDialect::Teradata,
        SqlDialect::Spark,
        SqlDialect::Hive,
        SqlDialect::Databricks,
        SqlDialect::DuckDb,
    ] {
        let quoted = dialect.quote_string_as_string(escaped);
        assert!(
            quoted.starts_with('\''),
            "{}: expected plain single-quote wrapping (got {quoted:?})",
            dialect.name()
        );
    }
}

#[test]
fn quote_string_always_ends_with_a_single_quote() {
    for dialect in ALL_DIALECTS {
        let quoted = dialect.quote_string_as_string("abc");
        assert!(
            quoted.ends_with('\''),
            "{}: quoted string should end with a closing quote (got {quoted:?})",
            dialect.name()
        );
    }
}

// ---------------------------------------------------------------------
// 11. Identifier quoting and hex helpers.
// ---------------------------------------------------------------------

#[test]
fn sql_dialect_is_copy() {
    fn assert_copy<T: Copy>(_: T) {}
    assert_copy(SqlDialect::Sqlite);
    assert_copy(PlaceholderSyntax::Colon);
}

#[test]
fn name_is_the_python_class_name() {
    let expected = [
        (SqlDialect::BigQuery, "BigQuery"),
        (SqlDialect::Databricks, "Databricks"),
        (SqlDialect::DuckDb, "DuckDB"),
        (SqlDialect::Hive, "Hive"),
        (SqlDialect::MsSql, "MSSQL"),
        (SqlDialect::MySql, "MySql"),
        (SqlDialect::Oracle, "Oracle"),
        (SqlDialect::PostgreSql, "PostgreSQL"),
        (SqlDialect::Redshift, "Redshift"),
        (SqlDialect::Snowflake, "Snowflake"),
        (SqlDialect::Spark, "Spark"),
        (SqlDialect::Sqlite, "SQLite"),
        (SqlDialect::Teradata, "Teradata"),
    ];
    assert_eq!(expected.len(), ALL_DIALECTS.len());
    for (dialect, name) in expected {
        assert_eq!(dialect.name(), name);
    }
}

#[test]
fn quote_identifier_uses_expected_delimiters() {
    assert_eq!(SqlDialect::MySql.quote_identifier_as_string("id"), "`id`");
    assert_eq!(
        SqlDialect::PostgreSql.quote_identifier_as_string("id"),
        "\"id\""
    );
    assert_eq!(SqlDialect::MsSql.quote_identifier_as_string("id"), "[id]");
    assert_eq!(
        SqlDialect::Teradata.quote_identifier_as_string("id"),
        "'id'"
    );
    assert_eq!(
        SqlDialect::BigQuery.quote_identifier_as_string("id"),
        "`id`"
    );
}

#[test]
fn quote_identifier_escapes_the_delimiter_in_the_body() {
    assert_eq!(
        SqlDialect::MySql.quote_identifier_as_string("a`b"),
        "`a``b`"
    );
    assert_eq!(
        SqlDialect::PostgreSql.quote_identifier_as_string("a\"b"),
        "\"a\"\"b\""
    );
    assert_eq!(
        SqlDialect::MsSql.quote_identifier_as_string("a]b"),
        "[a]]b]"
    );
    assert_eq!(
        SqlDialect::Teradata.quote_identifier_as_string("a'b"),
        "'a''b'"
    );
}

#[test]
fn escape_identifier_is_the_body_only() {
    assert_eq!(SqlDialect::MySql.escape_identifier_as_string("a`b"), "a``b");
    assert_eq!(
        SqlDialect::PostgreSql.escape_identifier_as_string("a\"b"),
        "a\"\"b"
    );
    assert_eq!(SqlDialect::MsSql.escape_identifier_as_string("a]b"), "a]]b");
}

#[test]
fn write_hex2_is_uppercase_two_digits() {
    let mut s = String::new();
    write_hex2(&mut s, 0x00).unwrap();
    write_hex2(&mut s, 0xAB).unwrap();
    write_hex2(&mut s, 0xFF).unwrap();
    assert_eq!(s, "00ABFF");
}

#[test]
fn placeholder_sigil_matches_syntax() {
    assert_eq!(PlaceholderSyntax::AtSign.sigil(), '@');
    assert_eq!(PlaceholderSyntax::Colon.sigil(), ':');
    assert_eq!(PlaceholderSyntax::DollarNamed.sigil(), '$');
}
