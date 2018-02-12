//! A coverage report.
//!
//! The [`Report`] structure contains coverage information of every file.
//!
//! [`Report`]: struct.Report.html
pub use self::error::{MergeError, ParseError};
use self::parser::Parser;
use self::section::Sections;
use super::{Record, RecordKind};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;

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
/// # extern crate failure;
/// # extern crate lcov;
/// # use failure::Error;
/// use lcov::Report;
///
/// # fn foo() -> Result<(), Error> {
/// let mut report = Report::new();
///
/// // Merges a first file.
/// let reader1 = lcov::open_file("report_a.info")?;
/// report.merge(Report::from_reader(reader1)?)?;
///
/// // Merges a second file.
/// let reader2 = lcov::open_file("report_b.info")?;
/// report.merge(Report::from_reader(reader2)?)?;
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
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let reader = lcov::open_file("report.info")?;
    /// let report = Report::from_reader(reader)?;
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn from_reader<I, E>(iter: I) -> Result<Self, ParseError<E>>
    where
        I: IntoIterator<Item = Result<Record, E>>,
    {
        let mut parser = Parser::new(iter);
        let report = Report {
            sections: section::parse(&mut parser)?,
        };
        Ok(report)
    }

    /// Merges a report into `self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let reader = lcov::open_file("report.info")?;
    /// let mut report = Report::from_reader(reader)?;
    ///
    /// let reader2 = lcov::open_file("report2.info")?;
    /// let report2 = Report::from_reader(reader2)?;
    /// report.merge(report2)?;
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
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let reader = lcov::open_file("report.info")?;
    /// let mut report = Report::from_reader(reader)?;
    ///
    /// let reader2 = lcov::open_file("report2.info")?;
    /// let report2 = Report::from_reader(reader2)?;
    /// report.merge_lossy(report2);
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
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use lcov::Report;
    ///
    /// # fn foo() -> Result<(), Error> {
    /// let reader = lcov::open_file("report.info")?;
    /// let mut report = Report::from_reader(reader)?;
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
    iter: Box<Iterator<Item = Record>>,
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
