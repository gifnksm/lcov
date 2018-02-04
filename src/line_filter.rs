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
        I: IntoIterator<Item = (u32, u32)>,
    {
        let file = self.files.entry(path.into()).or_insert_with(File::default);
        for range in it {
            file.add_range(range);
        }
        file.normalize();
    }

    pub fn apply(&self, report: &mut Report) {
        report.filter_map(|(key, mut sect)| {
            self.files.get(&key.source_file).and_then(|file| {
                file.apply(&mut sect);
                if !sect.is_empty() {
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
}

impl File {
    fn add_range(&mut self, (start, end): (u32, u32)) {
        if end < start {
            return;
        }
        let hend = self.start2end.entry(start).or_insert(end);
        *hend = u32::max(*hend, end);
    }

    fn normalize(&mut self) {
        let mut iter = mem::replace(&mut self.start2end, BTreeMap::new())
            .into_iter()
            .map(|(start, end)| Hunk::new(start, end));
        let mut cur_hunk = match iter.next() {
            Some(cur_hunk) => cur_hunk,
            None => return,
        };
        for hunk in iter {
            cur_hunk = cur_hunk.join(hunk).unwrap_or_else(|| {
                let _ = self.start2end.insert(cur_hunk.start, cur_hunk.end);
                hunk
            });
        }
        let _ = self.start2end.insert(cur_hunk.start, cur_hunk.end);

        debug_assert!(self.start2end.iter().all(|(s, e)| s <= e));
    }

    fn contains_range(&self, (start, end): (u32, u32)) -> bool {
        self.start2end
            .range((Bound::Unbounded, Bound::Included(end)))
            .next_back()
            .map(|(&_hstart, &hend)| hend >= start)
            .unwrap_or(false)
    }

    fn contains_line(&self, line: u32) -> bool {
        self.contains_range((line, line))
    }

    fn apply(&self, section: &mut Section) {
        section.func_list().filter_map(|(key, data)| {
            if self.contains_range((data.start_line, data.end_line)) {
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
struct Hunk {
    start: u32,
    end: u32,
}

impl Hunk {
    fn new(start: u32, end: u32) -> Self {
        assert!(start <= end);
        Hunk { start, end }
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
    use super::{File, Hunk};

    #[test]
    fn join() {
        fn check(expect: Option<(u32, u32)>, hunk1: (u32, u32), hunk2: (u32, u32)) {
            let hunk1 = Hunk::new(hunk1.0, hunk1.1);
            let hunk2 = Hunk::new(hunk2.0, hunk2.1);
            let expect = expect.map(|expect| Hunk::new(expect.0, expect.1));
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
        check(Some((0, 4)), (0, 0), (0, 4));
        check(Some((0, max)), (0, max), (100, 400));
    }

    #[test]
    fn add_range() {
        fn check(file: &File, start2end: &[(u32, u32)]) {
            assert_eq!(file.start2end, start2end.iter().cloned().collect());
        }

        let mut file = File::default();
        file.add_range((10, 10));
        check(&file, &[(10, 10)]);
        file.add_range((15, 20));
        check(&file, &[(10, 10), (15, 20)]);
        file.add_range((15, 40));
        check(&file, &[(10, 10), (15, 40)]);
        file.add_range((10, 40));
        check(&file, &[(10, 40), (15, 40)]);
        file.normalize();
        check(&file, &[(10, 40)]);

        file.add_range((50, 100));
        file.normalize();
        check(&file, &[(10, 40), (50, 100)]);
    }

    #[test]
    fn contains() {
        fn gen_file(i: u32, n: u32) -> (File, Vec<bool>) {
            let map = (0..n).map(|j| (i & (1 << j)) != 0).collect::<Vec<_>>();
            let mut file = File::default();
            for (i, &f) in map.iter().enumerate() {
                if f {
                    file.add_range((i as u32, i as u32));
                }
            }
            file.normalize();
            (file, map)
        }

        let n = 8;
        for i in 0..(2u32.pow(n)) {
            let (file, map) = gen_file(i, n);
            for start in 0..n {
                for end in start..n {
                    let res = file.contains_range((start, end));
                    let cmp = map[(start as usize)..((end + 1) as usize)]
                        .iter()
                        .any(|&f| f);
                    assert_eq!(res, cmp);
                }
            }
        }
    }
}
