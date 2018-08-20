use ext::SliceExt;
use traits::{find::Find, Build, Imprinter, Read, StaticSize, Write};

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

impl<'a, T: Find> Find for Literal<'a, T> {
    type Strategy = T::Strategy;
}

impl<'a, T: StaticSize> StaticSize for Literal<'a, T> {
    #[inline]
    fn size() -> ::Ptr {
        T::size()
    }
}

impl<'a, T: StaticSize> Build<'a> for Literal<'a, T> {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<(&'a mut [u8], Self)> {
        let (left, right) = arena.noser_split(T::size())?;

        Ok((
            right,
            Literal {
                arena: left,
                phantom: PhantomData,
            },
        ))
    }
}

impl<'a, T: StaticSize> Imprinter<'a> for T {
    type OnSuccess = ();

    #[inline]
    fn imprint(&self, arena: &'a mut [u8]) -> ::Result<()> {
        arena.noser_split(Self::size())?;
        Ok(())
    }
}
