#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

use heapless::Vec;

#[derive(Debug, Eq, PartialEq)]
pub struct Series<const N : usize, I, T> {
    pub max_deviation: T,
    pub buckets: Vec<SerieEntry<I, T>, N>
}

impl <const N : usize, I : Clone, T : Clone> Clone for Series<N, I, T> {
    fn clone(&self) -> Self {
        Series { max_deviation: self.max_deviation.clone(), buckets: self.buckets.clone() }
    }
}

impl <const N : usize, I : Ord, T : Deviate> Series<N, I, T> {
    pub const fn new(max_deviation: T) -> Series<N, I, T> {
        Series { max_deviation, buckets: Vec::new() }
    }

    pub fn append_monotonic(&mut self, at: I, value: T) -> bool {
        if *&self.buckets.is_full() {
            return false
        } else {
            return match self.buckets.pop() {
                Some(v) => {
                    let gt_start = &at > &v.range.start;
                    let gt_end = &v.range.end.as_ref().map(|x| &at > x).unwrap_or(true);

                    if gt_start && *gt_end {
                        if v.value.deviate(&value, &self.max_deviation) {
                            self.buckets.push(v);
                            self.buckets.push(SerieEntry { range: Range::new(at), value });
                        } else {
                            let new_range = v.range.extend(at);
                            self.buckets.push(SerieEntry { range: new_range, value: v.value });
                        }
                        true
                    } else {
                        self.buckets.push(v);
                        false
                    }
                },
                None => {
                    self.buckets.push(SerieEntry { range: Range::new(at), value });
                    true
                }
            }
        }
    }

    pub fn starts_at(&self) -> Option<&I> {
        self.buckets.first().map(|x| &x.range.start)
    }

    pub fn ends_at(&self) -> Option<&I> {
        let mut end: Option<&I> = None;

        for b in &self.buckets {
            end = Some(&b.range.start).max(end);
            end = b.range.end.as_ref().max(end);
        }

        return end
    }

    pub fn is_full(&self) -> bool {
        return *&self.buckets.is_full()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Range<I> {
    pub start: I,
    pub end: Option<I>
}

impl <I: Clone> Clone for Range<I> {
    fn clone(&self) -> Self {
        Range { start: self.start.clone(), end: self.end.clone() }
    }
}

impl <I: Ord + Sized> Range<I> {
    pub fn new(start: I) -> Range<I> {
        Range { start, end: None }
    }

    pub fn extend(self, value: I) -> Range<I> {
        Range {
            start: self.start,
            end: Some(value)
        }
    }
}

pub trait Deviate {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool;
}

impl Deviate for f32 {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        self - other > *max_deviation
    }
}

impl Deviate for f64 {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        self - other > *max_deviation
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SerieEntry<I, T> {
    pub range: Range<I>,
    pub value: T
}

impl <I: Clone, T: Clone> Clone for SerieEntry<I, T> {
    fn clone(&self) -> Self {
        SerieEntry { range: self.range.clone(), value: self.value.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(timeseries.append_monotonic( 2_u8, 32.7f32));
        assert!(timeseries.append_monotonic( 3_u8, 32.5f32));
        assert!(timeseries.append_monotonic( 4_u8, 33.8f32));
        assert!(timeseries.append_monotonic( 6_u8, 34.0f32));
        assert!(timeseries.append_monotonic( 8_u8, 28.2f32));
        assert!(timeseries.append_monotonic( 10_u8, 12.3f32));

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry { range: Range { start: 1, end: Some(6) }, value: 32.6 });
        expected.push(SerieEntry { range: Range { start: 8, end: None }, value: 28.2 });
        expected.push(SerieEntry { range: Range { start: 10, end: None }, value: 12.3 });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn should_not_append_when_full() {
        let mut timeseries: Series<1, u8, f32> = Series::new(0.3);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(!timeseries.append_monotonic( 2_u8, 32.7f32));
        assert!(timeseries.is_full());

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry { range: Range { start: 1, end: None }, value: 32.6 });

        assert_eq!(timeseries.buckets, expected)
    }


    #[test]
    fn should_return_false_when_append_is_not_monotonic_with_end() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(timeseries.append_monotonic( 6_u8, 32.7f32));
        assert!(!timeseries.append_monotonic( 1_u8, 32.5f32));

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry { range: Range { start: 1, end: Some(6) }, value: 32.6 });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn should_return_false_when_append_is_not_monotonic_without_end() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(timeseries.append_monotonic( 6_u8, 35.7f32));
        assert!(!timeseries.append_monotonic( 1_u8, 32.5f32));

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry { range: Range { start: 1, end: Some(6) }, value: 32.6 });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn starts_at_some() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(timeseries.append_monotonic( 2_u8, 32.7f32));

        assert_eq!(timeseries.starts_at(), Some(&1_u8));
    }

    #[test]
    fn starts_at_none() {
        let timeseries: Series<1, u8, f32> = Series::new(0.3f32);
        assert_eq!(timeseries.starts_at(), None);
    }

    #[test]
    fn ends_at_some_range_end() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(timeseries.append_monotonic( 2_u8, 32.7f32));

        assert_eq!(timeseries.ends_at(), Some(&2_u8));
    }

    #[test]
    fn ends_at_some_range_start() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic( 1_u8, 32.6f32));
        assert!(timeseries.append_monotonic( 2_u8, 35.7f32));

        assert_eq!(timeseries.ends_at(), Some(&2_u8));
    }

    #[test]
    fn ends_at_none() {
        let timeseries: Series<1, u8, f32> = Series::new(0.3f32);
        assert_eq!(timeseries.ends_at(), None);
    }
}
