// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::io::BufRead;

pub trait BufReadFieldExt<B: BufRead> {
    fn to_fields(self, delimiter: char) -> FieldsIterator<B>;
}

impl<B: BufRead> BufReadFieldExt<B> for B {
    fn to_fields(self, delimiter: char) -> FieldsIterator<B> {
        FieldsIterator::new(self, delimiter)
    }
}

pub struct FieldsIterator<B: BufRead> {
    delimiter: char,
    inner: B,
}

impl<B: BufRead> FieldsIterator<B> {
    fn new(inner: B, delimiter: char) -> Self {
        Self { delimiter, inner }
    }
}

impl<B: BufRead> Iterator for FieldsIterator<B> {
    type Item = Result<(Vec<String>, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();

        match self.inner.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_) => {
                let (line, newline) = split_at_newline(&buf);

                Some(Ok((
                    line.split(self.delimiter).map(str::to_string).collect(),
                    String::from(newline),
                )))
            }
            Err(e) => Some(Err(anyhow::anyhow!(e))),
        }
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
    use super::BufReadFieldExt;
    use std::io::Cursor;

    #[test]
    fn test_to_fields() {
        let cursor = Cursor::new("Hello, World!\n");

        assert_eq!(
            (
                vec![String::from("Hello"), String::from(" World!")],
                String::from("\n")
            ),
            cursor.to_fields(',').next().unwrap().unwrap()
        );

        let cursor = Cursor::new("Hello, World!\r\n");

        assert_eq!(
            (
                vec![String::from("Hello"), String::from(" World!")],
                String::from("\r\n")
            ),
            cursor.to_fields(',').next().unwrap().unwrap()
        )
    }

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
