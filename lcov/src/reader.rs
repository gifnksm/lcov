use super::{ParseRecordError, Record};
use std::io::{self, BufRead, Lines};

/// Reading a LCOV records from a buffered reader.
#[derive(Debug)]
pub struct Reader<B> {
    lines: Lines<B>,
    line: u32,
}

impl<B> Reader<B>
where
    B: BufRead,
{
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
    /// let file = File::open("report.info")?;
    /// let buf = BufReader::new(file);
    /// let reader = Reader::new(buf);
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn new(buf: B) -> Reader<B> {
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
    /// use lcov::{ParseRecordError, ReadError, Reader};
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
