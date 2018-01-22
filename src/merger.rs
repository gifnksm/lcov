use super::{Record, RecordKind};
use std::collections::BTreeMap;
use std::collections::btree_map;
use std::mem;
use std::path::PathBuf;

#[derive(Debug, Clone, Fail, Eq, PartialEq)]
pub enum Error<ReadError> {
    #[fail(display = "failed to read record: {}", _0)] Read(#[cause] ReadError),
    #[fail(display = "unexpected record `{}`", _0)] UnexpectedRecord(RecordKind),
    #[fail(display = "unexpected end of stream")] UnexpectedEof,
    #[fail(display = "unmatched function line")] UnmatchedFunctionLine,
    #[fail(display = "unmatches checksum")] UnmatchedChecksum,
}

macro_rules! eat {
    ($parser:expr, $p:pat) => { eat!($parser, $p => {}) };
    ($parser:expr, $p:pat => $body:expr) => {
        match $parser.pop().map_err(Error::Read)? {
            Some($p) => $body,
            Some(rec) => Err(Error::UnexpectedRecord(rec.kind()))?,
            None => Err(Error::UnexpectedEof)?,
        }
    }
}

macro_rules! eat_if_matches {
    ($parser:expr, $p:pat) => { eat_if_matches!($parser, $p => {}) };
    ($parser:expr, $p:pat => $body:expr) => {
        match $parser.pop().map_err(Error::Read)? {
            Some($p)=>Some($body),
            Some(item) => {
                $parser.push(item);
                None
            }
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Parser<I, T> {
    iter: I,
    next_item: Option<T>,
}

impl<I, T, E> Parser<I, T>
where
    I: Iterator<Item = Result<T, E>>,
{
    fn new(iter: I) -> Self {
        Parser {
            iter,
            next_item: None,
        }
    }

    fn push(&mut self, item: T) {
        assert!(self.next_item.is_none());
        self.next_item = Some(item);
    }

    fn pop(&mut self) -> Result<Option<T>, E> {
        if let Some(next) = self.next_item.take() {
            return Ok(Some(next));
        }
        if let Some(item) = self.iter.next() {
            item.map(Some)
        } else {
            Ok(None)
        }
    }

    fn peek(&mut self) -> Result<Option<&T>, E> {
        if let Some(ref next) = self.next_item {
            return Ok(Some(next));
        }
        self.next_item = if let Some(item) = self.iter.next() {
            Some(item?)
        } else {
            None
        };
        Ok(self.next_item.as_ref())
    }
}


#[derive(Debug, Clone, Default)]
pub struct Merger {
    files: BTreeMap<FileKey, File>,
}

impl Merger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge<I, E>(&mut self, it: I) -> Result<(), Error<E>>
    where
        I: IntoIterator<Item = Result<Record, E>>,
    {
        let mut parser = Parser::new(it.into_iter());

        while let Some(_) = parser.peek().map_err(Error::Read)? {
            let test_name =
                eat_if_matches!(parser, Record::TestName { name } => name).unwrap_or("".into());
            let source_file = eat!(parser, Record::SourceFile { path } => path);
            let key = FileKey {
                test_name,
                source_file,
            };
            let file = self.files.entry(key).or_insert_with(Default::default);
            file.merge(&mut parser)?;
            eat!(parser, Record::EndOfRecord);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct FileKey {
    test_name: String,
    source_file: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct File {
    fn_lines: BTreeMap<String, u32>,
    fn_data: BTreeMap<String, u64>,
    br_data: BTreeMap<BranchKey, Option<u64>>,
    ln_data: BTreeMap<u32, LineData>,
}

impl File {
    fn merge<I, E>(&mut self, parser: &mut Parser<I, Record>) -> Result<(), Error<E>>
    where
        I: Iterator<Item = Result<Record, E>>,
    {
        // FunctionName
        while let Some((name, start_line)) =
            eat_if_matches!(parser, Record::FunctionName { name, start_line } => (name, start_line))
        {
            let line = *self.fn_lines.entry(name).or_insert(start_line);
            if line != start_line {
                Err(Error::UnmatchedFunctionLine)?;
            }
        }

        // FunctionData
        while let Some((name, count)) =
            eat_if_matches!(parser, Record::FunctionData { name, count } => { (name, count) })
        {
            *self.fn_data.entry(name).or_insert(0) += count;
        }

        eat_if_matches!(parser, Record::FunctionsFound { .. });
        eat_if_matches!(parser, Record::FunctionsHit { .. });

        // BranchData
        while let Some((key, taken)) = eat_if_matches!(parser,
            Record::BranchData { line, block, branch, taken } => {
                (BranchKey { line, block, branch }, taken)
            }
        ) {
            let org = self.br_data.entry(key).or_insert(None);
            if let Some(taken) = taken {
                *org = Some(org.unwrap_or(0) + taken);
            }
        }

        eat_if_matches!(parser, Record::BranchesFound { .. });
        eat_if_matches!(parser, Record::BranchesHit { .. });

        // LineData
        while let Some((line, count, checksum)) = eat_if_matches!(parser,
            Record::LineData { line, count, checksum } => {
                (line, count, checksum)
            }
        ) {
            let org = self.ln_data.entry(line).or_insert(LineData::default());
            org.count += count;
            if let Some(checksum) = checksum {
                if let Some(org_checksum) = org.checksum.as_ref() {
                    if checksum != *org_checksum {
                        Err(Error::UnmatchedChecksum)?;
                    }
                }
                org.checksum = Some(checksum);
            }
        }

        eat_if_matches!(parser, Record::LinesFound { .. });
        eat_if_matches!(parser, Record::LinesHit { .. });

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
struct BranchKey {
    line: u32,
    block: u32,
    branch: u32,
}

#[derive(Debug, Clone, Default)]
struct LineData {
    count: u64,
    checksum: Option<String>,
}

impl IntoIterator for Merger {
    type Item = Record;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

#[derive(Debug)]
pub struct IntoIter {
    state: IntoIterState,
}

impl IntoIter {
    fn new(merger: Merger) -> Self {
        IntoIter {
            state: IntoIterState::EmitTestName {
                files: merger.files.into_iter(),
            },
        }
    }
}

#[derive(Debug)]
enum IntoIterState {
    EmitTestName {
        files: btree_map::IntoIter<FileKey, File>,
    },
    EmitSourceFile {
        source_file: PathBuf,
        file: File,
        files: btree_map::IntoIter<FileKey, File>,
    },
    EmitFileBody {
        file: FileIntoIter,
        files: btree_map::IntoIter<FileKey, File>,
    },
    End,
}

impl Iterator for IntoIter {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        use self::IntoIterState::*;
        loop {
            return match mem::replace(&mut self.state, End) {
                EmitTestName { mut files } => match files.next() {
                    Some((key, file)) => {
                        self.state = EmitSourceFile {
                            source_file: key.source_file,
                            file,
                            files,
                        };
                        Some(Record::TestName {
                            name: key.test_name,
                        })
                    }
                    None => {
                        self.state = End;
                        continue;
                    }
                },

                EmitSourceFile {
                    source_file,
                    file,
                    files,
                } => {
                    self.state = EmitFileBody {
                        file: file.into_iter(),
                        files,
                    };
                    Some(Record::SourceFile { path: source_file })
                }

                EmitFileBody { mut file, files } => match file.next() {
                    Some(rec) => {
                        self.state = EmitFileBody { file, files };
                        Some(rec)
                    }
                    None => {
                        self.state = EmitTestName { files };
                        Some(Record::EndOfRecord)
                    }
                },

                End => None,
            };
        }
    }
}

impl IntoIterator for File {
    type Item = Record;
    type IntoIter = FileIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        FileIntoIter::new(self)
    }
}

#[derive(Debug)]
struct FileIntoIter {
    state: FileIntoIterState,
}

impl FileIntoIter {
    fn new(file: File) -> Self {
        FileIntoIter {
            state: FileIntoIterState::EmitFuncName {
                fn_found: 0,
                fn_lines: file.fn_lines.into_iter(),
                fn_data: file.fn_data,
                br_data: file.br_data,
                ln_data: file.ln_data,
            },
        }
    }
}

#[derive(Debug)]
enum FileIntoIterState {
    EmitFuncName {
        fn_found: u32,
        fn_lines: btree_map::IntoIter<String, u32>,
        fn_data: BTreeMap<String, u64>,
        br_data: BTreeMap<BranchKey, Option<u64>>,
        ln_data: BTreeMap<u32, LineData>,
    },
    EmitFuncData {
        fn_found: u32,
        fn_hit: u32,
        fn_data: btree_map::IntoIter<String, u64>,
        br_data: BTreeMap<BranchKey, Option<u64>>,
        ln_data: BTreeMap<u32, LineData>,
    },
    EmitFuncsFound {
        fn_found: u32,
        fn_hit: u32,
        br_data: BTreeMap<BranchKey, Option<u64>>,
        ln_data: BTreeMap<u32, LineData>,
    },
    EmitFuncsHit {
        fn_hit: u32,
        br_data: BTreeMap<BranchKey, Option<u64>>,
        ln_data: BTreeMap<u32, LineData>,
    },

    EmitBranchData {
        br_found: u32,
        br_hit: u32,
        br_data: btree_map::IntoIter<BranchKey, Option<u64>>,
        ln_data: BTreeMap<u32, LineData>,
    },
    EmitBranchesFound {
        br_found: u32,
        br_hit: u32,
        ln_data: BTreeMap<u32, LineData>,
    },
    EmitBranchesHit {
        br_hit: u32,
        ln_data: BTreeMap<u32, LineData>,
    },

    EmitLineData {
        ln_found: u32,
        ln_hit: u32,
        ln_data: btree_map::IntoIter<u32, LineData>,
    },
    EmitLinesFound {
        ln_found: u32,
        ln_hit: u32,
    },
    EmitLinesHit {
        ln_hit: u32,
    },

    End,
}

impl Iterator for FileIntoIter {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        use self::FileIntoIterState::*;
        loop {
            return match mem::replace(&mut self.state, End) {
                EmitFuncName {
                    mut fn_found,
                    mut fn_lines,
                    fn_data,
                    br_data,
                    ln_data,
                } => match fn_lines.next() {
                    Some((name, start_line)) => {
                        fn_found += 1;
                        self.state = EmitFuncName {
                            fn_found,
                            fn_lines,
                            fn_data,
                            br_data,
                            ln_data,
                        };
                        Some(Record::FunctionName { name, start_line })
                    }
                    None => {
                        self.state = EmitFuncData {
                            fn_found,
                            fn_hit: 0,
                            fn_data: fn_data.into_iter(),
                            br_data,
                            ln_data,
                        };
                        continue;
                    }
                },

                EmitFuncData {
                    fn_found,
                    mut fn_hit,
                    mut fn_data,
                    br_data,
                    ln_data,
                } => match fn_data.next() {
                    Some((name, count)) => {
                        if count > 0 {
                            fn_hit += 1;
                        }
                        self.state = EmitFuncData {
                            fn_found,
                            fn_hit,
                            fn_data,
                            br_data,
                            ln_data,
                        };
                        Some(Record::FunctionData { name, count })
                    }
                    None => {
                        self.state = EmitFuncsFound {
                            fn_found,
                            fn_hit,
                            br_data,
                            ln_data,
                        };
                        continue;
                    }
                },

                EmitFuncsFound {
                    fn_found,
                    fn_hit,
                    br_data,
                    ln_data,
                } => {
                    if fn_found == 0 {
                        debug_assert_eq!(fn_hit, 0);
                        self.state = EmitBranchData {
                            br_found: 0,
                            br_hit: 0,
                            br_data: br_data.into_iter(),
                            ln_data,
                        };
                        continue;
                    }

                    self.state = EmitFuncsHit {
                        fn_hit,
                        br_data,
                        ln_data,
                    };
                    Some(Record::FunctionsFound { found: fn_found })
                }

                EmitFuncsHit {
                    fn_hit,
                    br_data,
                    ln_data,
                } => {
                    self.state = EmitBranchData {
                        br_found: 0,
                        br_hit: 0,
                        br_data: br_data.into_iter(),
                        ln_data,
                    };
                    Some(Record::FunctionsHit { hit: fn_hit })
                }

                EmitBranchData {
                    mut br_found,
                    mut br_hit,
                    mut br_data,
                    ln_data,
                } => match br_data.next() {
                    Some((key, taken)) => {
                        br_found += 1;
                        if let Some(x) = taken {
                            if x > 0 {
                                br_hit += 1;
                            }
                        }
                        self.state = EmitBranchData {
                            br_found,
                            br_hit,
                            br_data,
                            ln_data,
                        };
                        Some(Record::BranchData {
                            line: key.line,
                            block: key.block,
                            branch: key.branch,
                            taken,
                        })
                    }
                    None => {
                        self.state = EmitBranchesFound {
                            br_found,
                            br_hit,
                            ln_data,
                        };
                        continue;
                    }
                },
                EmitBranchesFound {
                    br_found,
                    br_hit,
                    ln_data,
                } => {
                    if br_found == 0 {
                        debug_assert_eq!(br_hit, 0);
                        self.state = EmitLineData {
                            ln_found: 0,
                            ln_hit: 0,
                            ln_data: ln_data.into_iter(),
                        };
                        continue;
                    }

                    self.state = EmitBranchesHit { br_hit, ln_data };
                    Some(Record::BranchesFound { found: br_found })
                }
                EmitBranchesHit { br_hit, ln_data } => {
                    self.state = EmitLineData {
                        ln_found: 0,
                        ln_hit: 0,
                        ln_data: ln_data.into_iter(),
                    };
                    Some(Record::BranchesHit { hit: br_hit })
                }

                EmitLineData {
                    mut ln_found,
                    mut ln_hit,
                    mut ln_data,
                } => match ln_data.next() {
                    Some((line, data)) => {
                        ln_found += 1;
                        if data.count > 0 {
                            ln_hit += 1;
                        }
                        self.state = EmitLineData {
                            ln_found,
                            ln_hit,
                            ln_data,
                        };
                        Some(Record::LineData {
                            line,
                            count: data.count,
                            checksum: data.checksum,
                        })
                    }
                    None => {
                        self.state = EmitLinesFound { ln_found, ln_hit };
                        continue;
                    }
                },
                EmitLinesFound { ln_found, ln_hit } => {
                    self.state = EmitLinesHit { ln_hit };
                    Some(Record::LinesFound { found: ln_found })
                }
                EmitLinesHit { ln_hit } => {
                    self.state = End;
                    Some(Record::LinesHit { hit: ln_hit })
                }

                End => None,
            };
        }
    }
}
