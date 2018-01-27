use super::{MergeError, Parser, Record};
use std::{iter, mem};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct LineList {
    list: BTreeMap<LineKey, LineData>,
}

impl LineList {
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
        while let Some((line, count, checksum)) = eat_if_matches!(parser,
            Record::LineData { line, count, checksum } => {
                (line, count, checksum)
            }
        ) {
            let org = self.list
                .entry(LineKey { line })
                .or_insert(LineData::default());
            org.count += count;
            if let Some(checksum) = checksum {
                if let Some(org_checksum) = org.checksum.as_ref() {
                    if checksum != *org_checksum {
                        Err(MergeError::UnmatchedChecksum)?;
                    }
                }
                org.checksum = Some(checksum);
            }
        }

        eat_if_matches!(parser, Record::LinesFound { .. });
        eat_if_matches!(parser, Record::LinesHit { .. });

        Ok(())
    }

    pub(crate) fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut((LineKey, LineData)) -> Option<(LineKey, LineData)>,
    {
        let list = mem::replace(&mut self.list, BTreeMap::new());
        self.list.extend(list.into_iter().filter_map(f));
    }
}

#[derive(Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct LineKey {
    pub(crate) line: u32,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct LineData {
    pub(crate) count: u64,
    pub(crate) checksum: Option<String>,
}

impl IntoIterator for LineList {
    type Item = Record;
    type IntoIter = Box<Iterator<Item = Record>>;

    fn into_iter(self) -> Self::IntoIter {
        if self.list.is_empty() {
            return Box::new(iter::empty());
        }

        let found = self.list.len() as u32;
        enum Line {
            Data((LineKey, LineData)),
            Found,
            Hit(u32),
        }
        let iter = self.list
            .into_iter()
            .map(Line::Data)
            .chain(iter::once(Line::Found))
            .chain(iter::once(Line::Hit(0)))
            .scan(0, |hit_count, mut rec| {
                match &mut rec {
                    &mut Line::Data((_, ref data)) => if data.count > 0 {
                        *hit_count += 1
                    },
                    &mut Line::Found => {}
                    &mut Line::Hit(ref mut hit) => *hit = *hit_count,
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
}
