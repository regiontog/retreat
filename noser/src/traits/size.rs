pub trait DynamicSize {
    fn dsize(&self) -> usize;
}

pub trait StaticSize {
    fn size() -> usize;
}

impl<T: StaticSize> DynamicSize for T {
    fn dsize(&self) -> usize {
        Self::size()
    }
}
