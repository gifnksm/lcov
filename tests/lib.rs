extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::{Reader, Record, Report};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

const FIXTURE_DIR: &str = "./tests/fixtures";
const FIXTURE_GLOB: &str = "./tests/fixtures/*.info";

#[test]
fn is_identical_parse() {
    fn execute() -> Result<(), Error> {
        for entry in glob::glob(FIXTURE_GLOB)? {
            let file = File::open(entry?)?;
            let mut reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let rec = line.parse::<Record>()?;
                assert_eq!(line, rec.to_string());
            }
        }
        Ok(())
    }

    execute().expect("error");
}

#[test]
fn is_identical_reader() {
    fn execute() -> Result<(), Error> {
        for entry in glob::glob(FIXTURE_GLOB)? {
            let mut file = File::open(entry?)?;
            let mut input = String::new();
            file.read_to_string(&mut input)?;
            let mut reader = Reader::new(BufReader::new(input.as_bytes()));

            let mut output = vec![];
            for rec in reader {
                writeln!(output, "{}", rec?)?;
            }
            assert_eq!(input.as_bytes(), output.as_slice());
        }
        Ok(())
    }

    execute().expect("error");
}

#[test]
fn is_identical_report() {
    fn execute() -> Result<(), Error> {
        for entry in glob::glob(FIXTURE_GLOB)? {
            eprintln!("{:?}", entry);
            let mut file = File::open(entry?)?;
            let mut reader = Reader::new(BufReader::new(file));
            let mut records = reader.collect::<Result<Vec<_>, _>>()?;

            let mut report = Report::new();
            report.merge::<_, io::Error>(records.iter().cloned().map(Ok))?;

            let mut report2 = Report::new();
            report2.merge::<_, io::Error>(report.clone().into_iter().map(Ok))?;

            for (r1, r2) in report.into_iter().zip(report2) {
                eprintln!("{}\t{}", r1, r2);
                assert_eq!(r1, r2);
            }
        }

        Ok(())
    }

    execute().expect("error");
}

#[test]
fn merge_report() {
    fn execute() -> Result<(), Error> {
        let fixture_dir = Path::new(FIXTURE_DIR);
        for merged_file in &["report_checksum.info", "report.info"] {
            let mut merged_path = fixture_dir.join(merged_file);
            let mut init_path = merged_path.clone();
            init_path.set_extension("init.info");
            let mut run_path = merged_path.clone();
            run_path.set_extension("run.info");

            let merged = Reader::new(BufReader::new(File::open(merged_path)?));
            let init = Reader::new(BufReader::new(File::open(init_path)?));
            let run = Reader::new(BufReader::new(File::open(run_path)?));

            let mut report1 = Report::new();
            report1.merge(merged)?;

            let mut report2 = Report::new();
            report2.merge(init)?;
            report2.merge(run)?;

            for (r1, r2) in report1.clone().into_iter().zip(report2.clone()) {
                assert_eq!(r1, r2);
            }
            assert_eq!(report1, report2);
        }

        Ok(())
    }
    execute().expect("error");
}
