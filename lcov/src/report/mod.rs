//! A coverage report.
//!
//! The [`Report`] structure contains coverage information of every file.
//!
//! [`Report`]: struct.Report.html
pub use self::error::{MergeError, ParseError};
use self::parser::Parser;
use self::section::Sections;
use super::reader::Error as ReadError;
use super::{Reader, Record, RecordKind};
use failure::Error;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

#[macro_use]
mod parser;
mod error;
pub mod section;

/// An accumulated coverage information from some LCOV tracefiles.
///
/// `Report` is used for merging/filtering the coverage information.
///
/// # Examples
///
/// Merges LCOV tracefiles and outputs the result in LCOV tracefile format:
///
/// ```rust
/// # use failure::Error;
/// use lcov::Report;
///
/// # fn foo() -> Result<(), Error> {
/// let mut report = Report::new();
///
/// // Merges a first file.
/// report.merge(Report::from_file("report_a.info")?)?;
///
/// // Merges a second file.
/// report.merge(Report::from_file("report_b.info")?)?;
///
/// // Outputs the merge result in LCOV tracefile format.
/// for record in report.into_records() {
///     println!("{}", record);
/// }
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
///
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Report {
    /// Coverage information about every source files.
    pub sections: Sections,
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

    /// Creates a report from LCOV record reader.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use failure::Error;
    /// use lcov::{Report, Reader};
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let input = "\
    /// TN:test_name
    /// SF:/path/to/source/file.rs
    /// DA:1,2
    /// DA:3,0
    /// DA:5,6
    /// LF:3
    /// LH:2
    /// end_of_record
    /// ";
    /// let reader = Reader::new(input.as_bytes());
    /// let report = Report::from_reader(reader)?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn from_reader<I, E>(iter: I) -> Result<Self, ParseError>
    where
        I: IntoIterator<Item = Result<Record, E>>,
        E: Into<Error>,
    {
        let mut parser = Parser::new(iter.into_iter().map(|item| item.map_err(Into::into)));
        let report = Report {
            sections: section::parse(&mut parser)?,
        };
        Ok(report)
    }

    /// Creates a report from LCOV tracefile.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let report = Report::from_file("report.info")?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn from_file<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        let reader = Reader::open_file(path)
            .map_err(Into::into)
            .map_err(ReadError::Io)
            .map_err(Into::into)
            .map_err(ParseError::Read)?;
        Self::from_reader(reader)
    }

    /// Merges a report into `self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let mut report = Report::from_file("report1.info")?;
    /// report.merge(Report::from_file("report2.info")?)?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn merge(&mut self, other: Self) -> Result<(), MergeError> {
        self.sections.merge(other.sections)
    }

    /// Merges a report into `self` with ignoring an Errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let mut report = Report::from_file("report1.info")?;
    /// report.merge_lossy(Report::from_file("report2.info")?);
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn merge_lossy(&mut self, other: Self) {
        self.sections.merge_lossy(other.sections)
    }

    /// Creates an iterator which iterates over [LCOV section].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let mut report = Report::from_file("report.info")?;
    /// // ... Manipulate report
    /// for record in report.into_records() {
    ///    println!("{}", record);
    /// }
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    ///
    /// [LCOV records]: enum.Record.html
    pub fn into_records(self) -> IntoRecords {
        IntoRecords {
            iter: section::into_records(self.sections),
        }
    }
}

/// An iterator which iterates [LCOV records].
///
/// This `struct` is created by the [`into_records`] methods on [`Report`].
/// See its documentation for more.
///
/// [`LCOV records`]: ../struct.Record.html
/// [`into_records`]: struct.Report.html#method.into_records
/// [`Report`]: struct.Report.html
pub struct IntoRecords {
    iter: Box<dyn Iterator<Item = Record>>,
}

impl fmt::Debug for IntoRecords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "IntoIter {{ .. }}")
    }
}

impl Iterator for IntoRecords {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

trait Merge {
    fn merge(&mut self, other: Self) -> Result<(), MergeError>;
    fn merge_lossy(&mut self, other: Self);
}

impl<K, V> Merge for BTreeMap<K, V>
where
    K: Ord,
    V: Merge,
{
    fn merge(&mut self, other: Self) -> Result<(), MergeError> {
        for (key, value) in other {
            match self.entry(key) {
                Entry::Vacant(e) => {
                    let _ = e.insert(value);
                }
                Entry::Occupied(mut e) => e.get_mut().merge(value)?,
            }
        }
        Ok(())
    }

    fn merge_lossy(&mut self, other: Self) {
        for (key, value) in other {
            match self.entry(key) {
                Entry::Vacant(e) => {
                    let _ = e.insert(value);
                }
                Entry::Occupied(mut e) => e.get_mut().merge_lossy(value),
            }
        }
    }
}
