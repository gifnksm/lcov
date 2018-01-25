extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::{Reader, Record, Report};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};

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
