pub trait Tagged: Sized {
    fn variant_tag(&self) -> u64;
    fn from_tag(_: u64) -> crate::Result<Self>;
}

pub trait StaticEnum<'a>: Sized {
    type VariantEnum: Tagged;

    const VARIANT_BYTES: usize;
    const CONTENTS_SIZE: Option<usize>;

    #[inline]
    fn contents_size() -> usize {
        *Self::CONTENTS_SIZE.get_or_insert_with(Self::calculate_contents_size)
    }

    #[inline]
    fn static_size() -> crate::Ptr {
        (Self::VARIANT_BYTES + Self::contents_size()) as crate::Ptr
    }

    fn calculate_contents_size() -> usize;
    fn construct_variant(_: &Self::VariantEnum, _: &'a mut [u8]) -> crate::Result<Self>;
    fn unchecked_construct_variant(_: &Self::VariantEnum, _: &'a mut [u8]) -> Self;
}
