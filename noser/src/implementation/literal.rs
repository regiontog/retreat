use crate::ext::SliceExt;
use crate::traits::{size::ReadReturn, Build, Read, Sizable, Write};

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

impl<T> Sizable for Literal<'_, T>
where
    T: Sizable,
{
    type Strategy = T::Strategy;

    #[inline]
    fn read_size(arena: &[u8]) -> ReadReturn<T> {
        T::read_size(arena)
    }
}

impl<'l, T> Build<'l> for Literal<'l, T>
where
    T: Sizable,
{
    #[inline]
    unsafe fn unchecked_build<'a>(arena: &'a mut [u8]) -> Self
    where
        'a: 'l,
    {
        Literal {
            arena: arena,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn build<'a>(arena: &'a mut [u8]) -> crate::Result<(&'a mut [u8], Self)>
    where
        'a: 'l,
    {
        let size = T::read_size(arena).map_err(Into::into)?;
        let (left, right) = arena.noser_split(size)?;

        Ok((
            right,
            Literal {
                arena: left,
                phantom: PhantomData,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::scalars;
    use crate::traits::*;

    #[test]
    fn literal() {
        let mut arena = scalars::IMPRINT_U8.create_buffer().unwrap();

        let mut owned: Literal<'_, u8> = Literal::create(&mut arena).unwrap();
        owned.write(10);

        assert_eq!(owned.read(), 10);
    }

    #[test]
    fn undersized_arena() {
        let mut arena = scalars::IMPRINT_U64.create_buffer().unwrap();

        let undersized = &mut arena[..3];

        let mut results = vec![];

        results.push(scalars::IMPRINT_U64.imprint(undersized));
        results.push(Literal::<u64>::create(undersized).map(|_| ()));

        println!("{:?}", results);
        assert!(results.into_iter().all(|r| r.is_err()));
    }
}
