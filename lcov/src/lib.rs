//! LCOV tracefile parser/merger/filter in pure Rust.
//!
//! [LCOV] is a graphical front-end for coverage testing tool [gcov].
//! It collects gcov data for multiple source files and stores them into the file called as "tracefile".
//!
//! The purpose of this crate is to operate the LCOV tracefile faster than [the original LCOV Perl
//! implementation][LCOV GitHub].
//!
//! # Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! lcov = "0.8.1"
//! ```
//!
//! # Performance
//!
//! See [the README of `lcov-util`][readme-lcov-util].
//!
//! # Data structure
//!
//! In this crate, the data structure corresponding to each line of the LCOV tracefile is called
//! "LCOV record" and is represented as [`Record`].
//! Each line of the LCOV tracefile is composed of a string representing a kind of the record,
//! a colon, a comma-separated field list:
//!
//! ```lcov
//! <KIND>:<field#0>,<field#1>,...<field#N>
//! ```
//!
//! LCOV record kind is represented as a variant of [`Record`] or [`RecordKind`].
//! Each fields of an LCOV record are represented as fields of a struct-like variant of [`Record`].
//!
//! For details of the LCOV tracefile syntax, see [the manpage of geninfo][geninfo(1)].
//!
//! # Examples
//!
//! Parsing an LCOV tracefile:
//!
//! ```rust
//! # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
//! use lcov::{Record, RecordKind, Reader};
//!
//! // `Reader` is an iterator that iterates over `Result<lcov::Record, E>` read from the input buffer.
//! let mut reader = Reader::open_file("tests/fixtures/report.info")?;
//!
//! // Collect the read records into a vector.
//! let records = reader.collect::<Result<Vec<_>, _>>()?;
//! assert_eq!(records[0], Record::TestName { name: "".into() });
//! assert_eq!(records[1].kind(), RecordKind::SourceFile);
//!
//! // Outputs the read records in LCOV tracefile format.
//! for record in records {
//!     println!("{}", record);
//! }
//! # Ok(())
//! # }
//! # fn main() {
//! #   try_main().expect("failed to run test");
//! # }
//! ```
//!
//! Creating an LCOV report from `String`:
//!
//! ```rust
//! # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
//! use lcov::{Reader, Record};
//!
//! let input = "\
//! TN:test_name
//! SF:/path/to/source/file.rs
//! DA:1,2
//! DA:3,0
//! DA:5,6
//! LF:3
//! LH:2
//! end_of_record
//! ";
//!
//! // `&[u8]` implements `BufRead`, so you can pass it as an argument to `Reader::new`.
//! let mut reader = Reader::new(input.as_bytes());
//!
//! let records = reader.collect::<Result<Vec<_>, _>>()?;
//! assert_eq!(records[0], Record::TestName { name: "test_name".into() });
//! assert_eq!(records[1], Record::SourceFile { path: "/path/to/source/file.rs".into() });
//!
//! // Creates an `String` in tracefile format. In this example, it is the same as `input`.
//! let output = records.into_iter().map(|rec| format!("{}\n", rec)).collect::<String>();
//! assert_eq!(input, output);
//! # Ok(())
//! # }
//! # fn main() {
//! #   try_main().expect("failed to run test");
//! # }
//! ```
//!
//! Merging tracefiles:
//!
//! ```rust
//! # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
//! use lcov::{Record, RecordKind, Report};
//!
//! // Creates an empty `Report`.
//! let mut report = Report::new();
//!
//! // Merges a first file.
//! report.merge(Report::from_file("tests/fixtures/report.init.info")?)?;
//!
//! // Merges a second file.
//! report.merge(Report::from_file("tests/fixtures/report.run.info")?)?;
//!
//! // Outputs the merge result in LCOV tracefile format.
//! for record in report.into_records() {
//!     println!("{}", record);
//! }
//! # Ok(())
//! # }
//! # fn main() {
//! #   try_main().expect("failed to run test");
//! # }
//! ```
//!
//! [LCOV]: http://ltp.sourceforge.net/coverage/lcov.php
//! [gcov]: http://gcc.gnu.org/onlinedocs/gcc/Gcov.html
//! [LCOV GitHub]: https://github.com/linux-test-project/lcov
//! [geninfo(1)]: http://ltp.sourceforge.net/coverage/lcov/geninfo.1.php
//! [readme-lcov-util]: https://github.com/gifnksm/lcov/README.md
//! [`Record`]: enum.Record.html
//! [`RecordKind`]: enum.RecordKind.html

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![doc(html_root_url = "https://docs.rs/lcov/0.8.1")]

pub use reader::Reader;
pub use record::{Record, RecordKind};
pub use report::Report;

pub mod filter;
pub mod reader;
pub mod record;
pub mod report;
