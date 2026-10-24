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

/// Returns true if `comments` holds no text, so that a doc writer writes
/// nothing for it.
pub(crate) fn is_blank(comments: &[String]) -> bool {
    lines(comments).is_empty()
}

/// Writes `comments` as a C# XML doc comment: `///` lines with the text in a
/// `<summary>` element. The text is XML-escaped (`&`, `<` and `>`). A blank
/// line becomes `///`.
pub(crate) fn write_xml_doc<W: Write + ?Sized>(w: &mut W, comments: &[String]) -> Result<()> {
    write_xml_doc_with_params(w, comments, &[])
}

/// Writes a C# XML doc comment like [`write_xml_doc`], and after the
/// `<summary>` a `<param>` element for each `(name, comments)` pair, for the
/// positional parameters of a record.
///
/// If no parameter has a doc, no `<param>` element is written. Otherwise each
/// parameter gets one, empty if it has no doc, because the compiler warns
/// (CS1573) when only some parameters have a `<param>` element. A doc of one
/// line stays on the line of its `<param>` element.
pub(crate) fn write_xml_doc_with_params<W: Write + ?Sized>(
    w: &mut W,
    comments: &[String],
    params: &[(&str, &[String])],
) -> Result<()> {
    let summary = lines(comments);
    if !summary.is_empty() {
        writeln!(w, "/// <summary>")?;
        write_xml_lines(w, &summary)?;
        writeln!(w, "/// </summary>")?;
    }

    if params.iter().all(|(_, comments)| is_blank(comments)) {
        return Ok(());
    }
    for (name, comments) in params {
        match lines(comments).as_slice() {
            [] => writeln!(w, r#"/// <param name="{name}"></param>"#)?,
            [line] => writeln!(
                w,
                r#"/// <param name="{name}">{}</param>"#,
                xml_escape(line)
            )?,
            lines => {
                writeln!(w, r#"/// <param name="{name}">"#)?;
                write_xml_lines(w, lines)?;
                writeln!(w, "/// </param>")?;
            }
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
pub(crate) fn write_swift_parameters<W: Write + ?Sized>(
    w: &mut W,
    params: &[(&str, &[String])],
    separate: bool,
) -> Result<()> {
    if params.iter().all(|(_, comments)| is_blank(comments)) {
        return Ok(());
    }
    if separate {
        writeln!(w, "///")?;
    }
    writeln!(w, "/// - Parameters:")?;
    for (name, comments) in params {
        let lines = lines(comments);
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

fn write_xml_lines<W: Write + ?Sized>(w: &mut W, lines: &[&str]) -> Result<()> {
    for line in lines {
        if line.is_empty() {
            writeln!(w, "///")?;
        } else {
            writeln!(w, "/// {}", xml_escape(line))?;
        }
    }
    Ok(())
}

fn xml_escape(line: &str) -> String {
    line.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
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

    fn xml_with_params(lines: &[&str], params: &[(&str, &[&str])]) -> String {
        let params: Vec<(&str, Vec<String>)> = params
            .iter()
            .map(|(name, lines)| (*name, doc(lines)))
            .collect();
        let params: Vec<(&str, &[String])> = params
            .iter()
            .map(|(name, lines)| (*name, lines.as_slice()))
            .collect();
        let mut out = Vec::new();
        write_xml_doc_with_params(&mut out, &doc(lines), &params).unwrap();
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

    #[test]
    fn xml_doc_without_param_docs_writes_no_params() {
        assert_eq!(
            xml_with_params(&["Text."], &[("A", &[]), ("B", &[""])]),
            "/// <summary>\n/// Text.\n/// </summary>\n"
        );
        assert_eq!(xml_with_params(&[], &[("A", &[])]), "");
    }

    #[test]
    fn xml_doc_with_params() {
        insta::assert_snapshot!(
            xml_with_params(
                &["A record."],
                &[("One", &["For `Vec<u8>` & more."]), ("Two", &[]), ("Three", &["Two lines,", "", "a > b"])],
            ),
            @r#"
        /// <summary>
        /// A record.
        /// </summary>
        /// <param name="One">For `Vec&lt;u8&gt;` &amp; more.</param>
        /// <param name="Two"></param>
        /// <param name="Three">
        /// Two lines,
        ///
        /// a &gt; b
        /// </param>
        "#
        );
    }

    #[test]
    fn xml_doc_with_params_and_no_summary() {
        insta::assert_snapshot!(xml_with_params(&[""], &[("One", &["Text."])]), @r#"/// <param name="One">Text.</param>"#);
    }

    #[test]
    fn swift_parameters() {
        let params = |params: &[(&str, &[&str])], separate: bool| {
            let params: Vec<(&str, Vec<String>)> = params
                .iter()
                .map(|(name, lines)| (*name, doc(lines)))
                .collect();
            let params: Vec<(&str, &[String])> = params
                .iter()
                .map(|(name, lines)| (*name, lines.as_slice()))
                .collect();
            let mut out = Vec::new();
            write_swift_parameters(&mut out, &params, separate).unwrap();
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
