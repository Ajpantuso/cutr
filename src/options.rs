// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use std::str::FromStr;

#[derive(Debug, Parser)]
pub struct Options {
    #[arg(short = 'd', long = "delimiter")]
    #[arg(default_value = "\t")]
    pub delimiter: char,
    #[arg(short = 'f', long = "fields")]
    pub fields: Vec<Range>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl FromStr for Range {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('-') {
            Some((start, "")) => Ok(Self {
                start: start.parse()?,
                ..Default::default()
            }),
            Some(("", end)) => Ok(Self {
                end: end.parse()?,
                ..Default::default()
            }),
            Some((start, end)) => Ok(Self {
                start: start.parse()?,
                end: end.parse()?,
            }),
            None => Ok(Self {
                start: s.parse()?,
                end: s.parse()?,
            }),
        }
    }
}

impl IntoIterator for Range {
    type Item = usize;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self { start: 0, end: 0 } => Box::new(Vec::new().into_iter()),
            Self { start: 0, end } => Box::new(1..=end),
            Self { start, end: 0 } => Box::new(start..),
            Self { start, end } if start > end => Box::new((end..=start).rev()),
            Self { start, end } => Box::new(start..=end),
        }
    }
}

#[cfg(test)]
mod range_tests {
    use super::Range;
    use anyhow::Result;
    use std::str::FromStr;
    use test_case::test_case;

    #[test_case("1", Ok(Range::new(1, 1)) ; "1")]
    #[test_case("1-3", Ok(Range::new(1, 3)) ; "1-3")]
    #[test_case("3-1", Ok(Range::new(3, 1)) ; "3-1")]
    #[test_case("-3", Ok(Range::new(0, 3)) ; "0-3")]
    #[test_case("1-", Ok(Range::new(1, 0)) ; "1-")]
    fn from_str(s: &str, expected: Result<Range>) {
        let res = Range::from_str(s);

        match expected {
            Ok(r) => assert_eq!(r, res.unwrap()),
            Err(_) => assert!(res.is_err()),
        }
    }

    #[test_case(Range::new(1,1), vec![1] ; "1")]
    #[test_case(Range::new(1,0), vec![1] ; "1-")]
    #[test_case(Range::new(1,3), vec![1,2,3] ; "1-3")]
    #[test_case(Range::new(3,1), vec![3,2,1] ; "3-1")]
    #[test_case(Range::new(0,3), vec![1,2,3] ; "-3")]
    fn into_iter(range: Range, expected: Vec<usize>) {
        assert_eq!(
            expected,
            range
                .into_iter()
                .take(expected.len())
                .collect::<Vec<usize>>()
        );
    }
}
