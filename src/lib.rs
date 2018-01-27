#[macro_use]
extern crate failure;

pub use reader::{Error as ReadError, Reader};
pub use record::{ParseRecordError, Record, RecordKind};
pub use report::{MergeError, Report};

pub mod line_filter;
mod report;
mod record;
mod reader;
