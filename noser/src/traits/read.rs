use traits::size::{Sizable, Static};

pub trait Read<'a> {
    type Output;

    fn read(&'a [u8]) -> Self::Output;

    #[inline]
    fn read_safe(arena: &'a [u8]) -> Result<Self::Output, ::NoserError>
    where
        Self: Sizable<Strategy = Static>,
    {
        let len = Self::static_size() as usize;

        if arena.len() < len {
            Err(::NoserError::Undersized(len, arena.to_vec()))
        } else {
            Ok(Self::read(arena))
        }
    }
}
