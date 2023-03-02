// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::io::{self, Write};

pub mod options;

pub struct Command<'a> {
    options: &'a options::Options,
}

impl<'a> Command<'a> {
    pub fn run(&self) -> Result<()> {
        let stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        let mut stderr = io::stderr().lock();

        csv::ReaderBuilder::new()
            .delimiter(self.options.delimiter.try_into()?)
            .has_headers(false)
            .from_reader(stdin)
            .records()
            .try_for_each(|r| match r {
                Ok(rec) => {
                    let line = self
                        .options
                        .fields
                        .iter()
                        .flat_map(|range| {
                            range.clone().into_iter().filter_map(|idx| rec.get(idx - 1))
                        })
                        .collect::<Vec<&str>>()
                        .join(&self.options.delimiter.to_string());

                    writeln!(stdout, "{line}")
                }
                Err(e) => {
                    writeln!(stderr, "{e}")
                }
            })?;

        Ok(())
    }
}

impl<'a> From<&'a options::Options> for Command<'a> {
    fn from(options: &'a options::Options) -> Self {
        Self { options }
    }
}
