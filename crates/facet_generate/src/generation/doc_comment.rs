//! Doc-comment text rules that every language shares.
//!
//! Rust doc comments arrive as one string per `///` line. Each language module
//! writes them in its own doc-comment syntax (for example `kotlin::write_doc`),
//! and uses the functions here to prepare the lines first.
//!
//! # Links
//!
//! A rustdoc link to a Rust item points to nothing in the generated code. Thus
//! [`lines`] replaces each link to a Rust item with the text of the link, and
//! removes each reference definition line. A link to a URL stays a link. The
//! rules are:
//!
//! - An inline link, for example ``[`Value`](crate::Value)``, becomes its
//!   text, `` `Value` ``. The text keeps its code span.
//! - A shortcut link, for example ``[`Value`]``, becomes its text if a
//!   reference definition has its label. If no definition has the label, the
//!   link becomes its text only if the text is a Rust path. The path must be
//!   in one code span (``[`Value`]``), or have `::` (`[crate::Value]`). The
//!   path can have generic arguments, for example ``[`Vec<T>`]``.
//! - A full or collapsed reference link, for example
//!   ``[the value][`Value`]`` or ``[`Value`][]``, follows the same rule, with
//!   the reference in place of the text.
//! - A reference definition line, for example ``[`Value`]: crate::Value``, is
//!   removed. The links that use its label get its target.
//! - A target is a URL if it has `://`, or if it starts with `mailto:`. A
//!   Rust path cannot have `://`. Each other target is a Rust item, for
//!   example `crate::Value`, `struct@Value` or `Value::new()`.
//! - A link to a URL becomes `[text](url)` for [`Links::Markdown`], and
//!   `text (url)` for [`Links::Text`].
//!
//! The rules do not change code spans, fenced code blocks, images or escaped
//! brackets. Brackets that are not a link stay as they are, for example
//! `[0, 1]`, or `[note]` when no definition has the label `note`.
//!
//! A reference definition must start the doc, or come after a blank line or
//! after another definition. A line that continues a paragraph is not a
//! definition. A link can continue on the next line of a paragraph.

use std::collections::HashMap;

/// How [`lines`] writes a link to a URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Links {
    /// As an inline Markdown link, `[text](url)`, for languages whose doc
    /// tools read Markdown.
    #[cfg_attr(
        not(any(feature = "kotlin", feature = "swift", feature = "typescript")),
        allow(dead_code)
    )]
    Markdown,
    /// As plain text, `text (url)`, for languages whose doc tools do not read
    /// Markdown.
    Text,
}

/// Prepares the lines of a doc comment for a doc writer.
///
/// The function trims trailing whitespace from each line, and applies the
/// link rules of this module. Then it drops blank lines at the start and at
/// the end. If the removal of reference definitions puts two blank lines
/// together, it drops one of them.
pub(crate) fn lines<S: AsRef<str>>(comments: &[S], links: Links) -> Vec<String> {
    let trimmed: Vec<&str> = comments
        .iter()
        .map(|line| line.as_ref().trim_end())
        .collect();
    let code = code_lines(&trimmed);

    // A definition cannot interrupt a paragraph. It must come first, or
    // after a blank line, or after another definition.
    let mut definitions = HashMap::new();
    let mut is_definition = vec![false; trimmed.len()];
    for (index, line) in trimmed.iter().enumerate() {
        let follows_break = index == 0 || trimmed[index - 1].is_empty() || is_definition[index - 1];
        if code[index] || !follows_break {
            continue;
        }
        if let Some((label, target)) = definition(line) {
            definitions.entry(label).or_insert(target);
            is_definition[index] = true;
        }
    }

    let mut out: Vec<String> = Vec::with_capacity(trimmed.len());
    let mut removed = false;
    let mut index = 0;
    while index < trimmed.len() {
        let line = trimmed[index];
        if is_definition[index] {
            removed = true;
            index += 1;
            continue;
        }
        if line.is_empty() && removed && out.last().is_some_and(String::is_empty) {
            index += 1;
            continue;
        }
        removed = false;
        if code[index] || line.is_empty() {
            out.push(line.to_string());
            index += 1;
            continue;
        }
        // Rewrite a paragraph as one text, so that a link can continue on
        // the next line.
        let end = (index..trimmed.len())
            .find(|&i| code[i] || is_definition[i] || trimmed[i].is_empty())
            .unwrap_or(trimmed.len());
        let paragraph = trimmed[index..end].join("\n");
        out.extend(
            rewrite_links(&paragraph, &definitions, links)
                .split('\n')
                .map(ToString::to_string),
        );
        index = end;
    }

    let start = out.iter().position(|line| !line.is_empty());
    let end = out.iter().rposition(|line| !line.is_empty());
    match (start, end) {
        (Some(start), Some(end)) => out.drain(start..=end).collect(),
        _ => Vec::new(),
    }
}

