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
#[derive(Clone, Copy)]
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
    pub fn new(out: T, config: IndentConfig) -> Self {
        Self {
            out,
            indentation: Vec::new(),
            config,
            at_beginning_of_line: true,
        }
    }
}

impl<T: Write> IndentWrite for IndentedWriter<T> {
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
                if let Some(idx) = buf.iter().position(|&b| b == b'\n') {
                    (&buf[..idx], true, &buf[idx + 1..])
                } else {
                    (buf, false, &buf[buf.len()..])
                };

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
}
