//! Indentation-aware writer used by [`Emitter`](crate::generation::Emitter)
//! implementations to produce correctly indented source code.
//!
//! Wrap any [`Write`] in an [`IndentedWriter`](crate::generation::indent::IndentedWriter) and use [`indent()`] /
//! [`unindent()`] to change the nesting level — every new line is
//! automatically prefixed with the right amount of whitespace. For `{ }`
//! blocks, [`block()`] returns an RAII [`Block`](crate::generation::indent::Block) guard that indents on
//! creation and writes the closing brace on drop.
//!
//! [`indent()`]: crate::generation::indent::IndentWrite::indent
//! [`unindent()`]: crate::generation::indent::IndentWrite::unindent
//! [`block()`]: crate::generation::indent::IndentWrite::block

use std::io::{Result, Write};

/// How a single indentation level is represented in the output.
#[derive(Clone, Copy, Debug)]
pub enum IndentConfig {
    Tab,
    Space(usize),
}

/// Controls where extra newlines are inserted around a `{ }` block opened
/// by [`IndentWrite::block`]. See the constants for examples.
#[derive(Clone, Copy, Default)]
pub struct Newlines {
    pub after_open: usize,
    pub after_close: usize,
}

impl Newlines {
    /// No extra newlines around the braces.
    ///
    /// Produces `{<content>}` — rarely used in practice.
    pub const NONE: Self = Self {
        after_open: 0,
        after_close: 0,
    };

    /// Newline after the opening brace only.
    ///
    /// Produces:
    /// ```text
    /// {
    ///     <content>}
    /// ```
    ///
    /// Good for inline closures / trailing lambdas where more code follows
    /// the closing brace on the same line, e.g. TypeScript:
    /// ```text
    /// serializeArray(value, serializer, (item, serializer) => {
    ///     serializer.serializeStr(item);
    /// });
    /// ```
    /// or C# object initializers:
    /// ```text
    /// return new MyClass {
    ///     Foo = foo,
    /// };
    /// ```
    pub const OPEN: Self = Self {
        after_open: 1,
        after_close: 0,
    };

    /// Newline after the closing brace only.
    ///
    /// Produces:
    /// ```text
    /// {<content>}
    /// ```
    ///
    /// Good for empty or minimal bodies where the opening `{` belongs to
    /// the preceding declaration, e.g. Kotlin:
    /// ```text
    /// data class UnitVariant() {
    /// }
    /// ```
    pub const CLOSE: Self = Self {
        after_open: 0,
        after_close: 1,
    };

    /// Newlines after both braces — standard block formatting.
    ///
    /// Produces:
    /// ```text
    /// {
    ///     <content>
    /// }
    /// ```
    ///
    /// The most common choice — used for class bodies, function bodies,
    /// enum declarations, namespace blocks, etc.
    pub const BOTH: Self = Self {
        after_open: 1,
        after_close: 1,
    };
}

/// Extension of [`Write`] that tracks indentation level.
///
/// Call [`indent()`](Self::indent) / [`unindent()`](Self::unindent) to
/// change nesting depth, or use [`block()`](Self::block) for RAII-managed
/// `{ }` pairs. The concrete implementation is [`IndentedWriter`].
pub trait IndentWrite: Write {
    /// Returns the indentation config this writer was created with.
    ///
    /// This is object-safe (no `Self: Sized` bound) so it can be called
    /// through a `&mut dyn IndentWrite` trait object.
    fn config(&self) -> IndentConfig;

    fn indent(&mut self) {}
    fn unindent(&mut self) {}

    /// Start a new block with RAII-style automatic closing.
    ///
    /// Writes a brace and indents, returning a [`Block`] guard that implements [`IndentWrite`].
    /// When the guard is dropped it unindents and writes a closing brace.
    /// Newlines are inserted according to the configured [`Newlines`].
    ///
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn block(&mut self, newlines: Newlines) -> Result<Block<'_>>
    where
        Self: Sized,
    {
        write!(self, "{{")?;
        for _ in 0..newlines.after_open {
            writeln!(self)?;
        }
        self.indent();
        Ok(Block {
            writer: self,
            newlines,
        })
    }

    /// Create a child writer that writes to `out`, inheriting this writer's
    /// [`IndentConfig`].
    ///
    /// The child writes to its own separate `out` — the parent's buffer is
    /// unaffected.  Use this when output ordering forces you to buffer content
    /// before flushing it to the parent (e.g. writing feature helpers after
    /// imports).
    fn child<W: Write>(&self, out: W) -> IndentedWriter<W>
    where
        Self: Sized,
    {
        IndentedWriter::new(out, self.config())
    }

