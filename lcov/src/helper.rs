//! Helper functions.

use super::Reader;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

/// Opens an LCOV tracefile.
///
/// # Example
///
/// ```rust
/// # extern crate failure;
/// # extern crate lcov;
/// # use failure::Error;
/// #
/// # fn foo() -> Result<(), Error> {
/// let reader = lcov::open_file("report.info")?;
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
pub fn open_file<P>(path: P) -> Result<Reader<BufReader<File>>, io::Error>
where
    P: AsRef<Path>,
{
    Ok(Reader::new(BufReader::new(File::open(path)?)))
}
