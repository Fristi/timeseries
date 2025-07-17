#![deny(unsafe_code)]
#![cfg_attr(not(test), no_std)]

//! A custom deviation-based time series compression library for `no_std` environments.
//!
//! This library implements a compression algorithm similar to Swinging Door Compression,
//! designed for embedded systems and constrained environments with predictable memory usage.
//!
//! # Features
//!
//! - **No-std compatible**: Works in embedded and constrained environments
//! - **Memory safe**: Uses `#![deny(unsafe_code)]` for guaranteed memory safety
//! - **Fixed capacity**: Based on `heapless::Vec` for predictable memory usage
//! - **Monotonic timestamps**: Points must be added in strictly increasing order
//! - **Deviation-based compression**: Only stores values that deviate significantly
//!
//! # Example
//!
//! ```rust
//! use timeseries::Series;
//!
//! // Create a series with capacity for 10 entries, max deviation of 0.3
//! let mut timeseries: Series<10, u8, f32> = Series::new(0.3);
//!
//! // Add monotonic data points
//! assert!(timeseries.append_monotonic(1, 32.6));
//! assert!(timeseries.append_monotonic(2, 32.7)); // Within deviation, extends range
//! assert!(timeseries.append_monotonic(4, 33.8)); // Exceeds deviation, new entry
//! ```

use heapless::Vec;

/// A time series data structure that compresses data points using deviation-based compression.
///
/// The `Series` stores data points in compressed segments called "buckets". When a new data point
/// is within the allowed deviation from the last stored value, it extends the time range of that
/// bucket instead of creating a new one. This provides efficient compression while maintaining
/// a bounded deviation from the original data.
///
/// # Type Parameters
///
/// * `N` - Maximum number of compressed segments (const generic for compile-time sizing)
/// * `I` - Index/timestamp type (must implement `Ord + Clone`)
/// * `T` - Value type (must implement `Deviate + Clone`)
///
/// # Examples
///
/// ```rust
/// use timeseries::Series;
///
/// let mut series: Series<10, u8, f32> = Series::new(0.5);
/// assert!(series.append_monotonic(1, 10.0));
/// assert!(series.append_monotonic(2, 10.3)); // Within deviation
/// assert!(series.append_monotonic(3, 11.0)); // Exceeds deviation, new bucket
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Series<const N: usize, I, T> {
    /// Maximum allowed deviation between values before creating a new bucket
    pub max_deviation: T,
    /// Compressed data segments, each representing a time range with a representative value
    pub buckets: Vec<SerieEntry<I, T>, N>,
}

impl<const N: usize, I: Clone, T: Clone> Clone for Series<N, I, T> {
    fn clone(&self) -> Self {
        Series {
            max_deviation: self.max_deviation.clone(),
            buckets: self.buckets.clone(),
        }
    }
}

impl<const N: usize, I: Ord, T: Deviate> Series<N, I, T> {
    /// Creates a new time series with the specified maximum deviation threshold.
    ///
    /// # Parameters
    ///
    /// * `max_deviation` - The maximum allowed deviation between values before creating a new bucket
    ///
    /// # Returns
    ///
    /// A new empty `Series` instance with the given deviation threshold
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Series;
    ///
    /// let series: Series<10, u8, f32> = Series::new(0.5);
    /// assert_eq!(series.max_deviation, 0.5);
    /// ```
    pub const fn new(max_deviation: T) -> Series<N, I, T> {
        Series {
            max_deviation,
            buckets: Vec::new(),
        }
    }

    /// Appends a new data point to the series if it maintains monotonic ordering.
    ///
    /// This method enforces that timestamps must be strictly increasing. If the new value
    /// is within the allowed deviation from the last stored value, it extends the time range
    /// of the current bucket. Otherwise, it creates a new bucket.
    ///
    /// # Parameters
    ///
    /// * `at` - The timestamp/index for the new data point (must be greater than previous)
    /// * `value` - The value to store at this timestamp
    ///
    /// # Returns
    ///
    /// * `true` - If the point was successfully added or merged with existing data
    /// * `false` - If the point was rejected due to:
    ///   - Non-monotonic timestamp (not greater than previous)
    ///   - Series is at full capacity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Series;
    ///
    /// let mut series: Series<5, u8, f32> = Series::new(0.3);
    ///
    /// assert!(series.append_monotonic(1, 10.0));  // First point
    /// assert!(series.append_monotonic(2, 10.2));  // Within deviation, extends range
    /// assert!(series.append_monotonic(3, 10.8));  // Exceeds deviation, new bucket
    /// assert!(!series.append_monotonic(2, 9.0));  // Non-monotonic, rejected
    /// ```
    pub fn append_monotonic(&mut self, at: I, value: T) -> bool {
        if *&self.buckets.is_full() {
            return false;
        } else {
            return match self.buckets.pop() {
                Some(v) => {
                    let gt_start = &at > &v.range.start;
                    let gt_end = &v.range.end.as_ref().map(|x| &at > x).unwrap_or(true);

                    if gt_start && *gt_end {
                        if v.value.deviate(&value, &self.max_deviation) {
                            let _ = self.buckets.push(v);
                            let _ = self.buckets.push(SerieEntry {
                                range: Range::new(at),
                                value,
                            });
                        } else {
                            let new_range = v.range.extend(at);
                            let _ = self.buckets.push(SerieEntry {
                                range: new_range,
                                value: v.value,
                            });
                        }
                        true
                    } else {
                        let _ = self.buckets.push(v);
                        false
                    }
                }
                None => {
                    let _ = self.buckets.push(SerieEntry {
                        range: Range::new(at),
                        value,
                    });
                    true
                }
            };
        }
    }

