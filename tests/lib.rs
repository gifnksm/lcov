extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::{LineFilter, Reader, Record, Report};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

const FIXTURE_DIR: &str = "./tests/fixtures";
const FIXTURE_GLOB: &str = "./tests/fixtures/*.info";

fn open<P>(path: P) -> Result<Reader<BufReader<File>>, Error>
where
    P: AsRef<Path>,
{
    let file = File::open(path.as_ref())?;
    let reader = Reader::new(BufReader::new(file));
    Ok(reader)
}

fn open_fixture<P>(file: P) -> Result<Reader<BufReader<File>>, Error>
where
    P: AsRef<Path>,
{
    let fixture_dir = Path::new(FIXTURE_DIR);
    open(fixture_dir.join(file))
}

fn check_report_same(report1: Report, report2: Report) {
    assert_eq!(report1, report2);
    for (rec1, rec2) in report1.into_iter().zip(report2) {
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
            let mut reader = open(entry?)?;
            let mut records = reader.collect::<Result<Vec<_>, _>>()?;

            let mut report1 = Report::new();
            report1.merge::<_, io::Error>(records.iter().cloned().map(Ok))?;

            let mut report2 = Report::new();
            report2.merge::<_, io::Error>(report1.clone().into_iter().map(Ok))?;

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

            let mut report1 = Report::new();
            report1.merge(merged)?;

            let mut report2 = Report::new();
            report2.merge(init)?;
            report2.merge(run)?;

            check_report_same(report1, report2);
        }

        Ok(())
    }
    execute().expect("error");
}

#[test]
fn line_filter() {
    fn execute() -> Result<(), Error> {
        let mut filter = LineFilter::new();
        filter.insert(
            "/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/div.c",
            [(3, 3)].iter().cloned(),
        );
        filter.insert(
            "/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/fizzbuzz.c",
            [(3, 6), (14, u32::max_value())].iter().cloned(),
        );
        filter.insert(
            "/home/nksm/rhq/github.com/gifnksm/lcov/tests/fixtures/src/main.c",
            [(12, 15), (25, 30)].iter().cloned(),
        );

        let original = open_fixture("report.info")?;
        let mut original_report = Report::new();
        original_report.merge(original)?;
        filter.apply(&mut original_report);

        let filtered = open_fixture("report.filtered.info")?;
        let mut filtered_report = Report::new();
        filtered_report.merge(filtered)?;

        check_report_same(original_report, filtered_report);

        Ok(())
    }

    execute().expect("error");
}
