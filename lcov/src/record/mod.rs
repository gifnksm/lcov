//! An LCOV record.
//!
//! The [`Record`] structure represents all kinds of LCOV records.
//!
//! [`Record`]: enum.Record.html
pub use self::parse::*;
use std::path::PathBuf;

mod display;
mod parse;
#[cfg(test)]
mod tests;

/// Represents all kinds of LCOV records.
///
/// This `struct` can be created by parsing an LCOV record string by [`parse`] method (provided by the `FromStr` trait).
/// This `struct` can be converted into an LCOV record string by [`to_string`] method (provided by the `ToString` trait).
///
/// See those documentation for more.
///
/// [`parse`]: enum.Record.html#method.parse
/// [`to_string`]: enum.Record.html#method.to_string
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Record {
    /// Represents a `TN` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("TN:test_name".parse(), Ok(Record::TestName { name: "test_name".into() }));
    /// ```
    TestName {
        /// test name
        name: String,
    },
    /// Represents a `SF` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("SF:/usr/include/stdio.h".parse(),
    ///            Ok(Record::SourceFile { path: "/usr/include/stdio.h".into() }));
    /// ```
    SourceFile {
        /// Absolute path to the source file.
        path: PathBuf,
    },

    /// Represents a `FN` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("FN:10,main".parse(),
    ///            Ok(Record::FunctionName { name: "main".into(), start_line: 10 }));
    /// ```
    FunctionName {
        /// Function name.
        name: String,
        /// Line number of function start.
        start_line: u32,
    },
    /// Represents a `FNDA` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("FNDA:1,main".parse(),
    ///            Ok(Record::FunctionData { name: "main".into(), count: 1 }));
    /// ```
    FunctionData {
        /// Function name.
        name: String,
        /// Execution count.
        count: u64,
    },
    /// Represents a `FNF` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("FNF:10".parse(), Ok(Record::FunctionsFound { found: 10 }));
    /// ```
    FunctionsFound {
        /// Number of functions found.
        found: u32,
    },
    /// Represents a `FNH` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("FNH:7".parse(), Ok(Record::FunctionsHit { hit: 7 }));
    /// ```
    FunctionsHit {
        /// Number of functions hit.
        hit: u32,
    },

    /// Represents a `BRDA` record.
    ///
    /// `block` and `branch` are gcc internal IDs for the branch.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("BRDA:10,30,40,-".parse(),
    ///            Ok(Record::BranchData { line: 10, block: 30, branch: 40, taken: None }));
    /// assert_eq!("BRDA:10,30,40,3".parse(),
    ///            Ok(Record::BranchData { line: 10, block: 30, branch: 40, taken: Some(3) }));
    /// ```
    BranchData {
        /// Line number.
        line: u32,
        /// Block number.
        block: u32,
        /// Branch number.
        branch: u32,
        /// A number indicating how often that branch was taken.
        taken: Option<u64>,
    },
    /// Represents a `BRF` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("BRF:40".parse(), Ok(Record::BranchesFound { found: 40 }));
    /// ```
    BranchesFound {
        /// Number of branches found.
        found: u32,
    },
    /// Represents a `BRH` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("BRH:20".parse(), Ok(Record::BranchesHit { hit: 20 }));
    /// ```
    BranchesHit {
        /// Number of branches hit.
        hit: u32,
    },

    /// Represents a `DA` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("DA:8,30".parse(), Ok(Record::LineData { line: 8, count: 30, checksum: None }));
    /// assert_eq!("DA:8,30,asdfasdf".parse(),
    ///            Ok(Record::LineData { line: 8, count: 30, checksum: Some("asdfasdf".into()) }));
    /// ```
    LineData {
        /// Line number.
        line: u32,
        /// Execution count.
        count: u64,
        /// Checksum for each instrumented line.
        checksum: Option<String>,
    },
    /// Represents a `LF` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("LF:123".parse(), Ok(Record::LinesFound { found: 123 }));
    /// ```
    LinesFound {
        /// Number of instrumented line.
        found: u32,
    },
    /// Represents a `LH` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("LH:45".parse(), Ok(Record::LinesHit { hit: 45 }));
    /// ```
    LinesHit {
        /// Number of lines with a non-zero execution count.
        hit: u32,
    },

    /// Represents a `end_of_record` record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Record;
    /// assert_eq!("end_of_record".parse(), Ok(Record::EndOfRecord));
    /// ```
    EndOfRecord,
}

/// Represents all LCOV record kinds.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecordKind {
    /// Represents a `TN` record.
    TestName,
    /// Represents a `SF` record.
    SourceFile,
    /// Represents a `FN` record.
    FunctionName,
    /// Represents a `FNDA` record.
    FunctionData,
    /// Represents a `FNF` record.
    FunctionsFound,
    /// Represents a `FNH` record.
    FunctionsHit,
    /// Represents a `BRDA` record.
    BranchData,
    /// Represents a `BRF` record.
    BranchesFound,
    /// Represents a `BRH` record.
    BranchesHit,
    /// Represents a `DA` record.
    LineData,
    /// Represents a `LF` record.
    LinesFound,
    /// Represents a `LH` record.
    LinesHit,
    /// Represents a `end_of_record` record.
    EndOfRecord,
}

macro_rules! kind_impl {
    ($rec:expr; $($kind:ident),*) => {
        match $rec {
            $(Record::$kind { .. } => RecordKind::$kind),*
        }
    }
}

impl Record {
    /// Returns the corresponding `RecordKind` for this record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::{Record, RecordKind};
    /// let rec = Record::LinesHit { hit: 32 };
    /// assert_eq!(rec.kind(), RecordKind::LinesHit);
    /// ```
    pub fn kind(&self) -> RecordKind {
        kind_impl! {
            *self;
            TestName, SourceFile,
            FunctionName, FunctionData, FunctionsFound, FunctionsHit,
            BranchData, BranchesFound, BranchesHit,
            LineData, LinesFound, LinesHit,
            EndOfRecord
        }
    }
}

impl RecordKind {
    /// Returns the corresponding `&str` for the record kind.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::RecordKind;
    /// assert_eq!(RecordKind::TestName.as_str(), "TN");
    /// ```
    pub fn as_str(&self) -> &'static str {
        use RecordKind::*;

        match *self {
            TestName => "TN",
            SourceFile => "SF",
            FunctionName => "FN",
            FunctionData => "FNDA",
            FunctionsFound => "FNF",
            FunctionsHit => "FNH",
            BranchData => "BRDA",
            BranchesFound => "BRF",
            BranchesHit => "BRH",
            LineData => "DA",
            LinesFound => "LF",
            LinesHit => "LH",
            EndOfRecord => "end_of_record",
        }
    }
}
