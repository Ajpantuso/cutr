// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::io::BufRead;

pub trait BufReadExt<B: BufRead> {
    fn unzip_lines(self) -> UnzipNewline<B>;
}

impl<B: BufRead> BufReadExt<B> for B {
    fn unzip_lines(self) -> UnzipNewline<B> {
        UnzipNewline::new(self)
    }
}

pub struct UnzipNewline<B: BufRead> {
    buf: String,
    inner: B,
}

impl<B: BufRead> UnzipNewline<B> {
    fn new(inner: B) -> Self {
        Self {
            buf: String::new(),
            inner,
        }
    }
}

impl<B: BufRead> Iterator for UnzipNewline<B> {
    type Item = Result<(String, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = match self.inner.read_line(&mut self.buf) {
            Ok(0) => None,
            Ok(_) => {
                let (line, newline) = split_at_newline(&self.buf);

                Some(Ok((line.to_owned(), newline.to_owned())))
            }
            Err(e) => Some(Err(anyhow::anyhow!(e))),
        };

        self.buf.clear();

        res
    }
}

fn split_at_newline(s: &str) -> (&str, &str) {
    match s.rfind('\n') {
        Some(i) if i > 0 && &s[i - 1..=i] == "\r\n" => s.split_at(i - 1),
        Some(i) => s.split_at(i),
        None => (s, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::split_at_newline;

    #[test]
    fn test_split_at_newline() {
        assert_eq!(("Hello, World!", "\n"), split_at_newline("Hello, World!\n"));
        assert_eq!(
            ("Hello, World!", "\r\n"),
            split_at_newline("Hello, World!\r\n")
        );
        assert_eq!(("Hello, ", "\nWorld!"), split_at_newline("Hello, \nWorld!"));
    }
}