    /// Returns the earliest timestamp in the series.
    ///
    /// # Returns
    ///
    /// * `Some(&I)` - Reference to the earliest timestamp if the series contains data
    /// * `None` - If the series is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Series;
    ///
    /// let mut series: Series<5, u8, f32> = Series::new(0.3);
    /// assert_eq!(series.starts_at(), None);
    ///
    /// series.append_monotonic(5, 10.0);
    /// series.append_monotonic(8, 10.2);
    /// assert_eq!(series.starts_at(), Some(&5));
    /// ```
    pub fn starts_at(&self) -> Option<&I> {
        self.buckets.first().map(|x| &x.range.start)
    }

    /// Returns the latest timestamp in the series.
    ///
    /// This method finds the maximum timestamp across all buckets, considering both
    /// the start timestamps and end timestamps of ranges.
    ///
    /// # Returns
    ///
    /// * `Some(&I)` - Reference to the latest timestamp if the series contains data
    /// * `None` - If the series is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Series;
    ///
    /// let mut series: Series<5, u8, f32> = Series::new(0.3);
    /// assert_eq!(series.ends_at(), None);
    ///
    /// series.append_monotonic(5, 10.0);
    /// series.append_monotonic(8, 10.2);  // Extends range to 8
    /// assert_eq!(series.ends_at(), Some(&8));
    /// ```
    pub fn ends_at(&self) -> Option<&I> {
        let mut end: Option<&I> = None;

        for b in &self.buckets {
            end = Some(&b.range.start).max(end);
            end = b.range.end.as_ref().max(end);
        }

        return end;
    }

    /// Checks if the series has reached its maximum capacity.
    ///
    /// # Returns
    ///
    /// * `true` - If the series cannot accept any more buckets
    /// * `false` - If there is still capacity for more data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Series;
    ///
    /// let mut series: Series<2, u8, f32> = Series::new(0.3);
    /// assert!(!series.is_full());
    ///
    /// series.append_monotonic(1, 10.0);
    /// series.append_monotonic(2, 15.0);  // Creates second bucket due to deviation
    /// assert!(series.is_full());
    /// ```
    pub fn is_full(&self) -> bool {
        return *&self.buckets.is_full();
    }
}

/// Represents a time range with a start timestamp and optional end timestamp.
///
/// A `Range` can represent either a single point in time (when `end` is `None`) or
/// a time interval spanning from `start` to `end` (when `end` is `Some`).
///
/// # Type Parameters
///
/// * `I` - The timestamp/index type (must implement `Ord + Clone`)
///
/// # Examples
///
/// ```rust
/// use timeseries::Range;
///
/// // Single point in time
/// let point = Range::new(5);
/// assert_eq!(point.start, 5);
/// assert_eq!(point.end, None);
///
/// // Time interval
/// let interval = point.extend(10);
/// assert_eq!(interval.start, 5);
/// assert_eq!(interval.end, Some(10));
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Range<I> {
    /// The starting timestamp of the range
    pub start: I,
    /// The optional ending timestamp of the range
    pub end: Option<I>,
}