    /// Create a child writer that writes *through* this writer (i.e. uses the
    /// parent as its buffer), inheriting the [`IndentConfig`].
    ///
    /// The parent's current indentation becomes the baseline; the child's own
    /// [`indent`](Self::indent) / [`unindent`](Self::unindent) calls add on
    /// top of it.  The intermediate `Vec` + `write_all` pattern is eliminated
    /// entirely.
    fn child_writer(&mut self) -> IndentedWriter<&mut Self>
    where
        Self: Sized,
    {
        let config = self.config();
        IndentedWriter::new(self, config)
    }
}

/// RAII guard returned by [`IndentWrite::block`].
///
/// Implements [`Write`] and [`IndentWrite`] by delegating to the wrapped
/// writer. On [`Drop`], it unindents and writes the closing `}\n`.
pub struct Block<'a> {
    writer: &'a mut dyn IndentWrite,
    newlines: Newlines,
}

impl Write for Block<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}

impl IndentWrite for Block<'_> {
    fn config(&self) -> IndentConfig {
        self.writer.config()
    }

    fn indent(&mut self) {
        self.writer.indent();
    }

    fn unindent(&mut self) {
        self.writer.unindent();
    }
}

impl Drop for Block<'_> {
    fn drop(&mut self) {
        self.writer.unindent();
        let _ = write!(self.writer, "}}");
        for _ in 0..self.newlines.after_close {
            let _ = writeln!(self.writer);
        }
    }
}

/// Wraps any [`Write`] and automatically prepends the current indentation
/// at the start of each new line.
pub struct IndentedWriter<T> {
    out: T,
    indentation: Vec<u8>,
    config: IndentConfig,
    at_beginning_of_line: bool,
}

impl<T> IndentedWriter<T> {
    pub const fn new(out: T, config: IndentConfig) -> Self {
        Self {
            out,
            indentation: Vec::new(),
            config,
            at_beginning_of_line: true,
        }
    }
}

impl<T: Write> IndentWrite for IndentedWriter<T> {
    fn config(&self) -> IndentConfig {
        self.config
    }

    fn indent(&mut self) {
        match self.config {
            IndentConfig::Tab => {
                self.indentation.push(b'\t');
            }
            IndentConfig::Space(n) => {
                self.indentation.resize(self.indentation.len() + n, b' ');
            }
        }
    }

    fn unindent(&mut self) {
        match self.config {
            IndentConfig::Tab => {
                self.indentation.pop();
            }
            IndentConfig::Space(n) => {
                self.indentation
                    .truncate(self.indentation.len().saturating_sub(n));
            }
        }
    }
}

impl<T: Write> Write for IndentedWriter<T> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize> {
        let mut bytes_written = 0;

