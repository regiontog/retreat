pub trait Write {
    fn write(_: &mut [u8], val: Self);
}