impl<I: Clone> Clone for Range<I> {
    fn clone(&self) -> Self {
        Range {
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl<I: Ord + Sized> Range<I> {
    /// Creates a new range starting at the given timestamp with no end time.
    ///
    /// # Parameters
    ///
    /// * `start` - The starting timestamp for this range
    ///
    /// # Returns
    ///
    /// A new `Range` instance representing a single point in time
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Range;
    ///
    /// let range = Range::new(42);
    /// assert_eq!(range.start, 42);
    /// assert_eq!(range.end, None);
    /// ```
    pub fn new(start: I) -> Range<I> {
        Range { start, end: None }
    }

    /// Extends this range to include the given end timestamp.
    ///
    /// This method consumes the original range and returns a new range that spans
    /// from the original start time to the provided end time.
    ///
    /// # Parameters
    ///
    /// * `value` - The ending timestamp for this range
    ///
    /// # Returns
    ///
    /// A new `Range` instance representing a time interval
    ///
    /// # Examples
    ///
    /// ```rust
    /// use timeseries::Range;
    ///
    /// let point = Range::new(10);
    /// let interval = point.extend(20);
    ///
    /// assert_eq!(interval.start, 10);
    /// assert_eq!(interval.end, Some(20));
    /// ```
    pub fn extend(self, value: I) -> Range<I> {
        Range {
            start: self.start,
            end: Some(value),
        }
    }
}

/// Trait for determining if two values deviate beyond a specified threshold.
///
/// This trait is used by the time series compression algorithm to decide whether
/// a new data point should extend an existing bucket or create a new one.
///
/// # Examples
///
/// ```rust
/// use timeseries::Deviate;
///
/// let value1 = 10.0f32;
/// let value2 = 10.5f32;
/// let threshold = 0.3f32;
///
/// assert!(value2.deviate(&value1, &threshold)); // 0.5 > 0.3, so it deviates
/// ```
pub trait Deviate {
    /// Determines if this value deviates from another by more than the maximum allowed deviation.
    ///
    /// # Parameters
    ///
    /// * `other` - The reference value to compare against
    /// * `max_deviation` - The maximum allowed deviation threshold
    ///
    /// # Returns
    ///
    /// * `true` - If the absolute difference exceeds the maximum deviation
    /// * `false` - If the values are within the allowed deviation range
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool;
}

impl Deviate for f32 {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        (self - other).abs() > *max_deviation
    }
}

impl Deviate for f64 {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        (self - other).abs() > *max_deviation
    }
}

/// Represents a compressed data segment in the time series.
///
/// A `SerieEntry` contains a time range and a representative value for that range.
/// This is the fundamental unit of compression in the time series, where multiple
/// data points within the deviation threshold are represented by a single entry.
///
/// # Type Parameters
///
/// * `I` - The timestamp/index type (must implement `Ord + Clone`)
/// * `T` - The value type (must implement `Deviate + Clone`)
///
/// # Examples
///
/// ```rust
/// use timeseries::{SerieEntry, Range};
///
/// // Single point entry
/// let entry = SerieEntry {
///     range: Range::new(5),
///     value: 10.5f32,
/// };
///
/// // Range entry representing compressed data from time 5 to 8
/// let compressed_entry = SerieEntry {
///     range: Range::new(5).extend(8),
///     value: 10.5f32,
/// };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct SerieEntry<I, T> {
    /// The time range this entry represents
    pub range: Range<I>,
    /// The representative value for this time range
    pub value: T,
}

impl<I: Clone, T: Clone> Clone for SerieEntry<I, T> {
    fn clone(&self) -> Self {
        SerieEntry {
            range: self.range.clone(),
            value: self.value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3);

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(timeseries.append_monotonic(2_u8, 32.7f32));
        assert!(timeseries.append_monotonic(3_u8, 32.5f32));
        assert!(timeseries.append_monotonic(4_u8, 33.8f32));
        assert!(timeseries.append_monotonic(6_u8, 34.0f32));
        assert!(timeseries.append_monotonic(8_u8, 28.2f32));
        assert!(timeseries.append_monotonic(10_u8, 12.3f32));

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry {
            range: Range {
                start: 1,
                end: Some(6),
            },
            value: 32.6,
        });
        expected.push(SerieEntry {
            range: Range {
                start: 8,
                end: None,
            },
            value: 28.2,
        });
        expected.push(SerieEntry {
            range: Range {
                start: 10,
                end: None,
            },
            value: 12.3,
        });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn should_not_append_when_full() {
        let mut timeseries: Series<1, u8, f32> = Series::new(0.3);

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(!timeseries.append_monotonic(2_u8, 32.7f32));
        assert!(timeseries.is_full());

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry {
            range: Range {
                start: 1,
                end: None,
            },
            value: 32.6,
        });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn should_return_false_when_append_is_not_monotonic_with_end() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(timeseries.append_monotonic(6_u8, 32.7f32));
        assert!(!timeseries.append_monotonic(1_u8, 32.5f32));

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry {
            range: Range {
                start: 1,
                end: Some(6),
            },
            value: 32.6,
        });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn should_return_false_when_append_is_not_monotonic_without_end() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(timeseries.append_monotonic(6_u8, 35.7f32));
        assert!(!timeseries.append_monotonic(1_u8, 32.5f32));

        let mut expected: Vec<SerieEntry<u8, f32>, 3> = Vec::new();

        expected.push(SerieEntry {
            range: Range {
                start: 1,
                end: Some(6),
            },
            value: 32.6,
        });

        assert_eq!(timeseries.buckets, expected)
    }

    #[test]
    fn starts_at_some() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(timeseries.append_monotonic(2_u8, 32.7f32));

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

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(timeseries.append_monotonic(2_u8, 32.7f32));

        assert_eq!(timeseries.ends_at(), Some(&2_u8));
    }

    #[test]
    fn ends_at_some_range_start() {
        let mut timeseries: Series<10, u8, f32> = Series::new(0.3f32);

        assert!(timeseries.append_monotonic(1_u8, 32.6f32));
        assert!(timeseries.append_monotonic(2_u8, 35.7f32));

        assert_eq!(timeseries.ends_at(), Some(&2_u8));
    }

    #[test]
    fn ends_at_none() {
        let timeseries: Series<1, u8, f32> = Series::new(0.3f32);
        assert_eq!(timeseries.ends_at(), None);
    }
}
