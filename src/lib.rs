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

        let mut lines: Box<dyn Iterator<Item = Result<String>>> =
            if let Some(fields) = &self.options.fields {
                Box::new(
                    buffer
                        .map(|res| {
                            res.map(|(line, newline)| {
                                (to_fields(&line, self.options.delimiter), newline)
                            })
                        })
                        .map(|r| {
                            r.map(|(words, newline)| {
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
                                    .collect::<Vec<String>>()
                                    .join(&self.options.delimiter.to_string())
                                    + &newline
                            })
                            .map_err(|e| anyhow::anyhow!(e))
                        }),
                )
            } else if let Some(bytes) = &self.options.bytes {
                Box::new(buffer.map(|r| {
                    r.map(|(line, newline)| {
                        bytes
                            .iter()
                            .filter_map(|range| {
                                let mut indexes = range.clone().into_iter().take(line.len());
                                let first = indexes.next()? - 1;

                                let bytes = line.bytes().collect::<Vec<u8>>();

                                match indexes.last() {
                                    Some(last) => bytes.get(first..=last - 1),
                                    None => bytes.get(first..=first),
                                }
                                .map(|bs| String::from_utf8_lossy(bs).into_owned())
                            })
                            .collect::<String>()
                            + &newline
                    })
                }))
            } else if let Some(chars) = &self.options.characters {
                Box::new(buffer.map(|r| {
                    r.map(|(line, newline)| {
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
                                    Some(last) => chars.get(first..=last - 1),
                                    None => chars.get(first..=first),
                                }
                                .map(|cs| cs.join(""))
                            })
                            .collect::<String>()
                            + &newline
                    })
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
}

impl<'a> From<&'a options::Options> for Command<'a> {
    fn from(options: &'a options::Options) -> Self {
        Self { options }
    }
}

fn to_fields(s: &str, delimiter: char) -> Vec<String> {
    s.split(delimiter).map(str::to_string).collect()
}
