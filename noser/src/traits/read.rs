pub trait Read {
    type Output;

    fn read(&[u8]) -> Self::Output;
}
