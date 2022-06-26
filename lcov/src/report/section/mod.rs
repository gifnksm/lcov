//! A coverage information about a source file.
//!
//! Some coverage information is stored in a [`Sections`] as `BTreeMap` .
//!
//! [`Sections`]: ./type.Sections.html
use self::branch::Branches;
use self::function::Functions;
use self::line::Lines;
use super::{Merge, MergeError, ParseError, Parser, ReadError, Record, RecordKind};
use std::collections::BTreeMap;
use std::iter;
use std::path::PathBuf;

pub mod branch;
pub mod function;
pub mod line;

/// A map of coverage information about source files.
pub type Sections = BTreeMap<Key, Value>;

/// A key of a coverage information about a source file.
///
/// This struct is used as a key of [`Sections`].
///
/// [`Sections`]: ./type.Sections.html
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Key {
    /// Name of the test.
    pub test_name: String,
    /// Path of the source file.
    pub source_file: PathBuf,
}

/// A value of a coverage information about a source file.
///
/// This struct is  used as a value of [`Sections`].
///
/// [`Sections`]: ./type.Sections.html
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Value {
    /// Function coverage information in the section.
    pub functions: Functions,
    /// Branch coverage information in the section.
    pub branches: Branches,
    /// Line coverage information in the section.
    pub lines: Lines,
}

impl Value {
    /// Returns `true` if `self` has no coverage information.
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.branches.is_empty() && self.lines.is_empty()
    }
}

impl Merge for Value {
    fn merge(&mut self, other: Self) -> Result<(), MergeError> {
        self.functions.merge(other.functions)?;
        self.branches.merge(other.branches)?;
        self.lines.merge(other.lines)?;
        Ok(())
    }

    fn merge_lossy(&mut self, other: Self) {
        self.functions.merge_lossy(other.functions);
        self.branches.merge_lossy(other.branches);
        self.lines.merge_lossy(other.lines);
    }
}

pub(crate) fn parse<I>(parser: &mut Parser<I, Record>) -> Result<Sections, ParseError>
where
    I: Iterator<Item = Result<Record, ReadError>>,
{
    let mut sections = Sections::new();

    while parser.peek().map_err(ParseError::Read)?.is_some() {
        // Sometimes, lcov emits TN: records multiple times, so skip the first TN: record.
        let mut test_name = None;
        while let Some(tn) = eat_if_matches!(parser, Record::TestName { name } => name) {
            test_name = Some(tn);
        }
        // Sometimes, lcov emit extra TN: records at the end of the tracefile.
        if parser.peek().map_err(ParseError::Read)?.is_none() {
            break;
        }

        let mut source_file = None;
        let mut functions = Functions::default();
        let mut branches = Branches::default();
        let mut lines = Lines::default();

        loop {
            match parser.pop()?.ok_or(ParseError::UnexpectedEof)? {
                rec @ Record::TestName { .. } => {
                    return Err(ParseError::UnexpectedRecord {
                        expected: RecordKind::EndOfRecord,
                        found: rec.kind(),
                    })
                }
                Record::SourceFile { path } => source_file = Some(path),
                Record::FunctionName { name, start_line } => {
                    let _ = functions.insert(
                        function::Key { name },
                        function::Value {
                            start_line: Some(start_line),
                            count: 0,
                        },
                    );
                }
                Record::FunctionData { name, count } => {
                    let data = functions.entry(function::Key { name }).or_default();
                    data.count += count;
                }
                Record::FunctionsFound { .. } => {} // ignore
                Record::FunctionsHit { .. } => {}   // ignore
                Record::BranchData {
                    line,
                    block,
                    branch,
                    taken,
                } => {
                    let _ = branches.insert(
                        branch::Key {
                            line,
                            block,
                            branch,
                        },
                        branch::Value { taken },
                    );
                }
                Record::BranchesFound { .. } => {} // ignore
                Record::BranchesHit { .. } => {}   // ignore
                Record::LineData {
                    line,
                    count,
                    checksum,
                } => {
                    let _ = lines.insert(line::Key { line }, line::Value { count, checksum });
                }
                Record::LinesFound { .. } => {} // ignore
                Record::LinesHit { .. } => {}   // ignore
                Record::EndOfRecord => break,
            }
        }

        let key = Key {
            test_name: test_name.unwrap_or_default(),
            source_file: source_file.unwrap_or_default(),
        };
        let value = Value {
            functions,
            branches,
            lines,
        };
        // If the new section contains no data, ignore it.
        // LCOV merge (`lcov -c -a XXX`) behaves the same way.
        if !value.is_empty() {
            let _ = sections.insert(key, value);
        }
    }

    Ok(sections)
}

pub(crate) fn into_records(sections: Sections) -> Box<dyn Iterator<Item = Record>> {
    let iter = sections.into_iter().flat_map(|(key, value)| {
        let test_name = Record::TestName {
            name: key.test_name,
        };
        let source_file = Record::SourceFile {
            path: key.source_file,
        };
        iter::once(test_name)
            .chain(iter::once(source_file))
            .chain(function::into_records(value.functions))
            .chain(branch::into_records(value.branches))
            .chain(line::into_records(value.lines))
            .chain(iter::once(Record::EndOfRecord))
    });
    Box::new(iter)
}
