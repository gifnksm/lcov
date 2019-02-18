extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::filter::{FilterMap, LineNum};
use lcov::{Reader, Record, Report};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};

const FIXTURE_DIR: &str = "./tests/fixtures";
const FIXTURE_GLOB: &str = "./tests/fixtures/*.info";

fn open_fixture<P>(file: P) -> Result<Reader<BufReader<File>>, Error>
where
    P: AsRef<Path>,
{
    Ok(Reader::open_file(Path::new(FIXTURE_DIR).join(file))?)
}

fn check_report_same(report1: Report, report2: Report) {
    assert_eq!(report1, report2);
    for (rec1, rec2) in report1.into_records().zip(report2.into_records()) {
        assert_eq!(rec1, rec2);
    }
}

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
            let mut reader = Reader::open_file(entry?)?;
            let mut records = reader.collect::<Result<Vec<_>, _>>()?;

            let mut report1 = Report::from_reader::<_, io::Error>(records.iter().cloned().map(Ok))?;
            let mut report2 =
                Report::from_reader::<_, io::Error>(report1.clone().into_records().map(Ok))?;

            check_report_same(report1, report2);
        }

        Ok(())
    }

    execute().expect("error");
}

#[test]
fn merge_report() {
    fn execute() -> Result<(), Error> {
        for merged_file in &["report_checksum.info", "report.info"] {
            let merged_file = PathBuf::from(merged_file);
            let mut init_file = merged_file.clone();
            init_file.set_extension("init.info");
            let mut run_file = merged_file.clone();
            run_file.set_extension("run.info");

            let merged = open_fixture(merged_file)?;
            let init = open_fixture(init_file)?;
            let run = open_fixture(run_file)?;

            let mut report1 = Report::from_reader(merged)?;

            let mut report2 = Report::new();
            report2.merge(Report::from_reader(init)?)?;
            report2.merge(Report::from_reader(run)?)?;

            check_report_same(report1, report2);
        }

        Ok(())
    }
    execute().expect("error");
}

#[test]
fn line_filter() {
    fn execute() -> Result<(), Error> {
        let mut filter = HashMap::new();
        filter.insert(
            PathBuf::from("/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c"),
            LineNum::from_iter([3..4].iter().cloned()),
        );
        filter.insert(
            PathBuf::from("/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/fizzbuzz.c"),
            LineNum::from_iter([3..7, 14..u32::max_value()].iter().cloned()),
        );
        filter.insert(
            PathBuf::from("/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/main.c"),
            LineNum::from_iter([12..16, 25..31].iter().cloned()),
        );

        let original = open_fixture("report.info")?;
        let mut original_report = Report::from_reader(original)?;
        original_report.sections.filter_map(|(key, mut value)| {
            filter.get(&key.source_file).and_then(|filter| {
                filter.apply(&mut value);
                if value.is_empty() {
                    None
                } else {
                    Some((key, value))
                }
            })
        });

        let filtered = open_fixture("report.filtered.info")?;
        let filtered_report = Report::from_reader(filtered)?;

        check_report_same(original_report, filtered_report);

        Ok(())
    }

    execute().expect("error");
}
