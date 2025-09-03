// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::{Result, Write};

#[derive(Clone, Copy)]
pub enum IndentConfig {
    Tab,
    Space(usize),
}

pub trait IndentWrite: Write {
    fn indent(&mut self) {}
    fn unindent(&mut self) {}

    /// Start a new block.
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn start_block(&mut self) -> Result<()> {
        writeln!(self, "{{")?;
        self.indent();
        Ok(())
    }

    /// Start a new block for continuation on the same line.
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn start_block_no_newline(&mut self) -> Result<()> {
        write!(self, "{{")?;
        self.indent();
        Ok(())
    }

    /// End a block.
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn end_block(&mut self) -> Result<()> {
        self.unindent();
        writeln!(self, "}}")
    }

    /// Start and end a block.
    /// # Errors
    /// Returns an error if writing to the underlying writer fails.
    fn empty_block(&mut self) -> Result<()> {
        writeln!(self, "{{}}")
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

        let expect: &[u8] = b"\
foo
  bar

  bar
    foobar

    foobar
foo
";
        assert_eq!(buffer, expect);

        Ok(())
    }
}
