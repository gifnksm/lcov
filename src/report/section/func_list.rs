use super::{MergeError, Parser, Record};
use std::collections::BTreeMap;
use std::iter;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct FuncList {
    list: BTreeMap<String, FuncData>,
}

impl FuncList {
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
        while let Some((name, start_line)) =
            eat_if_matches!(parser, Record::FunctionName { name, start_line } => (name, start_line))
        {
            let data = self.list.entry(name).or_insert(FuncData {
                start_line,
                count: 0,
            });
            if data.start_line != start_line {
                Err(MergeError::UnmatchedFunctionLine)?;
            }
        }

        while let Some((name, count)) =
            eat_if_matches!(parser, Record::FunctionData { name, count } => { (name, count) })
        {
            match self.list.get_mut(&name) {
                Some(data) => data.count += count,
                None => Err(MergeError::UnmatchedFunctionName)?,
            }
        }

        eat_if_matches!(parser, Record::FunctionsFound { .. });
        eat_if_matches!(parser, Record::FunctionsHit { .. });

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct FuncData {
    start_line: u32,
    count: u64,
}

impl IntoIterator for FuncList {
    type Item = Record;
    type IntoIter = Box<Iterator<Item = Record>>;

    fn into_iter(self) -> Self::IntoIter {
        if self.list.is_empty() {
            return Box::new(iter::empty());
        }

        let found = self.list.len() as u32;
        let mut list = self.list.into_iter().collect::<Vec<_>>();
        list.sort_by_key(|&(_, ref data)| data.start_line);

        enum Func {
            Line(String, u32),
            Data(String, u64),
            Found,
            Hit(u32),
        }
        let line = list.clone()
            .into_iter()
            .map(|(name, data)| Func::Line(name, data.start_line));
        let count = list.into_iter()
            .map(|(name, data)| Func::Data(name, data.count));
        let iter = line.chain(count)
            .chain(iter::once(Func::Found))
            .chain(iter::once(Func::Hit(0)))
            .scan(0, |hit_count, mut rec| {
                match &mut rec {
                    &mut Func::Line(..) => {}
                    &mut Func::Data(_, ref mut count) if *count > 0 => *hit_count += 1,
                    &mut Func::Data(..) => {}
                    &mut Func::Found => {}
                    &mut Func::Hit(ref mut hit) => *hit = *hit_count,
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
}
