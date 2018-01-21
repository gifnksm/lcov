extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::Record;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn is_identical() {
    fn execute() -> Result<(), Error> {
        for entry in glob::glob("./tests/fixtures/*.info")? {
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
