use super::RecordKind;
use crate::reader;

/// All possible errors that can occur when parsing LCOV records.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// An error indicating that reading record operation failed.
    ///
    /// This error occurs when the underlying reader returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use matches::assert_matches;
    /// # fn main() {
    /// use lcov::{Reader, Report};
    /// use lcov::report::ParseError;
    /// assert_matches!(Report::from_reader(Reader::new("FOO:1,2,3".as_bytes())), Err(ParseError::Read(_)));
    /// # }
    /// ```
    #[error("failed to read record: {}", _0)]
    Read(#[from] reader::Error),

    /// An error indicating that unexpected kind of record is read.
    ///
    /// This error occurs when the LCOV tracefile (or underlying reader) contains invalid record sequence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use matches::assert_matches;
    /// # fn main() {
    /// use lcov::{Reader, Report, RecordKind};
    /// use lcov::report::ParseError;
    /// let input = "\
    /// TN:test_name
    /// SF:/usr/include/stdio.h
    /// TN:next_test
    /// ";
    /// assert_matches!(Report::from_reader(Reader::new(input.as_bytes())),
    ///                 Err(ParseError::UnexpectedRecord(RecordKind::TestName)));
    /// # }
    /// ```
    #[error("unexpected record `{}`", _0)]
    UnexpectedRecord(RecordKind),

    /// An error indicating that unexpected "end of file".
    ///
    /// This error occurs when the LCOV tracefile (or underlying reader) contains invalid record sequence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use matches::assert_matches;
    /// # fn main() {
    /// use lcov::{Reader, Report};
    /// use lcov::report::ParseError;
    /// let input = "\
    /// TN:test_name
    /// SF:/usr/include/stdio.h
    /// ";
    /// assert_matches!(Report::from_reader(Reader::new(input.as_bytes())),
    ///                 Err(ParseError::UnexpectedEof));
    /// # }
    /// ```
    #[error("unexpected end of file")]
    UnexpectedEof,
}

/// All possible errors that can occur when merging LCOV records.
#[derive(Debug, Copy, Clone, Eq, PartialEq, thiserror::Error)]
pub enum MergeError {
    /// An error indicating that start line of functions are not same.
    ///
    /// This error occurs when merging not compatible LCOV tracefiles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use matches::assert_matches;
    /// # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    /// use lcov::{Reader, Report};
    /// use lcov::report::MergeError;
    /// let input1 = "\
    /// TN:test_name
    /// SF:foo.c
    /// FN:3,foo
    /// end_of_record
    /// ";
    /// let input2 = "\
    /// TN:test_name
    /// SF:foo.c
    /// FN:4,foo
    /// end_of_record
    /// ";
    /// let mut report1 = Report::from_reader(Reader::new(input1.as_bytes()))?;
    /// let report2 = Report::from_reader(Reader::new(input2.as_bytes()))?;
    /// assert_matches!(report1.merge(report2),
    ///                 Err(MergeError::UnmatchedFunctionLine));
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # try_main().expect("failed to run test");
    /// # }
    /// ```
    #[error("unmatched start line of function")]
    UnmatchedFunctionLine,

    /// An error indicating that checksum of lines are not same.
    ///
    /// This error occurs when merging not compatible LCOV tracefiles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use matches::assert_matches;
    /// # fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    /// use lcov::{Reader, Report};
    /// use lcov::report::MergeError;
    /// let input1 = "\
    /// TN:test_name
    /// SF:foo.c
    /// DA:4,1,valid_checksum
    /// end_of_record
    /// ";
    /// let input2 = "\
    /// TN:test_name
    /// SF:foo.c
    /// DA:4,4,invalid_checksum
    /// end_of_record
    /// ";
    /// let mut report1 = Report::from_reader(Reader::new(input1.as_bytes()))?;
    /// let report2 = Report::from_reader(Reader::new(input2.as_bytes()))?;
    /// assert_matches!(report1.merge(report2),
    ///                 Err(MergeError::UnmatchedChecksum));
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # try_main().expect("failed to run test");
    /// # }
    /// ```
    #[error("unmatched checksum")]
    UnmatchedChecksum,
}
