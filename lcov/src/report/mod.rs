use self::parser::Parser;
use self::section::Section;
use super::{Record, RecordKind};
use std::{fmt, iter, mem};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::PathBuf;

#[macro_use]
mod parser;
pub(crate) mod section;

/// All possible errors that can occur when merging LCOV records.
#[derive(Debug, Clone, Fail, Eq, PartialEq)]
pub enum MergeError<ReadError> {
    /// An error indicating that reading record operation failed.
    ///
    /// This error occurs when the underlying reader returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # fn main() {
    /// use lcov::{Reader, Report, MergeError};
    /// let mut report = Report::new();
    /// assert_matches!(report.merge(Reader::new("FOO:1,2,3".as_bytes())), Err(MergeError::Read(_)));
    /// # }
    /// ```
    #[fail(display = "failed to read record: {}", _0)]
    Read(#[cause] ReadError),

    /// An error indicating that unexpected kind of record is read.
    ///
    /// This error occurs when the LCOV tracefile (or underlying reader) contains invalid record sequence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # fn main() {
    /// use lcov::{Reader, RecordKind, Report, MergeError};
    /// let mut report = Report::new();
    /// let input = "\
    /// TN:test_name
    /// SF:/usr/include/stdio.h
    /// TN:next_test
    /// ";
    /// assert_matches!(report.merge(Reader::new(input.as_bytes())),
    ///                 Err(MergeError::UnexpectedRecord(RecordKind::TestName)));
    /// # }
    /// ```
    #[fail(display = "unexpected record `{}`", _0)]
    UnexpectedRecord(RecordKind),

    /// An error indicating that unexpected "end of file".
    ///
    /// This error occurs when the LCOV tracefile (or underlying reader) contains invalid record sequence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # fn main() {
    /// use lcov::{Reader, RecordKind, Report, MergeError};
    /// let mut report = Report::new();
    /// let input = "\
    /// TN:test_name
    /// SF:/usr/include/stdio.h
    /// ";
    /// assert_matches!(report.merge(Reader::new(input.as_bytes())),
    ///                 Err(MergeError::UnexpectedEof));
    /// # }
    /// ```
    #[fail(display = "unexpected end of file")]
    UnexpectedEof,

    /// An error indicating that the given function line does not match others.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate failure;
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use lcov::{Reader, RecordKind, Report, MergeError};
    /// let mut report = Report::new();
    /// let input1 = "\
    /// TN:
    /// SF:/usr/include/stdio.h
    /// FN:10,main
    /// end_of_record
    /// ";
    /// let input2 = "\
    /// TN:
    /// SF:/usr/include/stdio.h
    /// FN:15,main
    /// end_of_record
    /// ";
    /// report.merge(Reader::new(input1.as_bytes()))?;
    /// assert_matches!(report.merge(Reader::new(input2.as_bytes())),
    ///                 Err(MergeError::UnmatchedFunctionLine));
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # try_main().expect("failed to run test.");
    /// # }
    /// ```
    #[fail(display = "unmatched function line")]
    UnmatchedFunctionLine,

    /// An error indicating that the given function does not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # fn main() {
    /// use lcov::{Reader, RecordKind, Report, MergeError};
    /// let mut report = Report::new();
    /// let input = "\
    /// TN:test_name
    /// SF:/usr/include/stdio.h
    /// FNDA:123,foo
    /// end_of_record
    /// ";
    /// assert_matches!(report.merge(Reader::new(input.as_bytes())),
    ///                 Err(MergeError::UnmatchedFunctionName));
    /// # }
    /// ```
    #[fail(display = "unmatched function name")]
    UnmatchedFunctionName,

    /// An error indicating that the given given does not match.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate failure;
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// # fn try_main() -> Result<(), Error> {
    /// use lcov::{Reader, RecordKind, Report, MergeError};
    /// let mut report = Report::new();
    /// let input1 = "\
    /// TN:
    /// SF:/usr/include/stdio.h
    /// DA:10,5,valid_checksum
    /// end_of_record
    /// ";
    /// let input2 = "\
    /// TN:
    /// SF:/usr/include/stdio.h
    /// DA:10,1,invalid_checksum
    /// end_of_record
    /// ";
    /// report.merge(Reader::new(input1.as_bytes()))?;
    /// assert_matches!(report.merge(Reader::new(input2.as_bytes())),
    ///                 Err(MergeError::UnmatchedChecksum));
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # try_main().expect("failed to run test.");
    /// # }
    /// ```
    #[fail(display = "unmatches checksum")]
    UnmatchedChecksum,
}

/// An accumulated coverage information from some LCOV tracefiles.
///
/// `Report` is used for merging/filtering the coverage information.
///
/// # Examples
///
/// Merges LCOV tracefiles and outputs the result in LCOV tracefile format:
///
/// ```rust
/// # extern crate failure;
/// # extern crate lcov;
/// # use failure::Error;
/// use lcov::{Report, Reader};
/// use std::fs::File;
/// use std::io::BufReader;
///
/// # fn foo() -> Result<(), Error> {
/// let mut report = Report::new();
///
/// // Merges a first file.
/// let reader1 = Reader::new(BufReader::new(File::open("report_a.info")?));
/// report.merge(reader1)?;
///
/// // Merges a second file.
/// let reader2 = Reader::new(BufReader::new(File::open("report_b.info")?));
/// report.merge(reader2)?;
///
// Outputs the merge result in LCOV tracefile format.
/// for record in report {
///     println!("{}", record);
/// }
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
///
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Report {
    sections: BTreeMap<SectionKey, Section>,
}

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct SectionKey {
    pub(crate) test_name: String,
    pub(crate) source_file: PathBuf,
}

impl Report {
    /// Creates an empty report.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Report;
    /// let report = Report::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges LCOV tracefile into the report.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use lcov::{Report, Reader};
    /// use std::fs::File;
    /// use std::io::BufReader;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let mut report = Report::new();
    /// let reader = Reader::new(BufReader::new(File::open("report.info")?));
    /// report.merge(reader)?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn merge<I, E>(&mut self, it: I) -> Result<(), MergeError<E>>
    where
        I: IntoIterator<Item = Result<Record, E>>,
    {
        let mut parser = Parser::new(it.into_iter());

        while let Some(_) = parser.peek().map_err(MergeError::Read)? {
            let test_name =
                eat_if_matches!(parser, Record::TestName { name } => name).unwrap_or_else(String::new);
            let source_file = eat!(parser, Record::SourceFile { path } => path);
            let key = SectionKey {
                test_name,
                source_file,
            };
            match self.sections.entry(key) {
                Entry::Vacant(e) => {
                    let mut section = Section::default();
                    section.merge(&mut parser)?;
                    // If the new section contains no data, ignore it.
                    // LCOV merge (`lcov -c -a XXX`) behaves the same way.
                    if !section.is_empty() {
                        let _ = e.insert(section);
                    }
                }
                Entry::Occupied(mut e) => e.get_mut().merge(&mut parser)?,
            }
            eat!(parser, Record::EndOfRecord);
        }

        Ok(())
    }

    pub(crate) fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut((SectionKey, Section)) -> Option<(SectionKey, Section)>,
    {
        let sections = mem::replace(&mut self.sections, BTreeMap::new());
        self.sections.extend(sections.into_iter().filter_map(f));
    }
}

impl IntoIterator for Report {
    type Item = Record;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.sections.into_iter().flat_map(|(key, section)| {
            iter::once(Record::TestName {
                name: key.test_name,
            }).chain(iter::once(Record::SourceFile {
                path: key.source_file,
            }))
                .chain(section.into_iter())
                .chain(iter::once(Record::EndOfRecord))
        });
        IntoIter {
            inner: Box::new(iter),
        }
    }
}

pub struct IntoIter {
    inner: Box<Iterator<Item = Record>>,
}

impl fmt::Debug for IntoIter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "IntoIter {{ .. }}")
    }
}

impl Iterator for IntoIter {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
