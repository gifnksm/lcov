#[macro_use]
extern crate failure;

pub use reader::Reader;
pub use record::{ParseError as ParseRecordError, Record};

mod record;
mod reader;
