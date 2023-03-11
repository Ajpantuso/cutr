// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bufread_ext::BufReadExt;
use std::io::{self, Write};
use unicode_segmentation::UnicodeSegmentation;

pub mod options;

mod bufread_ext;

pub struct Command<'a> {
    options: &'a options::Options,
}

impl<'a> Command<'a> {
    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();

        let buffer = io::BufReader::new(stdin).unzip_lines();

        let mut lines: Box<dyn Iterator<Item = Result<String>>> = if self.options.fields.is_some() {
            Box::new(buffer.map(|r| {
                r.map(|(line, newline)| self.process_fields(&line).into_owned() + &newline)
            }))
        } else if self.options.bytes.is_some() {
            Box::new(buffer.map(|r| {
                r.map(|(line, newline)| self.process_bytes(&line).into_owned() + &newline)
            }))
        } else if self.options.characters.is_some() {
            Box::new(buffer.map(|r| {
                r.map(|(line, newline)| self.process_chars(&line).into_owned() + &newline)
            }))
        } else {
            Box::new(vec![].into_iter())
        };

        lines.try_for_each(|r| match r {
            Ok(s) => write!(stdout, "{s}"),
            Err(e) => writeln!(stderr, "{e}"),
        })?;

        Ok(())
    }
    fn process_fields(&self, line: &str) -> std::borrow::Cow<'_, str> {
        let words = to_fields(line, self.options.delimiter);

        match &self.options.fields {
            Some(fields) => std::borrow::Cow::from(
                fields
                    .iter()
                    .flat_map(|range| {
                        range
                            .clone()
                            .into_iter()
                            .take(words.len())
                            .filter_map(|idx| words.get(idx - 1))
                            .cloned()
                    })
                    .collect::<Vec<&str>>()
                    .join(&self.options.delimiter.to_string()),
            ),
            None => std::borrow::Cow::from(""),
        }
    }
    fn process_bytes(&self, line: &str) -> std::borrow::Cow<'_, str> {
        match &self.options.bytes {
            Some(bytes) => std::borrow::Cow::from(
                bytes
                    .iter()
                    .filter_map(|range| {
                        let mut indexes = range.clone().into_iter().take(line.len());
                        let first = indexes.next()? - 1;

                        let bytes = line.bytes().collect::<Vec<u8>>();

                        match indexes.last() {
                            //TODO: Properly handle reverse range
                            Some(last) => bytes.get(first..=last - 1),
                            None => bytes.get(first..=first),
                        }
                        .map(|bs| String::from_utf8_lossy(bs).into_owned())
                    })
                    .collect::<String>(),
            ),
            None => std::borrow::Cow::from(""),
        }
    }
    fn process_chars(&self, line: &str) -> std::borrow::Cow<'_, str> {
        match &self.options.characters {
            Some(chars) => std::borrow::Cow::from(
                chars
                    .iter()
                    .filter_map(|range| {
                        let mut indexes = range.clone().into_iter().take(line.len());
                        let first = indexes.next()? - 1;

                        let chars = line
                            .graphemes(true)
                            .map(str::to_string)
                            .collect::<Vec<String>>();

                        match indexes.last() {
                            //TODO: Properly handle reverse range
                            Some(last) => chars.get(first..=last - 1),
                            None => chars.get(first..=first),
                        }
                        .map(|cs| cs.join(""))
                    })
                    .collect::<String>(),
            ),
            None => std::borrow::Cow::from(""),
        }
    }
}

impl<'a> From<&'a options::Options> for Command<'a> {
    fn from(options: &'a options::Options) -> Self {
        Self { options }
    }
}

fn to_fields(s: &str, delimiter: char) -> Vec<&str> {
    s.split(delimiter).collect()
}
