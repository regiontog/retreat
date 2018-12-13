use crate::traits::size::{Sizable, Static};

pub trait Read<'a> {
    type Output;

    fn read(_: &'a [u8]) -> Self::Output;

    #[inline]
    fn read_safe(arena: &'a [u8]) -> Result<Self::Output, crate::NoserError>
    where
        Self: Sizable<Strategy = Static>,
    {
        let len = Self::static_size() as usize;

        if arena.len() < len {
            Err(crate::NoserError::Undersized(len, arena.to_vec()))
        } else {
            Ok(Self::read(arena))
        }
    }
}
