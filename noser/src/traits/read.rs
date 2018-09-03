pub trait Read<'a> {
    type Output;

    fn read(&'a [u8]) -> Self::Output;
}
