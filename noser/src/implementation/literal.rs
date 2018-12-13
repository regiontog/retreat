use ext::SliceExt;
use traits::{size::ReadReturn, Build, Read, Sizable, Write};

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Literal<'a, T> {
    arena: &'a mut [u8],
    phantom: PhantomData<T>,
}

impl<'a, T: Read<'a>> Literal<'a, T> {
    #[inline]
    pub fn read(&'a self) -> T::Output {
        T::read(self.arena)
    }
}

impl<'a, T: Write> Literal<'a, T> {
    #[inline]
    pub fn write(&mut self, val: T) {
        T::write(self.arena, val)
    }
}

impl<'a, T: Sizable> Sizable for Literal<'a, T> {
    type Strategy = T::Strategy;

    #[inline]
    fn read_size(arena: &[u8]) -> ReadReturn<T> {
        T::read_size(arena)
    }
}

impl<'a, T: Sizable> Build<'a> for Literal<'a, T> {
    #[inline]
    unsafe fn unchecked_build(arena: &'a mut [u8]) -> Self {
        Literal {
            arena: arena,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<(&'a mut [u8], Self)> {
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
    use implementation::scalars;
    use traits::*;

    #[test]
    fn literal() {
        let mut arena = scalars::IMPRINT_U8.create_buffer().unwrap();

        let mut owned: Literal<u8> = Literal::create(&mut arena).unwrap();
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
