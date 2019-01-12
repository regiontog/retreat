use crate::traits::{DefaultWriter, LiteralInnerType, WriteTypeInfo};
use crate::Literal;

pub struct LiteralWriter;

impl<'a, T> WriteTypeInfo<Literal<'a, T>> for LiteralWriter
where
    T: LiteralInnerType,
{
    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<()> {
        T::imprint(arena)
    }

    #[inline]
    fn result_size(&self) -> crate::Ptr {
        T::SIZE as crate::Ptr
    }
}

static WRITE_LITERAL_TYPE: LiteralWriter = LiteralWriter {};

impl<'a, T> DefaultWriter for Literal<'a, T>
where
    T: LiteralInnerType,
{
    fn writer() -> &'static WriteTypeInfo<Self> {
        &WRITE_LITERAL_TYPE
    }
}
