//! A [`Section`] filter that extracts only the records related to the specified line numbers.
//!
//! See [`LineNum`] documentation for more.
//!
//! [`Section`]: ../../report/section/index.html
//! [`LineNum`]: struct.LineNum.html
use super::FilterMap;
use crate::report::section;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, Bound};
use std::iter::{self, Extend, FromIterator};
use std::{mem, ops};

/// A [`Section`] filter that extracts only the records related to the specified line numbers.
///
/// This filter is useful for measuring the coverage of the part changed by a specific commit.
///
/// # Examples
///
/// ```rust
/// use lcov::Report;
/// use lcov::filter::{FilterMap, LineNum};
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use std::iter::FromIterator;
///
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// // Creates a `Report` from file.
/// let mut report = Report::from_file("report.info")?;
///
/// // Setup the filter.
/// let mut filter = HashMap::new();
/// filter.insert(
///     PathBuf::from("foo.rs"),
///     LineNum::from_iter([0..5, 10..20].iter().cloned())
/// );
///
/// // Filters the coverage information.
/// report.sections.filter_map(|(key, mut value)| {
///     filter.get(&key.source_file).and_then(|filter| {
///         filter.apply(&mut value);
///         if value.is_empty() { None } else { Some((key, value)) }
///     })
/// });
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
///
/// [`Section`]: ../../report/section/index.html
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct LineNum {
    start2end: BTreeMap<u32, u32>,
}

impl LineNum {
    /// Creates an empty filter.
    ///
    /// An empty filter filters out all records.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::filter::line_num::LineNum;
    ///
    /// let filter = LineNum::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a range of lines that those coverage information should be yielded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::filter::line_num::LineNum;
    ///
    /// let mut filter = LineNum::new();
    ///
    /// filter.insert(3..4);
    /// filter.insert(7..10);
    /// ```
    pub fn insert<R>(&mut self, range: R)
    where
        R: Into<Range>,
    {
        self.extend(iter::once(range));
    }

    /// Applies the filter to `section`.
    /// # Examples
    ///
    /// ```rust
    /// use lcov::Report;
    /// use lcov::filter::{FilterMap, LineNum};
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// use std::iter::FromIterator;
    ///
    /// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// // Creates a `Report` from file.
    /// let mut report = Report::from_file("report.info")?;
    ///
    /// // Setup the filter.
    /// let mut filter = HashMap::new();
    /// filter.insert(
    ///     PathBuf::from("foo.rs"),
    ///     LineNum::from_iter([0..5, 10..20].iter().cloned())
    /// );
    ///
    /// // Filters the coverage information.
    /// report.sections.filter_map(|(key, mut value)| {
    ///     filter.get(&key.source_file).and_then(|filter| {
    ///         filter.apply(&mut value);
    ///         if value.is_empty() { None } else { Some((key, value)) }
    ///     })
    /// });
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    pub fn apply(&self, section: &mut section::Value) {
        let mut functions = mem::take(&mut section.functions)
            .into_iter()
            .filter_map(|(key, value)| {
                value
                    .start_line
                    .map(|start_line| (start_line, 0, key, value))
            })
            .collect::<Vec<_>>();
        functions.sort_by_key(|&(start_line, _, _, _)| start_line);
        {
            let mut end = u32::max_value();
            for data in functions.iter_mut().rev() {
                data.1 = end;
                end = u32::saturating_sub(data.0, 1);
            }
        }
        let functions = functions
            .into_iter()
            .filter_map(|(start_line, end_line, key, value)| {
                if self.contains(Range::new(start_line, end_line)) {
                    Some((key, value))
                } else {
                    None
                }
            });
        section.functions.extend(functions);

        section.branches.filter_map(|(key, value)| {
            if self.contains(Range::from_line(key.line)) {
                Some((key, value))
            } else {
                None
            }
        });
        section.lines.filter_map(|(key, value)| {
            if self.contains(Range::from_line(key.line)) {
                Some((key, value))
            } else {
                None
            }
        });
    }

    fn contains<R>(&self, range: R) -> bool
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

    fn normalize(&mut self) {
        let mut iter = mem::take(&mut self.start2end)
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
}

impl<R> FromIterator<R> for LineNum
where
    R: Into<Range>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = R>,
    {
        let mut ranges = Self::new();
        ranges.extend(iter);
        ranges.normalize();
        ranges
    }
}

impl<R> Extend<R> for LineNum
where
    R: Into<Range>,
{
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = R>,
    {
        let iter = iter.into_iter().map(R::into).filter(Range::is_valid);
        for range in iter {
            match self.start2end.entry(range.start) {
                Entry::Vacant(e) => {
                    let _ = e.insert(range.end);
                }
                Entry::Occupied(mut e) => {
                    let rend = e.get_mut();
                    *rend = u32::max(*rend, range.end);
                }
            }
        }
        self.normalize();
    }
}

/// A range of lines.
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

    /// Creates a range which contains one line.
    pub fn from_line(line: u32) -> Self {
        Range {
            start: line,
            end: line,
        }
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
    use super::{LineNum, Range};

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
    fn insert() {
        fn check(file: &LineNum, start2end: &[(u32, u32)]) {
            assert_eq!(file.start2end, start2end.iter().cloned().collect());
        }

        let mut file = LineNum::default();
        file.insert(Range::new(10, 10));
        check(&file, &[(10, 10)]);
        file.insert(Range::new(15, 20));
        check(&file, &[(10, 10), (15, 20)]);
        file.insert(Range::new(15, 40));
        check(&file, &[(10, 10), (15, 40)]);
        file.insert(Range::new(10, 40));
        check(&file, &[(10, 40)]);
        file.insert(Range::new(50, 100));
        check(&file, &[(10, 40), (50, 100)]);
    }

    #[test]
    fn contains() {
        fn gen_file(i: u32, n: u32) -> (LineNum, Vec<bool>) {
            let map = (0..n).map(|j| (i & (1 << j)) != 0).collect::<Vec<_>>();
            let mut file = LineNum::default();
            for (i, &f) in map.iter().enumerate() {
                if f {
                    file.insert(Range::new(i as u32, i as u32));
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
                    let res = file.contains(Range::new(start, end));
                    let cmp = map[(start as usize)..((end + 1) as usize)]
                        .iter()
                        .any(|&f| f);
                    assert_eq!(res, cmp);
                }
            }
        }
    }
}
