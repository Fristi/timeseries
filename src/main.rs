use chrono::DateTime;
use crate::timeseries::Series;

mod timeseries;

fn main() {

    let mut timeseries = Series::new(0.3f32);

    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T14:48:00.000Z").unwrap(), 32.6f32);
    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T14:58:00.000Z").unwrap(), 32.7f32);
    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T15:18:00.000Z").unwrap(), 32.5f32);
    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T15:22:00.000Z").unwrap(), 33.8f32);
    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T16:48:00.000Z").unwrap(), 34.0f32);
    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T17:48:00.000Z").unwrap(), 28.2f32);
    timeseries.append( DateTime::parse_from_rfc3339("2011-10-05T18:48:00.000Z").unwrap(), 12.3f32);


    println!("buckets: {:?}", timeseries.buckets);
}
