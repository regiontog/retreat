use super::LiteralInnerType;

pub trait Read: LiteralInnerType {
    type Output;

    fn read(_: &[u8]) -> Self::Output;

    fn read_safe(arena: &[u8]) -> crate::Result<Self::Output> {
        if arena.len() < Self::SIZE {
            Err(crate::NoserError::Undersized(Self::SIZE, arena.to_vec()))
        } else {
            Ok(Self::read(arena))
        }
    }
}
