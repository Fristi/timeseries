use chrono::{DateTime, Utc};
use crate::timeseries::Series;

mod timeseries;

fn main() {

    let mut timeseries: Series<10, DateTime<Utc>, f32> = Series::new(0.3);

    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T14:48:00.000Z").unwrap().into(), 32.6f32);
    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T14:58:00.000Z").unwrap().into(), 32.7f32);
    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T15:18:00.000Z").unwrap().into(), 32.5f32);
    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T15:22:00.000Z").unwrap().into(), 33.8f32);
    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T16:48:00.000Z").unwrap().into(), 34.0f32);
    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T17:48:00.000Z").unwrap().into(), 28.2f32);
    timeseries.append_monotonic( DateTime::parse_from_rfc3339("2011-10-05T18:48:00.000Z").unwrap().into(), 12.3f32);

    let starts = timeseries.starts_at().unwrap();
    let ends = timeseries.ends_at().unwrap();

    let dur = ends.signed_duration_since(starts);


    println!("buckets: {:?}", timeseries.buckets);
    println!("dur: {:?}", dur);
}
