#[macro_use]
extern crate failure;

pub use record::{ParseError as ParseRecordError, Record};

mod record;
