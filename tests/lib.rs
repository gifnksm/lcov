extern crate failure;
extern crate glob;
extern crate lcov;

use failure::Error;
use lcov::{Merger, Reader, Record};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;

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
fn is_identical_merger() {
    fn normalize<I>(records: I) -> Vec<Record>
    where
        I: IntoIterator<Item = Record>,
    {
        fn read_file<I>(records: &mut I) -> Option<(String, PathBuf, Vec<Record>)>
        where
            I: Iterator<Item = Record>,
        {
            let mut test_name = String::new();
            let mut source_file = PathBuf::new();

            let mut result = vec![];
            let mut buf: Vec<Record> = vec![];
            for rec in records {
                match rec {
                    Record::TestName { name } => {
                        test_name = name;
                        continue;
                    }
                    Record::SourceFile { path } => {
                        source_file = path;
                        continue;
                    }
                    Record::EndOfRecord => break,
                    _ => {}
                }

                if buf.is_empty() || buf[0].kind() == rec.kind() {
                    buf.push(rec);
                } else {
                    buf.sort();
                    result.extend(buf);
                    buf = vec![rec];
                }
            }

            if buf.is_empty() {
                return None;
            }

            buf.sort();
            result.extend(buf);
            Some((test_name, source_file, result))
        }

        let mut records = records.into_iter();
        let mut map = BTreeMap::new();
        while let Some((test_name, source_file, recs)) = read_file(&mut records) {
            let _ = map.insert((test_name, source_file), recs);
        }

        let mut result = vec![];
        for ((test_name, source_file), recs) in map {
            result.push(Record::TestName { name: test_name });
            result.push(Record::SourceFile { path: source_file });
            result.extend(recs);
            result.push(Record::EndOfRecord);
        }
        result
    }

    fn execute() -> Result<(), Error> {
        for entry in glob::glob(FIXTURE_GLOB)? {
            eprintln!("{:?}", entry);
            let mut file = File::open(entry?)?;
            let mut reader = Reader::new(BufReader::new(file));
            let mut records = reader.collect::<Result<Vec<_>, _>>()?;

            let mut merger = Merger::new();
            merger.merge::<_, io::Error>(records.iter().cloned().map(Ok))?;

            let records = normalize(records);
            for (r1, r2) in merger.into_iter().zip(records) {
                eprintln!("{}\t{}", r1, r2);
                assert_eq!(r1, r2);
            }
        }

        Ok(())
    }

    execute().expect("error");
}
