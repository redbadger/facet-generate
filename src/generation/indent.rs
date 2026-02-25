use std::io::{Result, Write};

#[derive(Clone, Copy)]
pub enum IndentConfig {
    Tab,
    Space(usize),
}

pub trait IndentWrite: Write {
    fn indent(&mut self) {}
    fn unindent(&mut self) {}

    /// Start a new block with RAII-style automatic closing.
    ///
    /// Writes `{\n`, indents, and returns a [`Block`] guard that implements
    /// [`IndentWrite`]. When the guard is dropped it unindents and writes `}\n`.
    ///
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn blockln(&mut self) -> Result<Block<'_>>
    where
        Self: Sized,
    {
        writeln!(self, "{{")?;
        self.indent();
        Ok(Block { writer: self })
    }

    /// Start a new block with RAII-style automatic closing.
    ///
    /// Indents, and returns a [`Block`] guard that implements
    /// [`IndentWrite`]. When the guard is dropped it unindents and writes `}\n`.
    ///
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn block(&mut self) -> Result<Block<'_>>
    where
        Self: Sized,
    {
        write!(self, "{{")?;
        self.indent();
        Ok(Block { writer: self })
    }
}

/// RAII guard returned by [`IndentWrite::block`].
///
/// Implements [`Write`] and [`IndentWrite`] by delegating to the wrapped
/// writer. On [`Drop`], it unindents and writes the closing `}\n`.
pub struct Block<'a> {
    writer: &'a mut dyn IndentWrite,
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
        let _ = writeln!(self.writer, "}}");
    }
}

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
    fn block() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut w = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        write!(w, "fn foo() ")?;
        {
            let mut w = w.blockln()?;
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
    fn block_no_newline() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut w = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        write!(w, "fn foo() ")?;
        {
            let mut w = w.block()?;
            write!(w, r#"let _ = "hello";"#)?;
        }

        insta::assert_snapshot!(String::from_utf8(buffer).unwrap(), @r#"fn foo() {let _ = "hello";}"#);

        Ok(())
    }
}
