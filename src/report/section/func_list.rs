use super::{MergeError, Parser, Record};
use std::{iter, mem};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct FuncList {
    list: BTreeMap<FuncKey, FuncData>,
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
        let mut func_list = vec![];
        while let Some((key, start_line)) =
            eat_if_matches!(parser, Record::FunctionName { name, start_line } => (FuncKey { name}, start_line))
        {
            func_list.push((
                key,
                FuncData {
                    start_line,
                    end_line: 0,
                    count: 0,
                },
            ));
        }

        func_list.sort_by_key(|&(_, ref data)| data.start_line);
        let mut end = u32::max_value();
        for data in func_list.iter_mut().rev() {
            data.1.end_line = end;
            end = u32::saturating_sub(data.1.start_line, 1);
        }

        for (name, data) in func_list {
            match self.list.entry(name) {
                Entry::Vacant(e) => {
                    let _ = e.insert(data);
                }
                Entry::Occupied(mut e) => {
                    let set_data = e.get_mut();
                    if set_data.start_line != data.start_line {
                        Err(MergeError::UnmatchedFunctionLine)?;
                    }
                    set_data.end_line = u32::min(set_data.end_line, data.end_line);
                }
            }
        }

        while let Some((key, count)) =
            eat_if_matches!(parser, Record::FunctionData { name, count } => { (FuncKey {name}, count) })
        {
            match self.list.get_mut(&key) {
                Some(data) => data.count += count,
                None => Err(MergeError::UnmatchedFunctionName)?,
            }
        }

        eat_if_matches!(parser, Record::FunctionsFound { .. });
        eat_if_matches!(parser, Record::FunctionsHit { .. });

        Ok(())
    }

    pub(crate) fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut((FuncKey, FuncData)) -> Option<(FuncKey, FuncData)>,
    {
        let list = mem::replace(&mut self.list, BTreeMap::new());
        self.list.extend(list.into_iter().filter_map(f));
    }
}

#[derive(Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct FuncKey {
    name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FuncData {
    pub(crate) start_line: u32,
    pub(crate) end_line: u32,
    pub(crate) count: u64,
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
            .map(|(key, data)| Func::Line(key.name, data.start_line));
        let count = list.into_iter()
            .map(|(key, data)| Func::Data(key.name, data.count));
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
