//! C# XML doc comments.

use std::io::{Result, Write};

use crate::generation::{doc_comment, indent::IndentWrite};

/// Writes `lines` as a C# XML doc comment, at the current indentation of `w`.
///
/// The C# emitter writes each doc comment with this function. A plugin can
/// call it from a hook, so that its doc comments agree with the generated
/// ones. Give it one string for each line of the doc, for example the lines
/// of a Rust `///` comment.
///
/// The function trims trailing whitespace from each line and drops blank
/// lines at the start and at the end. If no text remains, it writes nothing.
/// Otherwise it writes `///` lines with the text in a `<summary>` element. A
/// blank line becomes `///`. The function escapes `&`, `<` and `>` in the
/// text, so that the text cannot break the XML.
///
/// # Errors
///
/// Returns an error if a write to `w` fails.
pub fn write_doc<S: AsRef<str>>(w: &mut dyn IndentWrite, lines: &[S]) -> Result<()> {
    write_doc_with_params::<_, S>(w, lines, &[])
}

/// Writes a C# XML doc comment like [`write_doc`], and after the `<summary>`
/// a `<param>` element for each `(name, comments)` pair, for the positional
/// parameters of a record.
///
/// If no parameter has a doc, no `<param>` element is written. Otherwise each
/// parameter gets one, empty if it has no doc, because the compiler warns
/// (CS1573) when only some parameters have a `<param>` element. A doc of one
/// line stays on the line of its `<param>` element.
pub(crate) fn write_doc_with_params<W: Write + ?Sized, S: AsRef<str>>(
    w: &mut W,
    comments: &[S],
    params: &[(&str, &[String])],
) -> Result<()> {
    let summary = doc_comment::lines(comments);
    if !summary.is_empty() {
        writeln!(w, "/// <summary>")?;
        write_xml_lines(w, &summary)?;
        writeln!(w, "/// </summary>")?;
    }

    if params
        .iter()
        .all(|(_, comments)| doc_comment::is_blank(comments))
    {
        return Ok(());
    }
    for (name, comments) in params {
        match doc_comment::lines(comments).as_slice() {
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

    fn doc_with_params(lines: &[&str], params: &[(&str, &[&str])]) -> String {
        let params: Vec<(&str, Vec<String>)> = params
            .iter()
            .map(|(name, lines)| (*name, lines.iter().map(ToString::to_string).collect()))
            .collect();
        let params: Vec<(&str, &[String])> = params
            .iter()
            .map(|(name, lines)| (*name, lines.as_slice()))
            .collect();
        let mut out = Vec::new();
        write_doc_with_params(&mut out, lines, &params).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn empty_doc_writes_nothing() {
        assert_eq!(doc(&[]), "");
        assert_eq!(doc(&[""]), "");
    }

    #[test]
    fn doc_with_blank_line_and_escaping() {
        assert_eq!(
            doc(&["Takes an `Option<String>` & more.", "", "a > b"]),
            "    /// <summary>\n    /// Takes an `Option&lt;String&gt;` &amp; more.\n    ///\n    /// a &gt; b\n    /// </summary>\n"
        );
    }

    #[test]
    fn doc_without_param_docs_writes_no_params() {
        assert_eq!(
            doc_with_params(&["Text."], &[("A", &[]), ("B", &[""])]),
            "/// <summary>\n/// Text.\n/// </summary>\n"
        );
        assert_eq!(doc_with_params(&[], &[("A", &[])]), "");
    }

    #[test]
    fn doc_with_param_docs() {
        insta::assert_snapshot!(
            doc_with_params(
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
    fn doc_with_params_and_no_summary() {
        insta::assert_snapshot!(doc_with_params(&[""], &[("One", &["Text."])]), @r#"/// <param name="One">Text.</param>"#);
    }
}
