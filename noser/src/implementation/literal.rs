use traits::{Build, Read, StaticSize, Write};

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
    fn build(arena: &'a mut [u8]) -> (&'a mut [u8], Self) {
        let (left, right) = arena.split_at_mut(T::size());

        (
            right,
            Literal {
                arena: left,
                phantom: PhantomData,
            },
        )
    }
}
