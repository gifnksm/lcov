#[macro_use]
extern crate failure;

pub use line_filter::Filter as LineFilter;
pub use reader::{Error as ReadError, Reader};
pub use record::{ParseRecordError, Record, RecordKind};
pub use report::{MergeError, Report};

mod line_filter;
mod report;
mod record;
mod reader;
