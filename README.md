# Timeseries

A custom deviation-based time series compression library, closely resembling the Swinging Door Compression algorithm. Built for `no_std` environments with predictable memory usage.

![Timeseries Compression Library](/timeseries-logo.jpg)

## Features

- **No-std compatible**: Works in embedded and constrained environments
- **Memory safe**: Uses `#![deny(unsafe_code)]` for guaranteed memory safety
- **Fixed capacity**: Based on `heapless::Vec` for predictable memory usage
- **Monotonic timestamps**: Points must be added in strictly increasing order
- **Deviation-based compression**: Only stores values that deviate significantly from previous ones
- **Generic types**: Works with any ordered index type and any numeric value type

## Key Characteristics

- **Monotonic timestamps**: Points must be added in strictly increasing order
- **Deviation threshold**: New values are only stored if they deviate from the last stored value by more than `max_deviation`
- **Range merging**: Values within the allowed deviation extend the time range of the last entry instead of creating new ones
- **Fixed-capacity**: Based on `heapless::Vec`, suitable for embedded/no-std use

## Algorithm Summary

- If a new point is within `max_deviation` of the last stored value, extend the time range
- Otherwise, store a new entry with its own range
- The internal structure stores "buckets" (compressed segments), each covering a time range with a representative value

This is a simplified and efficient form of lossy compression with a deviation bound, excellent for embedded systems or constrained environments.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
heapless = "0.7.16"
```

### Basic Example

```rust
use timeseries::Series;

// Create a series with capacity for 10 entries, max deviation of 0.3
let mut timeseries: Series<10, u8, f32> = Series::new(0.3);

// Add monotonic data points
assert!(timeseries.append_monotonic(1, 32.6));
assert!(timeseries.append_monotonic(2, 32.7)); // Within deviation, extends range
assert!(timeseries.append_monotonic(3, 32.5)); // Within deviation, extends range  
assert!(timeseries.append_monotonic(4, 33.8)); // Exceeds deviation, new entry
assert!(timeseries.append_monotonic(6, 34.0)); // Within deviation, extends range

// Check series bounds
println!("Starts at: {:?}", timeseries.starts_at()); // Some(1)
println!("Ends at: {:?}", timeseries.ends_at());     // Some(6)
println!("Is full: {}", timeseries.is_full());       // false
```

### Working with Different Types

The library is generic over index and value types:

```rust
// Using different numeric types
let mut series_i32: Series<5, i32, f64> = Series::new(1.0);
let mut series_u64: Series<100, u64, f32> = Series::new(0.1);

// Any ordered type can be used as index
let mut series_string: Series<10, String, f32> = Series::new(0.5);
```

## API Reference

### `Series<N, I, T>`

The main time series structure with:
- `N`: Maximum capacity (const generic)
- `I`: Index type (must implement `Ord + Clone`)  
- `T`: Value type (must implement `Deviate + Clone`)

#### Methods

- `new(max_deviation: T) -> Series<N, I, T>`: Create a new series
- `append_monotonic(&mut self, at: I, value: T) -> bool`: Add a data point, returns `false` if not monotonic or series is full
- `starts_at(&self) -> Option<&I>`: Get the earliest timestamp
- `ends_at(&self) -> Option<&I>`: Get the latest timestamp  
- `is_full(&self) -> bool`: Check if series has reached capacity

### `Deviate` Trait

Implemented for `f32` and `f64`. Custom types can implement this trait:

```rust
impl Deviate for f32 {
    fn deviate(&self, other: &Self, max_deviation: &Self) -> bool {
        self - other > *max_deviation
    }
}
```

### Data Structures

- `SerieEntry<I, T>`: Represents a compressed segment with a range and value
- `Range<I>`: Represents a time range with start and optional end

## Error Handling

The `append_monotonic` method returns `bool`:
- `true`: Point was successfully added or merged
- `false`: Point was rejected (not monotonic, series full, or other constraint)

## Memory Usage

Memory usage is predictable and bounded by the capacity `N`. Each entry stores:
- Range (start + optional end of type `I`)
- Value of type `T`

Total memory ≈ `N * (2 * sizeof(I) + sizeof(T) + metadata)`
