//! Doc-comment text rules that every language shares.
//!
//! Rust doc comments arrive as one string per `///` line. Each language module
//! writes them in its own doc-comment syntax (for example `kotlin::write_doc`),
//! and uses the functions here to prepare the lines first.

/// Trims trailing whitespace from each line, and drops blank lines at the
/// start and at the end.
pub(crate) fn lines<S: AsRef<str>>(comments: &[S]) -> Vec<&str> {
    let lines: Vec<&str> = comments
        .iter()
        .map(|line| line.as_ref().trim_end())
        .collect();
    let start = lines.iter().position(|line| !line.is_empty());
    let end = lines.iter().rposition(|line| !line.is_empty());
    match (start, end) {
        (Some(start), Some(end)) => lines[start..=end].to_vec(),
        _ => Vec::new(),
    }
}

/// Returns true if `comments` holds no text, so that a doc writer writes
/// nothing for it.
#[cfg_attr(
    not(any(feature = "swift", feature = "typescript", feature = "csharp")),
    allow(dead_code)
)]
pub(crate) fn is_blank<S: AsRef<str>>(comments: &[S]) -> bool {
    comments
        .iter()
        .all(|line| line.as_ref().trim_end().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_trims_and_drops_outer_blank_lines() {
        assert_eq!(
            lines(&["", " ", "First line.  ", "", "More text.", "  "]),
            ["First line.", "", "More text."]
        );
        assert_eq!(lines::<&str>(&[]), Vec::<&str>::new());
        assert_eq!(lines(&["", "  "]), Vec::<&str>::new());
    }

    #[test]
    fn is_blank_is_true_only_without_text() {
        assert!(is_blank::<&str>(&[]));
        assert!(is_blank(&["", " \t"]));
        assert!(!is_blank(&["", "Text."]));
        assert!(is_blank(&[String::new()]));
    }
}
