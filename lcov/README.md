# lcov

[![Crates.io](https://img.shields.io/crates/v/lcov.svg)](https://crates.io/crates/lcov)
[![Docs.rs](https://docs.rs/lcov/badge.svg)](https://docs.rs/lcov/)
[![Travis CI Build Status](https://travis-ci.org/gifnksm/lcov.svg?branch=master)](https://travis-ci.org/gifnksm/lcov)
![LICENSE](https://img.shields.io/crates/l/lcov.svg)

LCOV tracefile parser/merger/filter in pure Rust.

[LCOV] is a graphical front-end for coverage testing tool [gcov].
It collects gcov data for multiple source files and stores them into the file called as "tracefile".

The purpose of this crate is to operate the LCOV tracefile faster than [the original LCOV Perl
implementation][LCOV GitHub].

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
lcov = "0.1"
```

## Performance

See [the document of `lcov-util`][doc-lcov-util].

## Data structure

In this crate, the data structure corresponding to each line of the LCOV tracefile is called
"LCOV record" and is represented as [`Record`].
Each line of the LCOV tracefile is composed of a string representing a kind of the record,
a colon, a comma-separated field list:

```
<KIND>:<field#0>,<field#1>,...<field#N>
```

LCOV record kind is represented as a variant of [`Record`] or [`RecordKind`].
Each fields of a LCOV record are represented as fields of a struct-like variant of [`Record`].

For details of the LCOV tracefile syntax, see [the manpage of geninfo][geninfo(1)].

## Examples

Parsing a LCOV tracefile:

```rust
use std::fs::File;
use std::io::BufReader;
use lcov::{Reader, Record, RecordKind};

let file = File::open("tests/fixtures/report.info")?;
let buf = BufReader::new(file);

// `Reader` is an iterator that iterates over `Result<lcov::Record, E>` read from the input buffer.
let mut reader = Reader::new(buf);

// Collect the read records into a vector.
let records = reader.collect::<Result<Vec<_>, _>>()?;
assert_eq!(records[0], Record::TestName { name: "".into() });
assert_eq!(records[1].kind(), RecordKind::SourceFile);

// Outputs the read records in LCOV tracefile format.
for record in records {
    println!("{}", record);
}
```

Parsing a LCOV infomation from `String`:

```rust
use lcov::{Reader, Record};

let input = "\
TN:test_name
SF:/path/to/source/file.rs
DA:1,2
DA:3,0
DA:5,6
LF:3
LH:2
end_of_record
";

// `&[u8]` implements `BufRead`, so you can pass it as an argument to `Reader::new`.
let mut reader = Reader::new(input.as_bytes());

let records = reader.collect::<Result<Vec<_>, _>>()?;
assert_eq!(records[0], Record::TestName { name: "test_name".into() });
assert_eq!(records[1], Record::SourceFile { path: "/path/to/source/file.rs".into() });

// Creates an `String` in tracefile format. In this example, it is the same as `input`.
let output = records.into_iter().map(|rec| format!("{}\n", rec)).collect::<String>();
assert_eq!(input, output);
```

Merging tracefiles:

```rust
use std::fs::File;
use std::io::BufReader;
use lcov::{Reader, Record, RecordKind, Report};

// Creates an empty `Report`.
let mut report = Report::new();

// Merges a first file.
let reader1 = Reader::new(BufReader::new(File::open("tests/fixtures/report.init.info")?));
report.merge(reader1);

// Merges a second file.
let reader2 = Reader::new(BufReader::new(File::open("tests/fixtures/report.run.info")?));
report.merge(reader2);

// Outputs the merge result in LCOV tracefile format.
for record in report {
    println!("{}", record);
}
```

[LCOV]: http://ltp.sourceforge.net/coverage/lcov.php
[gcov]: http://gcc.gnu.org/onlinedocs/gcc/Gcov.html
[LCOV GitHub]: https://github.com/linux-test-project/lcov
[geninfo(1)]: http://ltp.sourceforge.net/coverage/lcov/geninfo.1.php
[doc-lcov-util]: https://docs.rs/lcov-util/
[`Record`]: enum.Record.html
[`RecordKind`]: enum.RecordKind.html

## License

This project is licensed under either of

    Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in lcov by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