/// Returns true if `comments` holds no text, so that a doc writer writes
/// nothing for it. A doc that holds only reference definitions has no text.
#[cfg_attr(
    not(any(feature = "swift", feature = "typescript", feature = "csharp")),
    allow(dead_code)
)]
pub(crate) fn is_blank<S: AsRef<str>>(comments: &[S]) -> bool {
    lines(comments, Links::Text).is_empty()
}

/// Marks each line that is part of a fenced code block, fences included. A
/// fence that is not closed continues to the end of the doc.
fn code_lines(lines: &[&str]) -> Vec<bool> {
    let mut code = Vec::with_capacity(lines.len());
    let mut open: Option<(char, usize)> = None;
    for line in lines {
        match (open, fence(line)) {
            (Some((kind, length)), Some((c, n, rest)))
                if c == kind && n >= length && rest.trim().is_empty() =>
            {
                code.push(true);
                open = None;
            }
            (Some(_), _) => code.push(true),
            (None, Some((c, n, _))) => {
                code.push(true);
                open = Some((c, n));
            }
            (None, None) => code.push(false),
        }
    }
    code
}

/// Returns the length of the run of `c` at the start of `text`.
fn run(text: &str, c: char) -> usize {
    text.len() - text.trim_start_matches(c).len()
}

/// If `line` starts a code fence (three or more `` ` `` or `~`, with at most
/// three spaces before them), returns the fence character, the fence length
/// and the text after the fence.
fn fence(line: &str) -> Option<(char, usize, &str)> {
    let text = line.trim_start_matches(' ');
    if line.len() - text.len() > 3 {
        return None;
    }
    let c = text.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let length = run(text, c);
    (length >= 3).then(|| (c, length, &text[length..]))
}

/// If `line` is a reference definition (`[label]: target`, with an optional
/// quoted title), returns its normalized label and its target.
fn definition(line: &str) -> Option<(String, String)> {
    let text = line.trim_start_matches(' ');
    if line.len() - text.len() > 3 || !text.starts_with('[') {
        return None;
    }
    let close = closing_bracket(text, 0)?;
    let label = &text[1..close];
    let rest = text[close + 1..].strip_prefix(':')?.trim_start();
    if label.trim().is_empty() || rest.is_empty() {
        return None;
    }
    let (target, title) = if let Some(rest) = rest.strip_prefix('<') {
        let end = rest.find('>')?;
        (&rest[..end], rest[end + 1..].trim())
    } else {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        (&rest[..end], rest[end..].trim())
    };
    let quoted = |open: char, close: char| {
        title.len() >= 2 && title.starts_with(open) && title.ends_with(close)
    };
    if !(title.is_empty() || quoted('"', '"') || quoted('\'', '\'') || quoted('(', ')')) {
        return None;
    }
    Some((normalize(label), target.to_string()))
}

/// Normalizes a link label, as `CommonMark` does: case does not matter, and a
/// run of whitespace is one space.
fn normalize(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Returns true if `target` is a URL.
fn is_url(target: &str) -> bool {
    target.contains("://") || target.to_ascii_lowercase().starts_with("mailto:")
}

/// Returns true if `text` is a Rust path that rustdoc can link to, for
/// example `Value`, `crate::Value::None`, `struct@Value`, `Value::new()` or
/// `vec!`.
fn is_path(text: &str) -> bool {
    let Some(text) = without_generics(text) else {
        return false;
    };
    let mut path = text.as_str();
    if let Some((prefix, rest)) = path.split_once('@')
        && !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_alphabetic())
    {
        path = rest;
    }
    let path = path
        .strip_suffix("()")
        .or_else(|| path.strip_suffix('!'))
        .unwrap_or(path);
    let segments = path.strip_prefix("::").unwrap_or(path);
    !segments.is_empty()
        && segments.split("::").all(|segment| {
            segment
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
                && segment.chars().all(|c| c.is_alphanumeric() || c == '_')
        })
}

/// Removes the generic arguments from a path, for example `Vec<T>` becomes
/// `Vec`, as rustdoc does when it resolves a link. Returns `None` if the
/// angle brackets do not match.
fn without_generics(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0_usize;
    for c in text.chars() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.checked_sub(1)?,
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    (depth == 0).then_some(out)
}

