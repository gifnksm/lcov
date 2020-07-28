//! A coverage information about a function.
//!
//! Some coverage information is stored in a [`Functions`] as `BTreeMap` .
//!
//! [`Functions`]: ./type.Functions.html
use super::{Merge, MergeError, ParseError, Parser, ReadError, Record};
use std::collections::BTreeMap;
use std::iter;

/// A map of coverage information about functions.
pub type Functions = BTreeMap<Key, Value>;

/// A key of a coverage information about a function.
///
/// This struct is used as a key of [`Functions`].
///
/// [`Functions`]: ./type.Functions.html
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Key {
    /// Function name.
    pub name: String,
}

/// A value of a coverage information about a function.
///
/// This struct is used as a value of [`Functions`].
///
/// [`Functions`]: ./type.Functions.html
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct Value {
    /// Line number of function start.
    pub start_line: Option<u32>,
    /// Execution count.
    pub count: u64,
}

impl Merge for Value {
    fn merge(&mut self, other: Self) -> Result<(), MergeError> {
        if let Some(start_line) = other.start_line.as_ref() {
            if let Some(my_start_line) = self.start_line.as_ref() {
                if start_line != my_start_line {
                    return Err(MergeError::UnmatchedFunctionLine);
                }
            }
        }
        // Don't check end_line. The value may differ between tracefiles.
        self.merge_lossy(other);
        Ok(())
    }

    fn merge_lossy(&mut self, other: Self) {
        if other.start_line.is_some() {
            self.start_line = other.start_line;
        }
        self.count = u64::saturating_add(self.count, other.count);
    }
}

pub(crate) fn parse<I>(parser: &mut Parser<I, Record>) -> Result<Functions, ParseError>
where
    I: Iterator<Item = Result<Record, ReadError>>,
{
    let mut functions = Functions::new();
    while let Some((key, start_line)) = eat_if_matches!(parser,
        Record::FunctionName { name, start_line } => (Key { name }, start_line)
    ) {
        let _ = functions.insert(
            key,
            Value {
                start_line: Some(start_line),
                count: 0,
            },
        );
    }

    while let Some((key, count)) = eat_if_matches!(parser,
        Record::FunctionData { name, count } => (Key { name }, count)
    ) {
        let data = functions.entry(key).or_insert_with(Value::default);
        data.count += count;
    }

    let _ = eat_if_matches!(parser, Record::FunctionsFound { .. });
    let _ = eat_if_matches!(parser, Record::FunctionsHit { .. });

    Ok(functions)
}

pub(crate) fn into_records(functions: Functions) -> Box<dyn Iterator<Item = Record>> {
    if functions.is_empty() {
        return Box::new(iter::empty());
    }

    let found = functions.len() as u32;
    let mut functions = functions.into_iter().collect::<Vec<_>>();
    functions.sort_by_key(|&(_, ref data)| data.start_line);

    enum Func {
        Line(String, u32),
        Data(String, u64),
        Found,
        Hit(u32),
    }
    let line = functions.clone().into_iter().filter_map(|(key, data)| {
        data.start_line
            .map(|start_line| Func::Line(key.name, start_line))
    });
    let count = functions
        .into_iter()
        .map(|(key, data)| Func::Data(key.name, data.count));
    let iter = line
        .chain(count)
        .chain(iter::once(Func::Found))
        .chain(iter::once(Func::Hit(0)))
        .scan(0, |hit_count, mut rec| {
            match rec {
                Func::Data(_, ref mut count) if *count > 0 => *hit_count += 1,
                Func::Hit(ref mut hit) => *hit = *hit_count,
                Func::Line(..) | Func::Data(..) | Func::Found => {}
            }
            Some(rec)
        })
        .map(move |rec| match rec {
            Func::Line(name, start_line) => Record::FunctionName { name, start_line },
            Func::Data(name, count) => Record::FunctionData { name, count },
            Func::Found => Record::FunctionsFound { found },
            Func::Hit(hit) => Record::FunctionsHit { hit },
        });
    Box::new(iter)
}
