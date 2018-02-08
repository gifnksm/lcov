use super::{MergeError, Parser, Record};
use std::{iter, mem};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct BranchList {
    list: BTreeMap<BranchKey, BranchData>,
}

impl BranchList {
    pub(crate) fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub(crate) fn merge<I, E>(
        &mut self,
        parser: &mut Parser<I, Record>,
    ) -> Result<(), MergeError<E>>
    where
        I: Iterator<Item = Result<Record, E>>,
    {
        while let Some((key, taken)) = eat_if_matches!(parser,
            Record::BranchData { line, block, branch, taken } => {
                (BranchKey { line, block, branch }, BranchData {taken})
            }
        ) {
            let org = self.list.entry(key).or_insert_with(BranchData::default);
            if let BranchData { taken: Some(taken) } = taken {
                org.taken = Some(org.taken.unwrap_or(0) + taken);
            }
        }

        let _ = eat_if_matches!(parser, Record::BranchesFound { .. });
        let _ = eat_if_matches!(parser, Record::BranchesHit { .. });

        Ok(())
    }

    pub(crate) fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut((BranchKey, BranchData)) -> Option<(BranchKey, BranchData)>,
    {
        let list = mem::replace(&mut self.list, BTreeMap::new());
        self.list.extend(list.into_iter().filter_map(f));
    }
}

#[derive(Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct BranchKey {
    pub(crate) line: u32,
    pub(crate) block: u32,
    pub(crate) branch: u32,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct BranchData {
    pub(crate) taken: Option<u64>,
}

impl IntoIterator for BranchList {
    type Item = Record;
    type IntoIter = Box<Iterator<Item = Record>>;

    fn into_iter(self) -> Self::IntoIter {
        if self.list.is_empty() {
            return Box::new(iter::empty());
        }

        let found = self.list.len() as u32;

        enum Branch {
            Data((BranchKey, BranchData)),
            Found,
            Hit(u32),
        }
        let iter = self.list
            .into_iter()
            .map(Branch::Data)
            .chain(iter::once(Branch::Found))
            .chain(iter::once(Branch::Hit(0)))
            .scan(0, |hit_count, mut rec| {
                match &mut rec {
                    &mut Branch::Data((_, ref data)) => if data.taken.unwrap_or(0) > 0 {
                        *hit_count += 1
                    },
                    &mut Branch::Found => {}
                    &mut Branch::Hit(ref mut hit) => *hit = *hit_count,
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
}
