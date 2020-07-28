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

use structopt;

use lcov::Report;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "Merges LCOV tracefiles")]
struct Opt {
    /// Disables varidation such as checksum checking
    #[structopt(long = "loose")]
    loose: bool,

    /// LCOV tracefiles to merge
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    let mut merged_report = Report::new();

    for path in &opt.files {
        let report = Report::from_file(path)?;
        if opt.loose {
            merged_report.merge_lossy(report);
        } else {
            merged_report.merge(report)?;
        }
    }

    for rec in merged_report.into_records() {
        println!("{}", rec);
    }

    Ok(())
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
