use traits::{Build, Read, StaticSize, WithArena, Write};

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Literal<'a, T> {
    arena: &'a mut [u8],
    phantom: PhantomData<T>,
}

impl<'a, T: Read> Literal<'a, T> {
    #[inline]
    pub fn read(&self) -> T {
        T::read(self.arena)
    }
}

impl<'a, T: Write> Literal<'a, T> {
    #[inline]
    pub fn write(&mut self, val: T) {
        T::write(self.arena, val)
    }
}

impl<'a, T: StaticSize> StaticSize for Literal<'a, T> {
    #[inline]
    fn size() -> usize {
        T::size()
    }
}

impl<'a, T: StaticSize> Build<'a> for Literal<'a, T> {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], Self)> {
        if arena.len() < T::size() {
            return Err(::NoserError::Undersized(T::size(), arena));
        }

        let (left, right) = arena.split_at_mut(T::size());

        Ok((
            right,
            Literal {
                arena: left,
                phantom: PhantomData,
            },
        ))
    }
}

impl<'a, T: StaticSize + Write> WithArena<'a, Literal<'a, T>> for T {
    #[inline]
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, Literal<'a, T>> {
        let (_, mut lit) = Literal::build(arena)?;
        lit.write(self);
        Ok(lit)
    }
}
