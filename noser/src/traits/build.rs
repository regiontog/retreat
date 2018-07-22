pub trait Build<'a>: Sized {
    fn build(&'a mut [u8]) -> ::Result<'a, (&'a mut [u8], Self)>;
}
