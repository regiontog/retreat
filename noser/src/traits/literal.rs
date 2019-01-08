pub trait LiteralInnerType {
    const SIZE: usize;

    fn imprint(_: &mut [u8]) -> crate::Result<()>;
}
