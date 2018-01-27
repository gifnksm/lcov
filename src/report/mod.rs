use self::parser::Parser;
use self::section::Section;
use super::{Record, RecordKind};
use std::{iter, mem};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::path::PathBuf;

#[macro_use]
mod parser;
pub(crate) mod section;

#[derive(Debug, Clone, Fail, Eq, PartialEq)]
pub enum MergeError<ReadError> {
    #[fail(display = "failed to read record: {}", _0)] Read(#[cause] ReadError),
    #[fail(display = "unexpected record `{}`", _0)] UnexpectedRecord(RecordKind),
    #[fail(display = "unexpected end of stream")] UnexpectedEof,
    #[fail(display = "unmatched function line")] UnmatchedFunctionLine,
    #[fail(display = "unmatched function name")] UnmatchedFunctionName,
    #[fail(display = "unmatches checksum")] UnmatchedChecksum,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Report {
    sections: BTreeMap<SectionKey, Section>,
}

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct SectionKey {
    pub(crate) test_name: String,
    pub(crate) source_file: PathBuf,
}

impl Report {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge<I, E>(&mut self, it: I) -> Result<(), MergeError<E>>
    where
        I: IntoIterator<Item = Result<Record, E>>,
    {
        let mut parser = Parser::new(it.into_iter());

        while let Some(_) = parser.peek().map_err(MergeError::Read)? {
            let test_name =
                eat_if_matches!(parser, Record::TestName { name } => name).unwrap_or("".into());
            let source_file = eat!(parser, Record::SourceFile { path } => path);
            let key = SectionKey {
                test_name,
                source_file,
            };
            match self.sections.entry(key) {
                Entry::Vacant(e) => {
                    let mut section = Section::default();
                    section.merge(&mut parser)?;
                    // If the new section contains no data, ignore it.
                    // LCOV merge (`lcov -c -a XXX`) behaves the same way.
                    if !section.is_empty() {
                        e.insert(section);
                    }
                }
                Entry::Occupied(mut e) => e.get_mut().merge(&mut parser)?,
            }
            eat!(parser, Record::EndOfRecord);
        }

        Ok(())
    }

    pub(crate) fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut((SectionKey, Section)) -> Option<(SectionKey, Section)>,
    {
        let sections = mem::replace(&mut self.sections, BTreeMap::new());
        self.sections.extend(sections.into_iter().filter_map(f));
    }
}

impl IntoIterator for Report {
    type Item = Record;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let iter = self.sections.into_iter().flat_map(|(key, section)| {
            iter::once(Record::TestName {
                name: key.test_name,
            }).chain(iter::once(Record::SourceFile {
                path: key.source_file,
            }))
                .chain(section.into_iter())
                .chain(iter::once(Record::EndOfRecord))
        });
        IntoIter {
            inner: Box::new(iter),
        }
    }
}

pub struct IntoIter {
    inner: Box<Iterator<Item = Record>>,
}

impl Iterator for IntoIter {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
