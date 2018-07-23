pub trait DynamicSize {
    fn dsize(&self) -> usize;
}

pub trait StaticSize {
    fn size() -> usize;
}
