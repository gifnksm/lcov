//! A coverage information about a branch.
//!
//! Some coverage information is stored in a [`Branches`] as `BTreeMap` .
//!
//! [`Branches`]: ./type.Branches.html
use super::{Merge, MergeError, ParseError, Parser, Record};
use failure::Error;
use std::collections::BTreeMap;
use std::iter;

/// A map of coverage information about branches.
pub type Branches = BTreeMap<Key, Value>;

/// A key of a coverage information about a branch.
///
/// This struct is used as a key of [`Branches`].
///
/// [`Branches`]: ./type.Branches.html
#[derive(Debug, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Key {
    /// Line number.
    pub line: u32,
    /// Block number.
    pub block: u32,
    /// Branch number.
    pub branch: u32,
}

/// A value of a coverage information about a branch.
///
/// This struct is used as a value of [`Branches`].
///
/// [`Branches`]: ./type.Branches.html
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct Value {
    /// A number indicating how often that branch was taken.
    pub taken: Option<u64>,
}

impl Merge for Value {
    fn merge(&mut self, other: Self) -> Result<(), MergeError> {
        self.merge_lossy(other);
        Ok(())
    }

    fn merge_lossy(&mut self, other: Self) {
        if let Value { taken: Some(taken) } = other {
            self.taken = Some(self.taken.unwrap_or(0) + taken);
        }
    }
}

pub(crate) fn parse<I>(parser: &mut Parser<I, Record>) -> Result<Branches, ParseError>
where
    I: Iterator<Item = Result<Record, Error>>,
{
    let mut branches = Branches::new();

    while let Some((key, value)) = eat_if_matches!(parser,
        Record::BranchData { line, block, branch, taken } => {
            (Key { line, block, branch }, Value { taken })
        }
    ) {
        let _ = branches.insert(key, value);
    }

    let _ = eat_if_matches!(parser, Record::BranchesFound { .. });
    let _ = eat_if_matches!(parser, Record::BranchesHit { .. });

    Ok(branches)
}

pub(crate) fn into_records(branches: Branches) -> Box<Iterator<Item = Record>> {
    if branches.is_empty() {
        return Box::new(iter::empty());
    }

    let found = branches.len() as u32;

    enum Branch {
        Data((Key, Value)),
        Found,
        Hit(u32),
    }
    let iter = branches
        .into_iter()
        .map(Branch::Data)
        .chain(iter::once(Branch::Found))
        .chain(iter::once(Branch::Hit(0)))
        .scan(0, |hit_count, mut rec| {
            match rec {
                Branch::Data((_, ref data)) => {
                    if data.taken.unwrap_or(0) > 0 {
                        *hit_count += 1
                    }
                }
                Branch::Found => {}
                Branch::Hit(ref mut hit) => *hit = *hit_count,
            }
            Some(rec)
        })
        .map(move |rec| match rec {
            Branch::Data((key, data)) => Record::BranchData {
                line: key.line,
                block: key.block,
                branch: key.branch,
                taken: data.taken,
            },
            Branch::Found => Record::BranchesFound { found },
            Branch::Hit(hit) => Record::BranchesHit { hit },
        });

    Box::new(iter)
}
