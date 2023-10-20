
pub struct Series<I, T> {
    max_deviation: T,
    pub buckets: Vec<SerieEntry<I, T>>
}

impl <I : Ord, T : Deviate> Series<I, T> {
    pub fn new(max_deviation: T) -> Series<I, T> {
        Series { max_deviation, buckets: Vec::new() }
    }

    pub fn append(&mut self, at: I, value: T) {
        match self.buckets.pop() {
            Some(v) => {
                if v.value.deviate(&value, &self.max_deviation) {
                    self.buckets.append(&mut vec![v, SerieEntry{ range: Range::new(at), value }])
                } else {
                    let new_range = v.range.extend(at);
                    self.buckets.append(&mut vec![SerieEntry { range: new_range, value: v.value}])
                }
            },
            None => self.buckets.append(&mut vec![SerieEntry{ range: Range::new(at), value }])
        }
    }
}

#[derive(Debug)]
struct Range<I> {
    start: I,
    end: Option<I>
}

impl <I: Ord> Range<I> {
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

trait Deviate {
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

#[derive(Debug)]
pub struct SerieEntry<I, T> {
    range: Range<I>,
    value: T
}

