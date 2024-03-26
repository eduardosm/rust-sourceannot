use std::ops::RangeInclusive;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct RangeSet<T: Copy + Ord>
where
    RangeInclusive<T>: Iterator,
{
    ranges: Vec<RangeInclusive<T>>,
}

impl<T: Copy + Ord + std::fmt::Debug> std::fmt::Debug for RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ranges.fmt(f)
    }
}

impl<T: Copy + Ord> From<T> for RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    fn from(value: T) -> Self {
        Self {
            ranges: vec![value..=value],
        }
    }
}

impl<T: Copy + Ord> From<RangeInclusive<T>> for RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    fn from(range: RangeInclusive<T>) -> Self {
        assert!(range.start() <= range.end());
        Self {
            ranges: vec![range],
        }
    }
}

impl<T: Copy + Ord> FromIterator<RangeInclusive<T>> for RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        let mut ranges = Self::new();
        for range in iter {
            ranges.insert(range);
        }
        ranges
    }
}

impl<T: Copy + Ord> Extend<RangeInclusive<T>> for RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        for range in iter {
            self.insert(range);
        }
    }
}

impl<T: Copy + Ord> Default for RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Ord> RangeSet<T>
where
    RangeInclusive<T>: Iterator,
{
    #[inline]
    pub(crate) fn new() -> Self {
        Self { ranges: Vec::new() }
    }

    #[inline]
    fn search(&self, value: &T) -> Result<usize, usize> {
        self.ranges.binary_search_by(|range| {
            if range.start() > value {
                std::cmp::Ordering::Greater
            } else if range.end() < value {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
    }

    #[inline]
    pub(crate) fn ranges(&self) -> impl Iterator<Item = RangeInclusive<T>> + '_ {
        self.ranges.iter().cloned()
    }

    pub(crate) fn insert(&mut self, new_range: RangeInclusive<T>) {
        assert!(new_range.start() <= new_range.end());
        match (self.search(new_range.start()), self.search(new_range.end())) {
            (Ok(i1), Ok(i2)) => {
                let start = *self.ranges[i1].start();
                let end = *self.ranges[i2].end();
                self.ranges[i2] = start..=end;
                self.ranges.drain(i1..i2);
            }
            (Ok(i1), Err(i2)) => {
                let fuse_next = if i2 != self.ranges.len() {
                    let next_start = self.ranges[i2].start();
                    new_range.end() >= next_start
                        || ((*new_range.end())..=(*next_start)).count() <= 2
                } else {
                    false
                };
                if fuse_next {
                    let start = *self.ranges[i1].start();
                    let end = *self.ranges[i2].end();
                    self.ranges[i2] = start..=end;
                    self.ranges.drain(i1..i2);
                } else {
                    let start = *self.ranges[i1].start();
                    let end = *new_range.end();
                    self.ranges[i1] = start..=end;
                    self.ranges.drain((i1 + 1)..i2);
                }
            }
            (Err(i1), Ok(i2)) => {
                let fuse_prev = if i1 != 0 {
                    let prev_end = self.ranges[i1 - 1].end();
                    prev_end >= new_range.start()
                        || ((*prev_end)..=(*new_range.start())).count() <= 2
                } else {
                    false
                };
                if fuse_prev {
                    let start = *self.ranges[i1 - 1].start();
                    let end = *self.ranges[i2].end();
                    self.ranges[i2] = start..=end;
                    self.ranges.drain((i1 - 1)..i2);
                } else {
                    let start = *new_range.start();
                    let end = *self.ranges[i2].end();
                    self.ranges[i2] = start..=end;
                    self.ranges.drain(i1..i2);
                }
            }
            (Err(i1), Err(i2)) => {
                let fuse_prev = if i1 != 0 {
                    let prev_end = self.ranges[i1 - 1].end();
                    prev_end >= new_range.start()
                        || ((*prev_end)..=(*new_range.start())).count() <= 2
                } else {
                    false
                };
                let fuse_next = if i2 != self.ranges.len() {
                    let next_start = self.ranges[i2].start();
                    new_range.end() >= next_start
                        || ((*new_range.end())..=(*next_start)).count() <= 2
                } else {
                    false
                };
                match (fuse_prev, fuse_next) {
                    (false, false) => {
                        self.ranges.drain(i1..i2);
                        self.ranges.insert(i1, new_range);
                    }
                    (true, false) => {
                        let start = *self.ranges[i1 - 1].start();
                        let end = *new_range.end();
                        self.ranges[i1 - 1] = start..=end;
                        self.ranges.drain(i1..i2);
                    }
                    (false, true) => {
                        let start = *new_range.start();
                        let end = *self.ranges[i2].end();
                        self.ranges[i2] = start..=end;
                        self.ranges.drain(i1..i2);
                    }
                    (true, true) => {
                        let start = *self.ranges[i1 - 1].start();
                        let end = *self.ranges[i2].end();
                        self.ranges[i1 - 1] = start..=end;
                        self.ranges.drain(i1..=i2);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RangeSet;

    #[test]
    fn test_ranges_1() {
        let mut set = RangeSet::new();
        set.insert(1..=5);
        set.insert(7..=10);
        assert_eq!(set.ranges, [1..=5, 7..=10]);

        let mut set = RangeSet::new();
        set.insert(1..=5);
        set.insert(6..=10);
        assert_eq!(set.ranges, [1..=10]);

        let mut set = RangeSet::new();
        set.insert(1..=5);
        set.insert(5..=10);
        assert_eq!(set.ranges, [1..=10]);

        let mut set = RangeSet::new();
        set.insert(1..=5);
        set.insert(4..=10);
        assert_eq!(set.ranges, [1..=10]);

        let mut set = RangeSet::new();
        set.insert(1..=5);
        set.insert(2..=4);
        assert_eq!(set.ranges, [1..=5]);
    }

    #[test]
    fn test_ranges_2() {
        let mut set = RangeSet::new();
        set.insert(5..=10);
        set.insert(0..=3);
        assert_eq!(set.ranges, [0..=3, 5..=10]);

        let mut set = RangeSet::new();
        set.insert(5..=10);
        set.insert(0..=4);
        assert_eq!(set.ranges, [0..=10]);

        let mut set = RangeSet::new();
        set.insert(5..=10);
        set.insert(0..=5);
        assert_eq!(set.ranges, [0..=10]);

        let mut set = RangeSet::new();
        set.insert(5..=10);
        set.insert(0..=6);
        assert_eq!(set.ranges, [0..=10]);
    }

    #[test]
    fn test_ranges_3() {
        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(12..=18);
        assert_eq!(set.ranges, [0..=10, 12..=18, 20..=30, 40..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(11..=18);
        assert_eq!(set.ranges, [0..=18, 20..=30, 40..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(12..=19);
        assert_eq!(set.ranges, [0..=10, 12..=30, 40..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(11..=19);
        assert_eq!(set.ranges, [0..=30, 40..=50]);
    }

    #[test]
    fn test_ranges_4() {
        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(5..=35);
        assert_eq!(set.ranges, [0..=35, 40..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(5..=39);
        assert_eq!(set.ranges, [0..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(5..=45);
        assert_eq!(set.ranges, [0..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(11..=35);
        assert_eq!(set.ranges, [0..=35, 40..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(11..=39);
        assert_eq!(set.ranges, [0..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(11..=45);
        assert_eq!(set.ranges, [0..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(15..=35);
        assert_eq!(set.ranges, [0..=10, 15..=35, 40..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(15..=39);
        assert_eq!(set.ranges, [0..=10, 15..=50]);

        let mut set = RangeSet::new();
        set.insert(0..=10);
        set.insert(20..=30);
        set.insert(40..=50);
        set.insert(15..=45);
        assert_eq!(set.ranges, [0..=10, 15..=50]);
    }
}
