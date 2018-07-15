pub trait DynamicSize {
    fn size(&self) -> usize;
}

pub trait StaticSize {
    fn size() -> usize;
}
