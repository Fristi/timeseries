timeseries
---

a custom deviation-based time series compression, closely resembling the Swinging Door Compression algorithm.

## Key Characteristics:
- Monotonic timestamps: Points must be added in strictly increasing order.
- Deviation threshold: New values are only stored if they deviate from the last stored value by more than max_deviation.
- Range merging: Values within the allowed deviation extend the time range of the last entry instead of creating new ones.
- Fixed-capacity: Based on heapless::Vec, suitable for embedded/no-std use.

### Algorithm Summary:
- If a new point is within max_deviation of the last stored value, extend the time range. Otherwise, store a new entry.
- The internal structure stores these "buckets" (compressed segments), each covering a time range with a representative value.

This is a simplified and efficient form of lossy compression with a deviation bound, excellent for embedded systems or constrained environments where you need predictable memory usage.
