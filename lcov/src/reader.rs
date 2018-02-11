//! A reader of [LCOV records].
//!
//! The [`Reader`] structure reads LCOV records from arbitrary buffered reader.
//!
//! If you want to create a reader which reads am LCOV tracefile, you can use [`open_file`] function.
//!
//! [LCOV records]: ../enum.Record.html
//! [`Reader`]: struct.Reader.html
//! [`open_file`]: ../fn.open_file.html
use super::record::{ParseRecordError, Record};
use std::io::{self, BufRead, Lines};

/// Reading an LCOV records from a buffered reader.
#[derive(Debug)]
pub struct Reader<B> {
    lines: Lines<B>,
    line: u32,
}

impl<B> Reader<B> {
    /// Creates a new `Reader`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use std::io::BufReader;
    /// use std::fs::File;
    /// use lcov::Reader;
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
    ///
    /// let reader = Reader::new(input.as_bytes());
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn new(buf: B) -> Reader<B>
    where
        B: BufRead,
    {
        Reader {
            lines: buf.lines(),
            line: 0,
        }
    }
}

/// All possible errors that can occur when reading LCOV tracefile.
#[derive(Debug, Fail)]
pub enum Error {
    /// An error indicating that I/O operation failed.
    ///
    /// This error occurs when the underlying reader returns an error.
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    /// An error indicating that record parsing failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use] extern crate matches;
    /// # extern crate lcov;
    /// # fn main() {
    /// use lcov::Reader;
    /// use lcov::reader::Error as ReadError;
    /// use lcov::record::ParseRecordError;
    /// let mut reader = Reader::new("FOO:1,2".as_bytes());
    /// assert_matches!(reader.next(), Some(Err(ReadError::ParseRecord(1, ParseRecordError::UnknownRecord))));
    /// # }
    /// ```
    #[fail(display = "invalid record syntax at line {}: {}", _0, _1)]
    ParseRecord(u32, #[cause] ParseRecordError),
}

impl<B> Iterator for Reader<B>
where
    B: BufRead,
{
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line| {
            line.map_err(Error::Io).and_then(|line| {
                self.line += 1;
                line.parse().map_err(|e| Error::ParseRecord(self.line, e))
            })
        })
    }
}
