pub trait Variants<'a>: Sized {
    fn variants() -> u64;
    fn ord(&self) -> u64;

    fn variant(_: u64, _: &'a mut [u8]) -> crate::Result<(&'a mut [u8], Self)>;

    #[inline]
    fn bytes_for_n_variants() -> usize {
        ((Self::variants() as f64).log2() / 8.0).ceil() as usize
    }
}
