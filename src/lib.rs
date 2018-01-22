#[macro_use]
extern crate failure;

pub use reader::{Error as ReadError, Reader};
pub use record::{ParseRecordError, Record, RecordKind};

mod record;
mod reader;
