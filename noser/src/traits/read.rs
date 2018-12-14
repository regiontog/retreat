use crate::traits::size::{Sizable, Static};

pub trait Read<'r> {
    type Output;

    fn read<'a>(_: &'a [u8]) -> Self::Output
    where
        'a: 'r;

    #[inline]
    fn read_safe<'a>(arena: &'a [u8]) -> Result<Self::Output, crate::NoserError>
    where
        Self: Sizable<Strategy = Static>,
        'a: 'r,
    {
        let len = Self::static_size() as usize;

        if arena.len() < len {
            Err(crate::NoserError::Undersized(len, arena.to_vec()))
        } else {
            Ok(Self::read(arena))
        }
    }
}
