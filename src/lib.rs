// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use field_reader::BufReadFieldExt;
use std::io::{self, BufRead, Write};

pub mod options;

mod field_reader;

pub struct Command<'a> {
    options: &'a options::Options,
}

impl<'a> Command<'a> {
    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();

        if let Some(fields) = &self.options.fields {
            io::BufReader::new(stdin)
                .to_fields(self.options.delimiter)
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
                })
                .try_for_each(|r| match r {
                    Ok(s) => write!(stdout, "{s}"),
                    Err(e) => writeln!(stderr, "{e}"),
                })?;
        } else if let Some(chars) = &self.options.characters {
            io::BufReader::new(stdin)
                .lines()
                .map(|r| {
                    r.map(|s| {
                        chars
                            .iter()
                            .flat_map(|range| {
                                range
                                    .clone()
                                    .into_iter()
                                    .take(s.len())
                                    .filter_map(|idx| s.chars().nth(idx - 1))
                            })
                            .collect::<String>()
                    })
                    .map_err(|e| anyhow::anyhow!(e))
                })
                .try_for_each(|r| match r {
                    Ok(s) => writeln!(stdout, "{s}"),
                    Err(e) => writeln!(stderr, "{e}"),
                })?;
        };

        Ok(())
    }
}

impl<'a> From<&'a options::Options> for Command<'a> {
    fn from(options: &'a options::Options) -> Self {
        Self { options }
    }
}
