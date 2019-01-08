use crate::prelude::SliceExt;
use crate::traits::{
    size::{ReadReturn, Sizeable, Static},
    Build, DefaultWriter, LiteralInnerType, Read, Write, WriteTypeInfo,
};

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Literal<'l, T> {
    arena: &'l mut [u8],
    phantom: PhantomData<T>,
}

impl<'r, T> Literal<'_, T>
where
    T: Read<'r>,
{
    #[inline]
    pub fn read<'s>(&'s self) -> T::Output
    where
        's: 'r,
    {
        T::read(self.arena)
    }
}

impl<T> Literal<'_, T>
where
    T: Write,
{
    #[inline]
    pub fn write(&mut self, val: T) {
        T::write(self.arena, val)
    }
}

impl<T> Sizeable for Literal<'_, T>
where
    T: LiteralInnerType,
{
    type Strategy = Static;

    #[inline]
    fn read_size(_: &[u8]) -> ReadReturn<Self> {
        Ok(T::SIZE as crate::Ptr)
    }
}

unsafe impl<'l, T> Build<'l> for Literal<'l, T>
where
    T: LiteralInnerType,
{
    #[inline]
    fn unchecked_build<'n>(arena: &'n mut [u8]) -> (&'n mut [u8], Literal<T>)
    where
        'n: 'l,
    {
        let (left, right) = arena.split_at_mut(T::SIZE);
        (
            right,
            Literal {
                arena: left,
                phantom: PhantomData,
            },
        )
    }

    #[inline]
    fn build<'n>(arena: &'n mut [u8]) -> crate::Result<(&'n mut [u8], Self)>
    where
        'n: 'l,
    {
        let (left, right) = arena.noser_split(T::SIZE as crate::Ptr)?;

        Ok((
            right,
            Literal {
                arena: left,
                phantom: PhantomData,
            },
        ))
    }
}

pub struct LiteralWriter;

static WRITE_LITERAL_TYPE: LiteralWriter = LiteralWriter {};

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

impl<'a, T> DefaultWriter for Literal<'a, T>
where
    T: LiteralInnerType,
{
    type Writer = LiteralWriter;

    fn writer() -> &'static Self::Writer {
        &WRITE_LITERAL_TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Build;

    #[test]
    fn literal() {
        let mut arena = <Literal<u8>>::buffer().unwrap();

        let mut owned: Literal<'_, u8> = Literal::create(&mut arena).unwrap();
        owned.write(10);

        assert_eq!(owned.read(), 10);
    }

    #[test]
    fn undersized_arena() {
        let mut arena = <Literal<u64>>::buffer().unwrap();

        let undersized = &mut arena[..3];

        let mut results = vec![];

        results.push(<Literal<u64>>::write_type(undersized));
        results.push(<Literal<u64>>::create(undersized).map(|_| ()));

        println!("{:?}", results);
        assert!(results.into_iter().all(|r| r.is_err()));
    }
}
