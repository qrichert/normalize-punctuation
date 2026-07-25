//! Punctuation normalization.

use std::sync::LazyLock;

use aho_corasick::{AhoCorasick, MatchKind};

/// Ordered list of punctuation patterns and their ASCII replacements.
///
/// Order is significant: at a shared start position the earlier entry
/// wins (e.g. `"\u{2009};"` before `"\u{2009}"` and `"« "` before
/// `"«"`), so context-specific forms take precedence over bare ones.
const REPLACEMENTS: [(&str, &str); 23] = [
    ("‘", "'"),
    ("’", "'"),
    ("“", "\""),
    ("”", "\""),
    ("ˋ", "`"), // Grave accent.
    ("‚", "'"),
    ("„", "\""),
    ("…", "..."),
    ("\u{a0}", "&nbsp;"),      // NBSP.
    ("\u{202f}", "&#8239;"),   // NNBSP.
    ("\u{2009};", "&#8239;;"), // Thin space before French punctuation.
    ("\u{2009}?", "&#8239;?"),
    ("\u{2009}!", "&#8239;!"),
    ("\u{2009}:", "&nbsp;:"),
    ("\u{2009}", "&thinsp;"), // Thin space.
    ("« ", "\""),
    ("«", "\""),
    (" »", "\""),
    ("»", "\""),
    ("‐", "-"),
    ("﹘", "-"),
    ("−", "-"),
    ("–", "-"), // en-dash.
];

/// Automaton over the `REPLACEMENTS` patterns, built once per process.
///
/// `MatchKind::LeftmostFirst` makes an earlier-listed pattern win at a
/// shared start position, so context-specific patterns take precedence
/// over their shorter forms (see tests).
static AUTOMATON: LazyLock<AhoCorasick> = LazyLock::new(|| {
    AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(REPLACEMENTS.iter().map(|(pattern, _)| *pattern))
        .expect("replacement patterns should build a valid automaton")
});

/// Normalize the punctuation in `input`.
///
/// Returns `Some(normalized)` if any replacement applied, or `None` if
/// `input` is already normalized. Pure and I/O-free, so it is
/// unit-testable and benchmarkable without touching the filesystem.
///
/// Runs in a single pass over `input` and allocates nothing when
/// `input` is already clean (beyond the one-time automaton
/// construction).
#[doc(hidden)]
#[must_use]
pub fn normalize_str(input: &str) -> Option<String> {
    let mut matches = AUTOMATON.find_iter(input);
    // Nothing to do: one scan, no allocation.
    let first = matches.next()?;

    let mut normalized = String::with_capacity(input.len());
    let mut last = 0;
    let mut current = first;
    loop {
        normalized.push_str(&input[last..current.start()]);
        normalized.push_str(REPLACEMENTS[current.pattern().as_usize()].1);
        last = current.end();
        match matches.next() {
            Some(next) => current = next,
            None => break,
        }
    }
    normalized.push_str(&input[last..]);
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_input_returns_none() {
        assert_eq!(normalize_str("Already normalized ASCII text.\n"), None);
        assert_eq!(normalize_str(""), None);
    }

    #[test]
    fn each_replacement_is_applied() {
        // Embedding a pattern between plain ASCII isolates it: bare guillemets
        // have no trailing/leading space, and the space forms carry their own.
        for (pattern, replacement) in REPLACEMENTS {
            let input = format!("a{pattern}b");
            let expected = format!("a{replacement}b");
            assert_eq!(
                normalize_str(&input).as_deref(),
                Some(expected.as_str()),
                "pattern {pattern:?} should normalize to {replacement:?}"
            );
        }
    }

    #[test]
    fn left_guillemet_with_space_drops_the_space() {
        // Pins `LeftmostFirst`: "« " (with space) wins over bare "«" at the
        // shared start position, so the inner space is dropped.
        assert_eq!(normalize_str("« x").as_deref(), Some("\"x"));
    }

    #[test]
    fn right_guillemet_with_space_drops_the_space() {
        assert_eq!(normalize_str("x »").as_deref(), Some("x\""));
    }

    #[test]
    fn thin_space_uses_non_breaking_entities_before_french_punctuation() {
        assert_eq!(
            normalize_str("\u{2009};\u{2009}?\u{2009}!\u{2009}:\u{2009}x").as_deref(),
            Some("&#8239;;&#8239;?&#8239;!&nbsp;:&thinsp;x")
        );
    }

    #[test]
    fn multiple_replacements_in_one_pass() {
        assert_eq!(
            normalize_str("“Hello… world”").as_deref(),
            Some("\"Hello... world\"")
        );
    }
}
