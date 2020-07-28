//! Filters for a LCOV report.
use std::iter::{Extend, IntoIterator};
use std::mem;

pub mod line_num;

pub use self::line_num::LineNum;

/// Filters elements of the collection in-place.
///
/// # Examples
///
/// ```rust
/// use lcov::filter::FilterMap;
///
/// let mut v = vec![1, 3, 2, 4, 1, 9, 30];
/// v.filter_map(|n| {
///     if n % 2 == 0 {
///         Some(n / 2)
///     } else {
///         None
///     }
/// });
/// assert_eq!(v, &[1, 2, 15]);
/// ```
pub trait FilterMap {
    /// The type of the elements.
    type Item;

    /// Filters elements of `self` in-place.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lcov::filter::FilterMap;
    ///
    /// let mut v = vec![1, 3, 2, 4, 1, 9, 30];
    /// v.filter_map(|n| {
    ///     if n % 2 == 0 {
    ///         Some(n / 2)
    ///     } else {
    ///         None
    ///     }
    /// });
    /// assert_eq!(v, &[1, 2, 15]);
    fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut(Self::Item) -> Option<Self::Item>;
}

impl<T, I> FilterMap for T
where
    T: Default + Extend<I> + IntoIterator<Item = I>,
{
    type Item = T::Item;

    fn filter_map<F>(&mut self, f: F)
    where
        F: FnMut(Self::Item) -> Option<Self::Item>,
    {
        let iter = mem::take(self).into_iter().filter_map(f);
        self.extend(iter);
    }
}