/// Returns true if the text of a link with no definition names a Rust item.
/// The text must be a Rust path in one code span (``[`Value`]``), or a plain
/// Rust path with `::` (`[crate::Value]`). A plain word such as `[note]` does
/// not name a Rust item.
fn names_item(text: &str) -> bool {
    let text = text.trim();
    whole_code_span(text).map_or_else(
        || text.contains("::") && is_path(text),
        |inner| is_path(inner.trim()),
    )
}

/// If `text` is exactly one code span, returns the text in it.
fn whole_code_span(text: &str) -> Option<&str> {
    if !text.starts_with('`') {
        return None;
    }
    let end = code_span_end(text, 0)?;
    let ticks = run(text, '`');
    (end == text.len()).then(|| &text[ticks..end - ticks])
}

/// If a code span starts at byte `start` of `text`, returns the byte index
/// after its closing backticks.
fn code_span_end(text: &str, start: usize) -> Option<usize> {
    let ticks = run(&text[start..], '`');
    let mut index = start + ticks;
    while let Some(offset) = text[index..].find('`') {
        let run_start = index + offset;
        let length = run(&text[run_start..], '`');
        if length == ticks {
            return Some(run_start + length);
        }
        index = run_start + length;
    }
    None
}

/// Returns the byte index of the `]` that closes the `[` at byte `open` of
/// `text`. Brackets nest, and brackets in code spans or after a `\` do not
/// count.
fn closing_bracket(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'`' => {
                index =
                    code_span_end(text, index).unwrap_or_else(|| index + run(&text[index..], '`'));
            }
            b'[' => {
                depth += 1;
                index += 1;
            }
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}

/// Returns the byte index of the `)` that closes the `(` at byte `open` of
/// `text`. Parentheses nest, and parentheses after a `\` do not count.
const fn closing_parenthesis(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 1,
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// A link that starts at a `[`.
struct Link<'a> {
    /// The text in the brackets.
    text: &'a str,
    /// The target. A link to a Rust item with no definition has no target.
    target: Option<&'a str>,
    /// The byte index after the link.
    end: usize,
}

/// If a link starts at the `[` at byte `open` of `line`, returns it.
fn link_at<'a>(
    line: &'a str,
    open: usize,
    definitions: &'a HashMap<String, String>,
) -> Option<Link<'a>> {
    let close = closing_bracket(line, open)?;
    let text = &line[open + 1..close];
    let after = close + 1;
    let new = |target, end| Link { text, target, end };
    let reference = |label: &str, end| {
        definitions.get(&normalize(label)).map_or_else(
            || names_item(label).then(|| new(None, end)),
            |target| Some(new(Some(target.as_str()), end)),
        )
    };

    // An inline link: `[text](target)`.
    if line[after..].starts_with('(')
        && let Some(end) = closing_parenthesis(line, after)
    {
        let inner = line[after + 1..end].trim();
        let target = match inner.strip_prefix('<') {
            Some(rest) => rest.split('>').next().unwrap_or(rest),
            None => inner.split_whitespace().next().unwrap_or(inner),
        };
        return Some(new(Some(target), end + 1));
    }

    // A full or collapsed reference link: `[text][label]` or `[text][]`.
    if line[after..].starts_with('[')
        && let Some(end) = closing_bracket(line, after)
    {
        let label = &line[after + 1..end];
        let label = if label.trim().is_empty() { text } else { label };
        return reference(label, end + 1);
    }

    // A shortcut link: `[text]`.
    reference(text, after)
}

/// Applies the link rules to one line that is not in a code block.
fn rewrite_links(line: &str, definitions: &HashMap<String, String>, links: Links) -> String {
    let mut out = String::with_capacity(line.len());
    let mut index = 0;
    while let Some(c) = line[index..].chars().next() {
        let rest = &line[index..];
        let end = match c {
            '\\' => index + rest.chars().take(2).map(char::len_utf8).sum::<usize>(),
            '`' => code_span_end(line, index).unwrap_or_else(|| index + run(rest, '`')),
            // An image stays as it is.
            '!' if rest.starts_with("![") => {
                link_at(line, index + 1, definitions).map_or(index + 2, |link| link.end)
            }
            // In text, an autolink such as `<https://example.com>` becomes its URL.
            '<' if links == Links::Text => {
                if let Some(url) = rest[1..]
                    .split_once('>')
                    .map(|(url, _)| url)
                    .filter(|url| is_url(url) && !url.contains(char::is_whitespace))
                {
                    out.push_str(url);
                    index += url.len() + 2;
                    continue;
                }
                index + 1
            }
            '[' => {
                if let Some(link) = link_at(line, index, definitions) {
                    write_link(&mut out, &link, links);
                    index = link.end;
                    continue;
                }
                index + 1
            }
            c => index + c.len_utf8(),
        };
        out.push_str(&line[index..end]);
        index = end;
    }
    out
}

