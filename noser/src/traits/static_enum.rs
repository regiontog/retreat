pub trait Tagged: Sized {
    fn variant_tag(&self) -> u64;
    fn from_tag(_: u64) -> crate::Result<Self>;
}

pub trait StaticEnum: Sized {
    type VariantEnum: Tagged;

    fn variant_bytes() -> usize;
    fn static_size() -> usize;
    fn construct_variant(_: &Self::VariantEnum, _: &mut [u8]) -> crate::Result<Self>;
    fn unchecked_construct_variant(_: &Self::VariantEnum, _: &mut [u8]) -> Self;
}
