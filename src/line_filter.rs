use super::report::Report;
use super::report::section::Section;
use std::collections::{BTreeMap, Bound, HashMap};
use std::mem;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Filter {
    files: HashMap<PathBuf, File>,
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<P, I>(&mut self, path: P, it: I)
    where
        P: Into<PathBuf>,
        I: IntoIterator<Item = LineHunk>,
    {
        let file = self.files.entry(path.into()).or_insert_with(File::default);
        for hunk in it {
            file.add_hunk(hunk);
        }
        file.normalize();
    }

    pub fn execute(&self, report: &mut Report) {
        report.filter_map(|(key, mut sect)| {
            self.files.get(&key.source_file).and_then(|file| {
                file.execute(&mut sect);
                if sect.is_empty() {
                    Some((key, sect))
                } else {
                    None
                }
            })
        })
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct File {
    start2end: BTreeMap<u32, u32>,
    end2start: BTreeMap<u32, u32>,
}

impl File {
    fn add_hunk(&mut self, hunk: LineHunk) {
        let end = self.start2end.entry(hunk.start).or_insert(hunk.end);
        if *end < hunk.end {
            *end = hunk.end;
        }
        let start = self.end2start.entry(hunk.end).or_insert(hunk.start);
        if *start > hunk.start {
            *start = hunk.start;
        }
    }

    fn normalize(&mut self) {
        fn exec<F, I>(map: &mut BTreeMap<u32, u32>, conv_from: F, mut conv_into: I)
        where
            F: FnMut((u32, u32)) -> LineHunk,
            I: FnMut(LineHunk) -> (u32, u32),
        {
            let mut iter = mem::replace(map, BTreeMap::new())
                .into_iter()
                .map(conv_from);
            let mut cur_hunk = match iter.next() {
                Some(cur_hunk) => cur_hunk,
                None => return,
            };
            for hunk in iter {
                cur_hunk = cur_hunk.join(hunk).unwrap_or_else(|| {
                    let (k, v) = conv_into(cur_hunk);
                    let _ = map.insert(k, v);
                    hunk
                });
            }
            let (k, v) = conv_into(cur_hunk);
            let _ = map.insert(k, v);
        }

        exec(
            &mut self.start2end,
            |(start, end)| LineHunk::new(start, end),
            |hunk| (hunk.start, hunk.end),
        );
        exec(
            &mut self.end2start,
            |(end, start)| LineHunk::new(start, end),
            |hunk| (hunk.end, hunk.start),
        )
    }


    fn contains_range(&self, start: u32, end: u32) -> bool {
        if let Some((&cend, &cstart)) = self.end2start.range(start..).next() {
            debug_assert!(cend >= start);
            if cstart <= end {
                return true;
            }
        }
        self.start2end
            .range((Bound::Included(start), Bound::Included(end)))
            .next()
            .is_some()
    }

    fn contains_line(&self, line: u32) -> bool {
        if let Some((&cend, &cstart)) = self.end2start.range(line..).next() {
            debug_assert!(cend >= line);
            return cstart <= line;
        }
        false
    }

    fn execute(&self, section: &mut Section) {
        section.func_list().filter_map(|(key, data)| {
            if self.contains_range(data.start_line, data.end_line) {
                Some((key, data))
            } else {
                None
            }
        });
        section.branch_list().filter_map(|(key, data)| {
            if self.contains_line(key.line) {
                Some((key, data))
            } else {
                None
            }
        });
        section.line_list().filter_map(|(key, data)| {
            if self.contains_line(key.line) {
                Some((key, data))
            } else {
                None
            }
        });
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct LineHunk {
    start: u32,
    end: u32,
}

impl LineHunk {
    pub fn new(start: u32, end: u32) -> Self {
        if end < start {
            panic!()
        }
        LineHunk { start, end }
    }

    fn join(self, other: Self) -> Option<Self> {
        if u32::saturating_add(other.end, 1) < self.start
            || u32::saturating_add(self.end, 1) < other.start
        {
            return None;
        }
        let start = u32::min(self.start, other.start);
        let end = u32::max(self.end, other.end);
        Some(Self::new(start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::LineHunk;

    #[test]
    fn join() {
        fn check(expect: Option<(u32, u32)>, hunk1: (u32, u32), hunk2: (u32, u32)) {
            let hunk1 = LineHunk::new(hunk1.0, hunk1.1);
            let hunk2 = LineHunk::new(hunk2.0, hunk2.1);
            let expect = expect.map(|expect| LineHunk::new(expect.0, expect.1));
            assert_eq!(expect, hunk1.join(hunk2));
            assert_eq!(expect, hunk2.join(hunk1));
            assert_eq!(Some(hunk1), hunk1.join(hunk1));
            assert_eq!(Some(hunk2), hunk2.join(hunk2));
        }
        let max = u32::max_value();
        check(Some((0, 3)), (0, 1), (1, 3));
        check(Some((0, 3)), (0, 1), (2, 3));
        check(Some((0, 3)), (0, 2), (1, 3));
        check(Some((0, 3)), (0, 3), (1, 2));
        check(Some((0, max)), (0, max), (100, 400));
    }
}