/// Writes `link` to `out`: a link to a Rust item as its text, and a link to a
/// URL in the form that `links` gives.
fn write_link(out: &mut String, link: &Link<'_>, links: Links) {
    match (link.target, links) {
        (Some(url), Links::Markdown) if is_url(url) => {
            out.push('[');
            out.push_str(link.text);
            out.push_str("](");
            out.push_str(url);
            out.push(')');
        }
        (Some(url), Links::Text) if is_url(url) => {
            if link.text.trim() != url {
                out.push_str(link.text);
                out.push_str(" (");
            }
            out.push_str(url);
            if link.text.trim() != url {
                out.push(')');
            }
        }
        _ => out.push_str(link.text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn markdown(comments: &[&str]) -> Vec<String> {
        lines(comments, Links::Markdown)
    }

    fn text(comments: &[&str]) -> Vec<String> {
        lines(comments, Links::Text)
    }

    #[test]
    fn lines_trims_and_drops_outer_blank_lines() {
        assert_eq!(
            markdown(&["", " ", "First line.  ", "", "More text.", "  "]),
            ["First line.", "", "More text."]
        );
        assert_eq!(lines::<&str>(&[], Links::Markdown), Vec::<String>::new());
        assert_eq!(markdown(&["", "  "]), Vec::<String>::new());
    }

    #[test]
    fn is_blank_is_true_only_without_text() {
        assert!(is_blank::<&str>(&[]));
        assert!(is_blank(&["", " \t"]));
        assert!(!is_blank(&["", "Text."]));
        assert!(is_blank(&[String::new()]));
    }

    #[test]
    fn is_blank_is_true_for_only_reference_definitions() {
        assert!(is_blank(&["[`Value`]: crate::Value", ""]));
        assert!(!is_blank(&["[`Value`]: crate::Value", "Text."]));
    }

    #[test]
    fn inline_link_to_rust_item_becomes_its_text() {
        assert_eq!(
            markdown(&["Returns [`Value::None`](crate::protocol::Value::None) here."]),
            ["Returns `Value::None` here."]
        );
        assert_eq!(
            text(&["See [the value](super::Value) and [`new`](Value::new())."]),
            ["See the value and `new`."]
        );
        assert_eq!(
            markdown(&["[`Value`](struct@crate::Value), [len](#method.len)"]),
            ["`Value`, len"]
        );
    }

    #[test]
    fn shortcut_link_to_rust_item_becomes_its_text() {
        assert_eq!(
            markdown(&["One [`DataResult`] or a [`KeyValueError::CursorNotFound`]."]),
            ["One `DataResult` or a `KeyValueError::CursorNotFound`."]
        );
        assert_eq!(
            markdown(&["Use [crate::Value], [`vec!`], [`Value::new()`] and [``a``]."]),
            ["Use crate::Value, `vec!`, `Value::new()` and ``a``."]
        );
    }

    #[test]
    fn reference_links_become_their_text() {
        assert_eq!(
            markdown(&["The [none value][`Value::None`] and [`Value`][]."]),
            ["The none value and `Value`."]
        );
        assert_eq!(
            markdown(&["The [none value][none].", "", "[none]: crate::Value::None"]),
            ["The none value."]
        );
        assert_eq!(
            markdown(&["A [Note] and a [note][NOTE].", "", "[note]: Value"]),
            ["A Note and a note."]
        );
    }

    #[test]
    fn reference_definitions_to_rust_items_are_removed() {
        assert_eq!(
            markdown(&[
                "Returns [`Value::None`].",
                "",
                "[`Value::None`]: crate::protocol::Value::None",
                "[`DataResult`]: super::DataResult",
                "",
            ]),
            ["Returns `Value::None`."]
        );
        assert_eq!(
            markdown(&["First [`A`].", "", "[`A`]: crate::A", "", "Second."]),
            ["First `A`.", "", "Second."]
        );
        assert_eq!(markdown(&["[a]: crate::A", "", "Text."]), ["Text."]);
    }

    #[test]
    fn external_links_stay_links_in_markdown() {
        assert_eq!(
            markdown(&["See [the docs](https://example.com/docs \"Docs\")."]),
            ["See [the docs](https://example.com/docs)."]
        );
        assert_eq!(
            markdown(&[
                "See [the docs][docs] and [`serde`].",
                "",
                "[docs]: https://example.com/docs",
                "[`serde`]: <https://serde.rs> 'Serde'",
            ]),
            ["See [the docs](https://example.com/docs) and [`serde`](https://serde.rs)."]
        );
        assert_eq!(
            markdown(&["Mail [us](mailto:a@example.com) or <https://example.com>."]),
            ["Mail [us](mailto:a@example.com) or <https://example.com>."]
        );
    }

    #[test]
    fn external_links_become_text_and_url_in_text() {
        assert_eq!(
            text(&["See [the docs](https://example.com/docs)."]),
            ["See the docs (https://example.com/docs)."]
        );
        assert_eq!(
            text(&[
                "See [docs][] and <https://serde.rs>.",
                "",
                "[docs]: http://example.com"
            ]),
            ["See docs (http://example.com) and https://serde.rs."]
        );
        assert_eq!(
            text(&["[https://example.com](https://example.com)", "a <b> c <"]),
            ["https://example.com", "a <b> c <"]
        );
    }

    #[test]
    fn code_spans_stay_as_they_are() {
        let line = "Takes `[u8; 32]`, a `Vec<[u8]>` and ``[`X`](crate::X)``.";
        assert_eq!(markdown(&[line]), [line]);
        assert_eq!(
            markdown(&["A [`Vec<[u8]>`](Vec) and a span ` [`A`]."]),
            ["A `Vec<[u8]>` and a span ` [`A`]."]
        );
    }

    #[test]
    fn fenced_code_blocks_stay_as_they_are() {
        let doc = [
            "Example:",
            "",
            "```rust",
            "#[effect]",
            "let a = [`X`](crate::X);",
            "[x]: crate::X",
            "```",
            "",
            "~~~~",
            "```",
            "[`Y`]",
            "~~~~",
            "After [`Z`].",
        ];
        let expected: Vec<String> = doc[..12]
            .iter()
            .map(ToString::to_string)
            .chain(["After `Z`.".to_string()])
            .collect();
        assert_eq!(markdown(&doc), expected);
    }

    #[test]
    fn plain_brackets_stay_as_they_are() {
        for line in [
            "An array [0, 1] and a [note].",
            "Index a[i] or [i][j].",
            "Not a path [`[u8; 32]`] or [`a b`].",
            r"Escaped \[`X`\] and \[`Y`](crate::Y).",
            "An image ![logo](crate::Logo) and ![`x`].",
            "Unclosed [`X` and `Y` here.",
            "A `[`, then [a]: b c in prose.",
            "Unicode [é] and ünïcödé.",
        ] {
            assert_eq!(markdown(&[line]), [line], "{line}");
        }
    }

    #[test]
    fn link_can_continue_on_the_next_line() {
        assert_eq!(
            markdown(&[
                "See [the",
                "value](crate::Value) and [the",
                "guide][docs].",
                "",
                "[docs]: https://example.com"
            ]),
            ["See the", "value and [the", "guide](https://example.com)."]
        );
        assert_eq!(
            text(&[
                "See [the",
                "guide][docs].",
                "",
                "[docs]: https://example.com"
            ]),
            ["See the", "guide (https://example.com)."]
        );
    }

    #[test]
    fn definition_cannot_interrupt_a_paragraph() {
        assert_eq!(
            markdown(&["Returns one of:", "[`Ok`]: success"]),
            ["Returns one of:", "`Ok`: success"]
        );
        assert_eq!(
            markdown(&[
                "Text.",
                "",
                "[a]: crate::A",
                "[b]: crate::B",
                "Uses [a] and [b]."
            ]),
            ["Text.", "", "Uses a and b."]
        );
    }

    #[test]
    fn shortcut_link_with_generics_becomes_its_text() {
        assert_eq!(
            markdown(&["A [`Vec<T>`] or a [`HashMap<K, V>::new()`]."]),
            ["A `Vec<T>` or a `HashMap<K, V>::new()`."]
        );
    }

    #[test]
    fn rust_paths_in_text_stay_as_they_are() {
        assert_eq!(
            markdown(&["Returns Value::None, not [`Value::Some`]."]),
            ["Returns Value::None, not `Value::Some`."]
        );
    }

    #[test]
    fn is_path_accepts_rustdoc_paths_only() {
        for path in [
            "Value",
            "crate::Value::None",
            "::std::vec::Vec",
            "struct@Value",
            "Value::new()",
            "vec!",
            "_private",
            "Vec<T>",
            "HashMap<K, V>::new()",
        ] {
            assert!(is_path(path), "{path}");
        }
        for path in [
            "", "0", "a b", "Vec<u8", "Vec>", "a::", "::", "[u8; 32]", "`Value`",
        ] {
            assert!(!is_path(path), "{path}");
        }
    }
}
