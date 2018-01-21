extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::{Reader, Record};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};

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
    assert!(execute().is_ok());
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
    assert!(execute().is_ok());
}
