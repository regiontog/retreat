pub trait Build<'a> {
    fn build(&'a mut [u8]) -> (&'a mut [u8], Self);
}
