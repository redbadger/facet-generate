//! Kotlin doc comments (`KDoc`).

use std::io::Result;

use crate::generation::{doc_comment, indent::IndentWrite};

/// Writes `lines` as a Kotlin doc comment (`KDoc`), at the current
/// indentation of `w`.
///
/// The Kotlin emitter writes each doc comment with this function. A plugin
/// can call it from a hook, so that its doc comments agree with the generated
/// ones. Give it one string for each line of the doc, for example the lines
/// of a Rust `///` comment.
///
/// The function trims trailing whitespace from each line and drops blank
/// lines at the start and at the end. If no text remains, it writes nothing.
/// A doc of one line becomes `/** Text. */`. A longer doc puts `/**` and `*/`
/// on their own lines and starts each line of text with ` * `.
///
/// Because Kotlin block comments nest, each `*/` in the text becomes `*\/`,
/// and each `/*` becomes `/\*`. Thus the text cannot close the comment or
/// open a new one.
///
/// # Errors
///
/// Returns an error if a write to `w` fails.
pub fn write_doc<S: AsRef<str>>(w: &mut dyn IndentWrite, lines: &[S]) -> Result<()> {
    let escape = |line: &str| line.replace("*/", "*\\/").replace("/*", "/\\*");

    match doc_comment::lines(lines).as_slice() {
        [] => Ok(()),
        [line] => writeln!(w, "/** {} */", escape(line)),
        lines => {
            writeln!(w, "/**")?;
            for line in lines {
                if line.is_empty() {
                    writeln!(w, " *")?;
                } else {
                    writeln!(w, " * {}", escape(line))?;
                }
            }
            writeln!(w, " */")
        }
    }
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
    fn single_line_doc() {
        assert_eq!(doc(&["Text."]), "    /** Text. */\n");
    }

    #[test]
    fn multi_line_doc_with_blank_line() {
        assert_eq!(
            doc(&["", "First line.", "", "More text.  ", ""]),
            "    /**\n     * First line.\n     *\n     * More text.\n     */\n"
        );
    }

    #[test]
    fn doc_escapes_comment_start_and_end() {
        assert_eq!(doc(&["a /* b */ c"]), "    /** a /\\* b *\\/ c */\n");
        assert_eq!(doc(&["*/*"]), "    /** *\\/\\* */\n");
    }

    #[test]
    fn takes_owned_lines() {
        let lines = vec!["Text.".to_string()];
        let mut out = Vec::new();
        write_doc(
            &mut IndentedWriter::new(&mut out, IndentConfig::Space(4)),
            &lines,
        )
        .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "/** Text. */\n");
    }
}
