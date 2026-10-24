//! Swift doc comments.

use std::io::{Result, Write};

use crate::generation::{doc_comment, indent::IndentWrite};

/// Writes `lines` as a Swift doc comment, at the current indentation of `w`.
///
/// The Swift emitter writes each doc comment with this function. A plugin can
/// call it from a hook, so that its doc comments agree with the generated
/// ones. Give it one string for each line of the doc, for example the lines
/// of a Rust `///` comment.
///
/// The function trims trailing whitespace from each line and drops blank
/// lines at the start and at the end. If no text remains, it writes nothing.
/// Otherwise it starts each line of text with `/// `. A blank line becomes
/// `///`. A line comment has no end marker, so the text needs no escapes.
///
/// # Errors
///
/// Returns an error if a write to `w` fails.
pub fn write_doc<S: AsRef<str>>(w: &mut dyn IndentWrite, lines: &[S]) -> Result<()> {
    for line in doc_comment::lines(lines) {
        if line.is_empty() {
            writeln!(w, "///")?;
        } else {
            writeln!(w, "/// {line}")?;
        }
    }
    Ok(())
}

/// Writes a Swift `- Parameters:` list in `///` lines, with an item for each
/// `(name, comments)` pair that has a doc. It documents the associated values
/// of an enum case. If no parameter has a doc, nothing is written.
///
/// If `separate` is true, a blank `///` line comes first, to keep the list
/// apart from the text before it. The second and later lines of an item are
/// indented, so that they stay in the item.
pub(crate) fn write_parameters<W: Write + ?Sized>(
    w: &mut W,
    params: &[(&str, &[String])],
    separate: bool,
) -> Result<()> {
    if params
        .iter()
        .all(|(_, comments)| doc_comment::is_blank(comments))
    {
        return Ok(());
    }
    if separate {
        writeln!(w, "///")?;
    }
    writeln!(w, "/// - Parameters:")?;
    for (name, comments) in params {
        let lines = doc_comment::lines(comments);
        let Some((first, rest)) = lines.split_first() else {
            continue;
        };
        writeln!(w, "///   - {name}: {first}")?;
        for line in rest {
            if line.is_empty() {
                writeln!(w, "///")?;
            } else {
                writeln!(w, "///     {line}")?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::indent::{IndentConfig, IndentedWriter};

    /// Calls `write_doc` through a `&mut dyn IndentWrite`, one level deep, as
    /// a plugin hook does.
    fn doc(lines: &[&str]) -> String {
        let mut out = Vec::new();
        let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
        w.indent();
        write_doc(&mut w as &mut dyn IndentWrite, lines).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn empty_doc_writes_nothing() {
        assert_eq!(doc(&[]), "");
        assert_eq!(doc(&["", " "]), "");
    }

    #[test]
    fn doc_with_blank_lines() {
        assert_eq!(
            doc(&["", "First line.  ", "", "More text.", ""]),
            "    /// First line.\n    ///\n    /// More text.\n"
        );
    }

    #[test]
    fn parameters() {
        let params = |params: &[(&str, &[&str])], separate: bool| {
            let params: Vec<(&str, Vec<String>)> = params
                .iter()
                .map(|(name, lines)| (*name, lines.iter().map(ToString::to_string).collect()))
                .collect();
            let params: Vec<(&str, &[String])> = params
                .iter()
                .map(|(name, lines)| (*name, lines.as_slice()))
                .collect();
            let mut out = Vec::new();
            write_parameters(&mut out, &params, separate).unwrap();
            String::from_utf8(out).unwrap()
        };

        assert_eq!(params(&[("one", &[]), ("two", &[" "])], true), "");
        insta::assert_snapshot!(
            params(&[("one", &["Text."]), ("two", &[]), ("three", &["Two lines,", "", "a > b"])], true),
            @"
        ///
        /// - Parameters:
        ///   - one: Text.
        ///   - three: Two lines,
        ///
        ///     a > b
        "
        );
    }
}
