//! A coverage information about a line.
//!
//! Some coverage information is stored in a [`Lines`] as `BTreeMap` .
//!
//! [`Lines`]: ./type.Linesh.html
use super::{Merge, MergeError, ParseError, Parser, Record};
use failure::Error;
use std::collections::BTreeMap;
use std::iter;

/// A map of coverage information about lines.
pub type Lines = BTreeMap<Key, Value>;

/// A key of a coverage information about a line.
///
/// This struct is used as a key of [`Lines`].
///
/// [`Lines`]: ./type.Lines.html
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Key {
    /// Line number.
    pub line: u32,
}

/// A value of a coverage information about a line.
///
/// This struct is used as a value of [`Lines`].
///
/// [`Lines`]: ./type.Lines.html
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Value {
    /// Execution count.
    pub count: u64,
    /// Checksum for each instrumented line.
    pub checksum: Option<String>,
}

impl Merge for Value {
    fn merge(&mut self, other: Self) -> Result<(), MergeError> {
        if let Some(checksum) = other.checksum.as_ref() {
            if let Some(my_checksum) = self.checksum.as_ref() {
                if checksum != my_checksum {
                    Err(MergeError::UnmatchedChecksum)?;
                }
            }
        }

        self.merge_lossy(other);
        Ok(())
    }

    fn merge_lossy(&mut self, other: Self) {
        if other.checksum.is_some() {
            self.checksum = other.checksum;
        }
        self.count += other.count;
    }
}

pub(crate) fn parse<I>(parser: &mut Parser<I, Record>) -> Result<Lines, ParseError>
where
    I: Iterator<Item = Result<Record, Error>>,
{
    let mut lines = Lines::new();

    while let Some((line, count, checksum)) = eat_if_matches!(parser,
        Record::LineData { line, count, checksum } => {
            (line, count, checksum)
        }
    ) {
        let _ = lines.insert(Key { line }, Value { count, checksum });
    }

    let _ = eat_if_matches!(parser, Record::LinesFound { .. });
    let _ = eat_if_matches!(parser, Record::LinesHit { .. });

    Ok(lines)
}

pub(crate) fn into_records(lines: Lines) -> Box<dyn Iterator<Item = Record>> {
    if lines.is_empty() {
        return Box::new(iter::empty());
    }

    let found = lines.len() as u32;
    enum Line {
        Data((Key, Value)),
        Found,
        Hit(u32),
    }
    let iter = lines
        .into_iter()
        .map(Line::Data)
        .chain(iter::once(Line::Found))
        .chain(iter::once(Line::Hit(0)))
        .scan(0, |hit_count, mut rec| {
            match rec {
                Line::Data((_, ref data)) => {
                    if data.count > 0 {
                        *hit_count += 1
                    }
                }
                Line::Found => {}
                Line::Hit(ref mut hit) => *hit = *hit_count,
            };
            Some(rec)
        })
        .map(move |rec| match rec {
            Line::Data((key, data)) => Record::LineData {
                line: key.line,
                count: data.count,
                checksum: data.checksum,
            },
            Line::Found => Record::LinesFound { found },
            Line::Hit(hit) => Record::LinesHit { hit },
        });

    Box::new(iter)
}