        while !buf.is_empty() {
            let (before_newline, has_newline, after_newline) =
                buf.iter().position(|&b| b == b'\n').map_or_else(
                    || (buf, false, &buf[buf.len()..]),
                    |idx| (&buf[..idx], true, &buf[idx + 1..]),
                );

            if self.at_beginning_of_line && !before_newline.is_empty() {
                self.out.write_all(&self.indentation)?;
                self.at_beginning_of_line = false;
            }

            self.out.write_all(before_newline)?;
            bytes_written += before_newline.len();

            if has_newline {
                self.out.write_all(b"\n")?;
                bytes_written += 1;
                self.at_beginning_of_line = true;
            }

            buf = after_newline;
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> Result<()> {
        self.out.flush()
    }
}

/// Creates a `{ }` block scope on a `dyn IndentWrite` writer.
///
/// This is the dynamic-dispatch equivalent of [`IndentWrite::block`].
/// Because `block()` requires `Self: Sized` (it stores `&mut dyn IndentWrite`
/// internally), it cannot be called through a trait object. This free function
/// fills that gap — it writes the opening brace, bumps indentation, invokes
/// the closure, then restores indentation and writes the closing brace.
///
/// # Example
///
/// ```rust,ignore
/// use facet_generate::generation::indent::{with_block, Newlines};
///
/// fn emit(w: &mut dyn IndentWrite) -> std::io::Result<()> {
///     write!(w, "fun serialize() ")?;
///     with_block(w, Newlines::BOTH, |w| {
///         writeln!(w, "// body")?;
///         Ok(())
///     })
/// }
/// ```
///
/// # Errors
///
/// Returns an error if writing the opening brace, invoking the closure, or
/// writing the closing brace fails.
pub fn with_block<F>(w: &mut dyn IndentWrite, newlines: Newlines, f: F) -> Result<()>
where
    F: FnOnce(&mut dyn IndentWrite) -> Result<()>,
{
    write!(w, "{{")?;
    for _ in 0..newlines.after_open {
        writeln!(w)?;
    }
    w.indent();
    let result = f(w);
    w.unindent();
    write!(w, "}}")?;
    for _ in 0..newlines.after_close {
        writeln!(w)?;
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut out = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        writeln!(out, "foo")?;
        out.indent();
        writeln!(out, "bar")?;
        writeln!(out)?;
        writeln!(out, "bar")?;
        out.indent();
        writeln!(out, "foobar")?;
        writeln!(out)?;
        writeln!(out, "foobar")?;
        out.unindent();
        out.unindent();
        writeln!(out, "foo")?;

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @"
        foo
          bar

          bar
            foobar

            foobar
        foo
        ");

        Ok(())
    }

    #[test]
    fn function_block() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut w = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        write!(w, "fn foo() ")?;
        {
            let mut w = w.block(Newlines::BOTH)?;
            writeln!(w, r#"let _ = "hello";"#)?;
        }

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @r#"
        fn foo() {
          let _ = "hello";
        }
        "#);

        Ok(())
    }

    #[test]
    fn closure_block() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut w = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        write!(w, "fn foo() ")?;
        {
            let mut w = w.block(Newlines::OPEN)?;
            write!(w, r#"let _ = "hello";"#)?;
        }

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @r#"
        fn foo() {
          let _ = "hello";}
        "#);

        Ok(())
    }

    #[test]
    fn trailing_block() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut w = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        write!(w, "fn foo() ")?;
        {
            let mut w = w.block(Newlines::CLOSE)?;
            write!(w, r#"let _ = "hello";"#)?;
        }

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @r#"fn foo() {let _ = "hello";}"#);

        Ok(())
    }

    // ----- child writer tests -----

    /// A child created with `parent.child(out)` writes to its own buffer but
    /// inherits the parent's `IndentConfig`, so callers never have to repeat
    /// the config at every call-site.
    #[test]
    fn child_inherits_config() -> Result<()> {
        // Use Space(3) – a deliberately unusual width so it is obvious the
        // child is not just picking up a hardcoded default.
        let mut parent_buf: Vec<u8> = Vec::new();
        let parent = IndentedWriter::new(&mut parent_buf, IndentConfig::Space(3));

        let mut child_buf: Vec<u8> = Vec::new();
        let mut child = parent.child(&mut child_buf);

        writeln!(child, "top")?;
        child.indent();
        writeln!(child, "indented")?;
        child.unindent();
        writeln!(child, "bottom")?;
        drop(child);

        // The parent's own buffer must be untouched – the child wrote to its
        // own separate buffer.
        assert!(parent_buf.is_empty());

        // The child used Space(3) inherited from the parent.
        insta::assert_snapshot!(String::from_utf8(child_buf).unwrap(), @"
top
   indented
bottom
");

        Ok(())
    }

    /// A child created with `parent.child_writer()` writes *through* the
    /// parent rather than to a separate buffer.  The parent's current
    /// indentation becomes the baseline, and the child's own indent/unindent
    /// calls add on top of it.
    #[test]
    fn child_writer_uses_parents_buffer() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut parent = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        writeln!(parent, "outer")?;
        parent.indent(); // parent is now at one level (2 spaces)

        {
            let mut child = parent.child_writer();
            writeln!(child, "middle")?; // gets parent's 2-space base
            child.indent();
            writeln!(child, "inner")?; // gets parent's 2 + child's 2 = 4 spaces
        } // child dropped; parent borrow released

        parent.unindent();
        writeln!(parent, "end")?;

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @"
outer
  middle
    inner
end
");

        Ok(())
    }

    /// When the parent is only available as `&mut dyn IndentWrite`, callers
    /// obtain the config with `w.config()` and then construct the child
    /// manually.  This covers the pattern used in bincode/kotlin where the
    /// intermediate `Vec` + `write_all` can be eliminated.
    #[test]
    fn child_writer_through_dyn_indent_write() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut parent = IndentedWriter::new(&mut buffer, IndentConfig::Space(4));
        parent.indent(); // simulate being one level deep (e.g., inside a class body)

        let w: &mut dyn IndentWrite = &mut parent;
        {
            let config = w.config();
            let mut child = IndentedWriter::new(&mut *w, config);

            writeln!(child, "content")?; // parent's 4-space base
            child.indent();
            writeln!(child, "nested")?; // parent's 4 + child's 4 = 8 spaces
        }

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @"
    content
        nested
");

        Ok(())
    }
}
