use std::iter::{Cloned, Repeat, Take};
use std::slice::Iter;

use crate::prelude::SliceExt;
use crate::traits::{DefaultWriter, LiteralInnerType, Write, WriteTypeInfo};
use crate::List;

use freyr::prelude::*;
use freyr::TakeExactly;

pub struct ListWriter<I> {
    writers: I,
    sum_writers_result_size: crate::Ptr,
    capacity: crate::Ptr,
}

pub struct SliceIntoIterWrapper<'a, 'b, T: ?Sized> {
    slice: &'a [&'b T],
}

impl<T: ?Sized> Clone for SliceIntoIterWrapper<'_, '_, T> {
    fn clone(&self) -> Self {
        SliceIntoIterWrapper { slice: self.slice }
    }
}

impl<'a, 'b, T: ?Sized> IntoIterator for SliceIntoIterWrapper<'a, 'b, T> {
    type IntoIter = Cloned<Iter<'a, &'b T>>;
    type Item = &'b T;

    fn into_iter(self) -> Self::IntoIter {
        self.slice.iter().cloned()
    }
}

impl<I> ListWriter<I> {
    pub fn new<'a, T>(writers: I) -> Self
    where
        T: 'a,
        I: IntoIterator<Item = &'a dyn WriteTypeInfo<T>> + Clone,
        I::IntoIter: ExactSizeIterator,
    {
        let iterator = writers.clone().into_iter();

        ListWriter {
            writers,
            capacity: iterator.len() as crate::Ptr,
            sum_writers_result_size: iterator.map(|w| w.result_size()).sum(),
        }
    }
}

pub type FromSlice<'a, 'b, T> =
    ListWriter<SliceIntoIterWrapper<'a, 'b, (dyn WriteTypeInfo<T> + 'b)>>;

impl<'a, 'b, T> FromSlice<'a, 'b, T> {
    pub fn from_slice(writers: &'a [&'b (dyn WriteTypeInfo<T> + 'b)]) -> Self {
        ListWriter::new(SliceIntoIterWrapper { slice: writers })
    }
}

pub type WithCapacity<T> = ListWriter<TakeExactly<Take<Repeat<&'static dyn WriteTypeInfo<T>>>>>;

impl<T> WithCapacity<T> {
    pub fn with_capacity(capacity: crate::Ptr) -> Self
    where
        T: DefaultWriter,
    {
        let writer: &dyn WriteTypeInfo<T> = T::writer();

        ListWriter::new(std::iter::repeat(writer).take_exactly(capacity as usize))
    }
}

impl<'a, T, I> WriteTypeInfo<List<'_, T>> for ListWriter<I>
where
    T: 'a,
    I: IntoIterator<Item = &'a dyn WriteTypeInfo<T>> + Clone,
{
    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<()> {
        // First write the capacity of the list
        let (left, mut arena) = arena.noser_split(crate::ListLen::SIZE as crate::Ptr)?;
        crate::ListLen::write(left, self.capacity);

        for kind in self.writers.clone().into_iter() {
            let (left, right) = arena.noser_split(kind.result_size())?;
            kind.imprint(left)?;
            arena = right;
        }

        Ok(())
    }

    #[inline]
    fn result_size(&self) -> crate::Ptr {
        self.sum_writers_result_size + crate::ListLen::SIZE as crate::Ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::WriteTypeInfo;
    use crate::Literal;

    #[test]
    fn compiles() {
        WithCapacity::<Literal<u64>>::with_capacity(3)
            .create_buffer()
            .unwrap();

        FromSlice::from_slice(&[&WithCapacity::<Literal<u64>>::with_capacity(3)])
            .create_buffer()
            .unwrap();
    }

    #[test]
    fn inference_problem() {
        use crate::traits::Build;

        let writer: &dyn WriteTypeInfo<Literal<u64>> = Literal::<u64>::writer();
        let writer = ListWriter::new(std::iter::repeat(writer).take_exactly(50));
        // let arena: noser::Result<Vec<u8>> = writer.create_buffer();
        let arena = writer.create_buffer();
        let mut arena = arena.unwrap();

        let owned = List::<Literal<u64>>::create(&mut arena).unwrap();
        owned.borrow(49);
    }
}
