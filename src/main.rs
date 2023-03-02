// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use cutr::{options, Command};

fn main() {
    let options = options::Options::parse();

    if let Err(e) = Command::from(&options).run() {
        eprintln!("{e}")
    }
}
