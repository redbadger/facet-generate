//! Doc-comment rendering shared by the language emitters.
//!
//! Rust doc comments arrive as one string per `///` line. Each target language
//! has its own doc-comment syntax, and the text must not be able to end the
//! comment early or break its markup.

use std::io::{Result, Write};

/// Trims trailing whitespace from each line, and drops blank lines at the
/// start and at the end.
fn lines(comments: &[String]) -> Vec<&str> {
    let lines: Vec<&str> = comments.iter().map(|line| line.trim_end()).collect();
    let start = lines.iter().position(|line| !line.is_empty());
    let end = lines.iter().rposition(|line| !line.is_empty());
    match (start, end) {
        (Some(start), Some(end)) => lines[start..=end].to_vec(),
        _ => Vec::new(),
    }
}

/// Writes `comments` as a `/** ... */` block comment (`KDoc`, `TSDoc` or `JSDoc`).
///
/// A doc of one line becomes `/** Text. */`. A longer doc puts `/**` and `*/`
/// on their own lines and starts each line of text with ` * `. A blank line
/// becomes ` *`.
///
/// Each `*/` in the text becomes `*\/`, so that the text cannot end the
/// comment. If `nested` is true, each `/*` also becomes `/\*`, because in a
/// language with nested block comments (Kotlin) the text must not open one.
pub(crate) fn write_block_doc<W: Write + ?Sized>(
    w: &mut W,
    comments: &[String],
    nested: bool,
) -> Result<()> {
    let escape = |line: &str| {
        let line = line.replace("*/", "*\\/");
        if nested {
            line.replace("/*", "/\\*")
        } else {
            line
        }
    };

    match lines(comments).as_slice() {
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

/// Writes `comments` as a C# XML doc comment: `///` lines with the text in a
/// `<summary>` element. The text is XML-escaped (`&`, `<` and `>`). A blank
/// line becomes `///`.
pub(crate) fn write_xml_doc<W: Write + ?Sized>(w: &mut W, comments: &[String]) -> Result<()> {
    let lines = lines(comments);
    if lines.is_empty() {
        return Ok(());
    }

    writeln!(w, "/// <summary>")?;
    for line in lines {
        if line.is_empty() {
            writeln!(w, "///")?;
        } else {
            let line = line
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            writeln!(w, "/// {line}")?;
        }
    }
    writeln!(w, "/// </summary>")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(lines: &[&str]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    fn block(lines: &[&str], nested: bool) -> String {
        let mut out = Vec::new();
        write_block_doc(&mut out, &doc(lines), nested).unwrap();
        String::from_utf8(out).unwrap()
    }

    fn xml(lines: &[&str]) -> String {
        let mut out = Vec::new();
        write_xml_doc(&mut out, &doc(lines)).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn empty_doc_writes_nothing() {
        assert_eq!(block(&[], false), "");
        assert_eq!(block(&["", " "], true), "");
        assert_eq!(xml(&[]), "");
        assert_eq!(xml(&[""]), "");
    }

    #[test]
    fn single_line_block_doc() {
        assert_eq!(block(&["Text."], false), "/** Text. */\n");
    }

    #[test]
    fn multi_line_block_doc_with_blank_line() {
        insta::assert_snapshot!(block(&["", "First line.", "", "More text.  ", ""], false), @r"
        /**
         * First line.
         *
         * More text.
         */
        ");
    }

    #[test]
    fn block_doc_escapes_comment_end() {
        assert_eq!(block(&["a */ b"], false), "/** a *\\/ b */\n");
        assert_eq!(block(&["a /* b"], false), "/** a /* b */\n");
    }

    #[test]
    fn nested_block_doc_escapes_comment_start() {
        assert_eq!(block(&["a /* b */ c"], true), "/** a /\\* b *\\/ c */\n");
        assert_eq!(block(&["*/*"], true), "/** *\\/\\* */\n");
    }

    #[test]
    fn xml_doc_with_blank_line_and_escaping() {
        insta::assert_snapshot!(xml(&["Takes an `Option<String>` & more.", "", "a > b"]), @r"
        /// <summary>
        /// Takes an `Option&lt;String&gt;` &amp; more.
        ///
        /// a &gt; b
        /// </summary>
        ");
    }
}
