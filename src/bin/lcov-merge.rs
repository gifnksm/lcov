//! Utility commands to operate and analyze LCOV trace file at blazingly fast.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

extern crate failure;
extern crate lcov;

use failure::Error;
use lcov::Report;
use std::{env, process};
use std::path::Path;

fn run<I, P>(it: I) -> Result<(), Error>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut report = Report::new();

    for path in it {
        let reader = lcov::open_file(path)?;
        report.merge(reader)?;
    }

    for rec in report {
        println!("{}", rec);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run(env::args()) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
