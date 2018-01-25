use super::{MergeError, Parser, Record};
use std::collections::BTreeMap;
use std::iter;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct BranchList {
    list: BTreeMap<BranchKey, Option<u64>>,
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
                (BranchKey { line, block, branch }, taken)
            }
        ) {
            let org = self.list.entry(key).or_insert(None);
            if let Some(taken) = taken {
                *org = Some(org.unwrap_or(0) + taken);
            }
        }

        eat_if_matches!(parser, Record::BranchesFound { .. });
        eat_if_matches!(parser, Record::BranchesHit { .. });

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct BranchKey {
    line: u32,
    block: u32,
    branch: u32,
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
            Data((BranchKey, Option<u64>)),
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
                    &mut Branch::Data((_, taken)) => if taken.unwrap_or(0) > 0 {
                        *hit_count += 1
                    },
                    &mut Branch::Found => {}
                    &mut Branch::Hit(ref mut hit) => *hit = *hit_count,
                }
                Some(rec)
            })
            .map(move |rec| match rec {
                Branch::Data((key, taken)) => Record::BranchData {
                    line: key.line,
                    block: key.block,
                    branch: key.branch,
                    taken: taken,
                },
                Branch::Found => Record::BranchesFound { found },
                Branch::Hit(hit) => Record::BranchesHit { hit },
            });

        Box::new(iter)
    }
}
