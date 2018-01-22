#[macro_use]
extern crate failure;

pub use merger::{Error as MergeError, Merger};
pub use reader::{Error as ReadError, Reader};
pub use record::{ParseRecordError, Record, RecordKind};

mod merger;
mod record;
mod reader;
