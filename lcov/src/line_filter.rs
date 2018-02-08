use super::report::Report;
use super::report::section::Section;
use std::{mem, ops};
use std::collections::{BTreeMap, Bound, HashMap};
use std::path::PathBuf;

/// A [`Report`] filter that extracts only the records related to the specified line.
///
/// This filter is useful for measuring the coverage of the part changed by a specific commit.
///
/// # Examples
///
/// ```rust
/// # extern crate failure;
/// # extern crate lcov;
/// # use failure::Error;
/// use lcov::{LineFilter, Report, Reader};
/// use std::fs::File;
/// use std::io::BufReader;
///
/// # fn foo() -> Result<(), Error> {
/// // Creates a `Report` from file.
/// let mut report = Report::new();
/// let reader = Reader::new(BufReader::new(File::open("report.info")?));
/// report.merge(reader)?;
///
/// // Setup the filter.
/// let mut filter = LineFilter::new();
/// filter.insert("foo.rs", [0..5, 10..20].iter().cloned());
///
/// // Filters the coverage information.
/// filter.apply(&mut report);
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
///
/// [`Report`]: struct.Report.html
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Filter {
    files: HashMap<PathBuf, File>,
}

impl Filter {
    /// Creates an empty filter.
    ///
    /// An empty filter filters out all records.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # extern crate failure;
    /// # extern crate lcov;
    /// # use failure::Error;
    /// use lcov::{LineFilter, Report, Reader};
    /// use std::fs::File;
    /// use std::io::BufReader;
    ///
    /// # fn try_main() -> Result<(), Error> {
    /// // Creates a `Report` from file.
    /// let mut report = Report::new();
    /// let input = "\
    /// TN:test_name
    /// SF:/path/to/source/file.rs
    /// DA:10,10
    /// DA:20,10
    /// DA:30,0
    /// DA:40,0
    /// LF:4
    /// LH:2
    /// end_of_record
    /// ";
    /// let reader = Reader::new(input.as_bytes());
    /// report.merge(reader)?;
    ///
    /// // Applies an empty filter.
    /// LineFilter::new().apply(&mut report);
    ///
    /// // No records returned.
    /// assert_eq!(report.into_iter().next(), None);
    /// # Ok(())
    /// # }
    /// # fn main() {
    /// # try_main().expect("failed to run test");
    /// # }
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the ranges for the `path`.
    pub fn insert<P, I, R>(&mut self, path: P, it: I)
    where
        P: Into<PathBuf>,
        I: IntoIterator<Item = R>,
        R: Into<Range>,
    {
        let file = self.files.entry(path.into()).or_insert_with(File::default);
        for range in it {
            file.add_range(range);
        }
        file.normalize();
    }

    /// Applies the filter to `report`.
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
    fn add_range<R>(&mut self, range: R)
    where
        R: Into<Range>,
    {
        let range = range.into();
        if !range.is_valid() {
            return;
        }
        let rend = self.start2end.entry(range.start).or_insert(range.end);
        *rend = u32::max(*rend, range.end);
    }

    fn normalize(&mut self) {
        let mut iter = mem::replace(&mut self.start2end, BTreeMap::new())
            .into_iter()
            .map(|(start, end)| Range::new(start, end));
        let mut cur_range = match iter.next() {
            Some(cur_range) => cur_range,
            None => return,
        };
        for range in iter {
            cur_range = cur_range.join(range).unwrap_or_else(|| {
                let _ = self.start2end.insert(cur_range.start, cur_range.end);
                range
            });
        }
        let _ = self.start2end.insert(cur_range.start, cur_range.end);

        debug_assert!(self.start2end.iter().all(|(s, e)| s <= e));
    }

    fn contains_range<R>(&self, range: R) -> bool
    where
        R: Into<Range>,
    {
        let range = range.into();
        self.start2end
            .range((Bound::Unbounded, Bound::Included(range.end)))
            .next_back()
            .map(|(&_start, &end)| end >= range.start)
            .unwrap_or(false)
    }

    fn contains_line(&self, line: u32) -> bool {
        self.contains_range(Range::new(line, line))
    }

    fn apply(&self, section: &mut Section) {
        section.func_list().filter_map(|(key, data)| {
            if self.contains_range(Range::new(data.start_line, data.end_line)) {
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

/// An range of lines.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
pub struct Range {
    start: u32,
    end: u32,
}

impl From<ops::Range<u32>> for Range {
    fn from(range: ops::Range<u32>) -> Self {
        Range::new(range.start, u32::saturating_sub(range.end, 1))
    }
}

impl From<ops::RangeFrom<u32>> for Range {
    fn from(range: ops::RangeFrom<u32>) -> Self {
        Range::new(range.start, u32::max_value())
    }
}

impl From<ops::RangeTo<u32>> for Range {
    fn from(range: ops::RangeTo<u32>) -> Self {
        Range::new(0, u32::saturating_sub(range.end, 1))
    }
}

impl From<ops::RangeFull> for Range {
    fn from(_: ops::RangeFull) -> Self {
        Range::new(0, u32::max_value())
    }
}

impl Range {
    fn new(start: u32, end: u32) -> Self {
        Range { start, end }
    }

    fn is_valid(&self) -> bool {
        self.start <= self.end
    }

    fn join(self, other: Self) -> Option<Self> {
        if !self.is_valid() {
            return Some(other);
        }
        if !other.is_valid() {
            return Some(self);
        }

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
    use super::{File, Range};

    #[test]
    fn join() {
        fn check(expect: Option<(u32, u32)>, range1: (u32, u32), range2: (u32, u32)) {
            let range1 = Range::new(range1.0, range1.1);
            let range2 = Range::new(range2.0, range2.1);
            let expect = expect.map(|expect| Range::new(expect.0, expect.1));
            assert_eq!(expect, range1.join(range2));
            assert_eq!(expect, range2.join(range1));
            assert_eq!(Some(range1), range1.join(range1));
            assert_eq!(Some(range2), range2.join(range2));
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
        file.add_range(Range::new(10, 10));
        check(&file, &[(10, 10)]);
        file.add_range(Range::new(15, 20));
        check(&file, &[(10, 10), (15, 20)]);
        file.add_range(Range::new(15, 40));
        check(&file, &[(10, 10), (15, 40)]);
        file.add_range(Range::new(10, 40));
        check(&file, &[(10, 40), (15, 40)]);
        file.normalize();
        check(&file, &[(10, 40)]);

        file.add_range(Range::new(50, 100));
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
                    file.add_range(Range::new(i as u32, i as u32));
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
                    let res = file.contains_range(Range::new(start, end));
                    let cmp = map[(start as usize)..((end + 1) as usize)]
                        .iter()
                        .any(|&f| f);
                    assert_eq!(res, cmp);
                }
            }
        }
    }
}
