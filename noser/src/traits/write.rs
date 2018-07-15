pub trait Write {
    fn write(&mut [u8], val: Self);
}
