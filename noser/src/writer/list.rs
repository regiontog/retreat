use std::iter::{Cloned, Repeat, Take};
use std::slice::Iter;

use crate::prelude::SliceExt;
use crate::traits::{DefaultWriter, Read, Write, WriteTypeInfo};
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

pub type From<'a, 'b, T> = ListWriter<SliceIntoIterWrapper<'a, 'b, (dyn WriteTypeInfo<T> + 'b)>>;

impl<'a, 'b, T> From<'a, 'b, T> {
    pub fn from(writers: &'a [&'b (dyn WriteTypeInfo<T> + 'b)]) -> Self {
        ListWriter::new(SliceIntoIterWrapper { slice: writers })
    }
}

pub type WithCapacity<'a, T> = ListWriter<TakeExactly<Take<Repeat<&'a dyn WriteTypeInfo<T>>>>>;

impl<'a, T> WithCapacity<'a, T> {
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
        let (left, mut arena) = arena.noser_split(crate::ListLen::OUT_SIZE as crate::Ptr)?;
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
        self.sum_writers_result_size + crate::ListLen::OUT_SIZE as crate::Ptr
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

        From::from(&[&WithCapacity::<Literal<u64>>::with_capacity(3)])
            .create_buffer()
            .unwrap();
    }
}
