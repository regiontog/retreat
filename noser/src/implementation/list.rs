use crate::ext::SliceExt;
// use traits::find::{DynamicFind, Find, LTP_SIZE};
use crate::traits::{
    size::{Dynamic, SizeKind, Static},
    Build, Imprinter, Read, Sizable, Write,
};
use crate::Ptr;

use std::borrow::Borrow;
use std::cell::Cell;
use std::marker::PhantomData;

pub type ListLen = u32;
pub const LIST_LEN_SIZE: Ptr = ::std::mem::size_of::<ListLen>() as Ptr;

#[derive(Debug)]
pub struct CovariantList<T> {
    phantom: PhantomData<T>,
    capacity: ListLen,
}

#[derive(Debug)]
pub struct List<'a, T> {
    pub inner: CovariantList<T>,
    pub arena: &'a mut [u8],
}

impl<'a, T> CovariantList<T> {
    #[inline]
    pub fn get<'b>(&self, arena: &'b mut [u8], idx: ListLen) -> T
    where
        T: Sizable + Build<'b>,
    {
        unsafe { T::unchecked_build(self.item_slice(arena, idx)) }
    }

    #[inline]
    pub fn item_slice<'b>(&self, arena: &'b mut [u8], idx: ListLen) -> &'b mut [u8]
    where
        T: Sizable,
    {
        assert!(idx < self.capacity);

        match T::size() {
            SizeKind::Exactly(size) => &mut arena[idx as usize * size as usize..],
            SizeKind::Dynamic => {
                // TODO: Store index in a cache?

                let mut ptr = 0;

                for _ in 0..idx {
                    ptr += T::read_size(&arena[ptr..]).expect(
                        "List should have verified the integrity of the buffer on List::build(...)",
                    ) as usize;
                }

                &mut arena[ptr..]
            }
        }
    }
}

impl<'a, T> List<'a, T> {
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.inner.capacity
    }

    #[inline]
    unsafe fn get_from_imut<'b>(&'b self, idx: ListLen) -> T
    where
        T: Sizable + Build<'b>,
    {
        let mut_self: &mut Self = &mut *(self as *const Self as *mut Self);

        self.inner.get(mut_self.arena, idx)
    }

    #[inline]
    pub fn borrow<'b>(&'b self, idx: ListLen, cb: impl Fn(&T))
    where
        T: Sizable + Build<'b>,
    {
        // VERIFY: This should be fine as we only give access to a &T.
        cb(&unsafe { self.get_from_imut(idx) });
    }

    #[inline]
    pub fn borrowable<'b>(&'b self, idx: ListLen) -> impl Borrow<T>
    where
        T: Sizable + Build<'b>,
    {
        // VERIFY: This should be fine as we only give access to a &T.
        unsafe { self.get_from_imut(idx) }
    }

    /// Helper function to coerce a closure's input type to this instance's T
    #[inline]
    pub fn coerce<F: Fn(T)>(&self, cb: F) -> F
    where
        T: Build<'a> + Sizable,
    {
        cb
    }

    #[inline]
    pub fn from<'b>(item_types: &'b [T]) -> DynamicItemListImprinter<'b, T>
    where
        T: Imprinter,
    {
        DynamicItemListImprinter {
            list_imprinter: ListImprinter {
                capacity: item_types.len() as ListLen,
            },
            items_sum_size: item_types.iter().map(|item| item.result_size()).sum(),
            item_types: item_types,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: ListLen) -> StaticItemListImprinter<T> {
        StaticItemListImprinter {
            list_imprinter: ListImprinter { capacity: capacity },
            phantom: PhantomData,
        }
    }
}

impl<'a, T: Sizable + Build<'a>> Build<'a> for List<'a, T> {
    #[inline]
    unsafe fn unchecked_build(arena: &'a mut [u8]) -> Self {
        let capacity = ListLen::read(arena);

        List {
            arena: &mut arena[LIST_LEN_SIZE as usize..],
            inner: CovariantList {
                capacity,
                phantom: PhantomData,
            },
        }
    }

    #[inline]
    fn build(arena: &'a mut [u8]) -> crate::Result<(&mut [u8], Self)> {
        // First bytes of list is it's length
        let (left, arena) = arena.noser_split(LIST_LEN_SIZE)?;
        let capacity = ListLen::read(left);

        let mut running_size: Ptr = 0;

        {
            // The rest is the arena of this list

            // Figure out the length of each element and write it to the lookup table,
            // as we could panic if the lookup table received is invalid.
            // Also return Err if arena is undersized here instead of in get's.

            for _ in 0..capacity as usize {
                let size = T::in_bounds(&arena[running_size as usize..])?;
                running_size = running_size
                    .checked_add(size)
                    .ok_or(crate::NoserError::IntegerOverflow)?;
            }
        }

        let (arena, right) = arena.noser_split(running_size)?;

        Ok((
            right,
            List {
                arena,
                inner: CovariantList {
                    capacity,
                    phantom: PhantomData,
                },
            },
        ))
    }
}

impl<'a, T> Sizable for List<'a, T>
where
    T: Sizable,
{
    type Strategy = Dynamic;

    fn read_size(arena: &[u8]) -> crate::Result<crate::Ptr> {
        let capacity = ListLen::read_safe(arena)?;

        match T::size() {
            SizeKind::Exactly(size) => Ok(capacity
                .checked_mul(size)
                .and_then(|mul| ListLen::static_size().checked_add(mul))
                .ok_or(crate::NoserError::IntegerOverflow)?),

            SizeKind::Dynamic => {
                let mut read_head = ListLen::static_size();

                for _ in 0..capacity {
                    read_head += T::read_size(&arena[read_head as usize..]).map_err(Into::into)?;
                }

                Ok(read_head)
            }
        }
    }
}

pub struct DynamicItemListImprinter<'a, A: Imprinter> {
    list_imprinter: ListImprinter,
    item_types: &'a [A],
    items_sum_size: Ptr,
}

pub struct StaticItemListImprinter<A> {
    list_imprinter: ListImprinter,
    phantom: PhantomData<A>,
}

struct ListImprinter {
    capacity: ListLen,
}

impl ListImprinter {
    #[inline]
    fn imprint<'a>(&self, arena: &'a mut [u8]) -> crate::Result<&'a mut [u8]> {
        // First write the capacity of the list
        let (left, right) = arena.noser_split(LIST_LEN_SIZE)?;
        ListLen::write(left, self.capacity);

        Ok(right)
    }
}

impl<'a, A> Imprinter for StaticItemListImprinter<A>
where
    A: Sizable<Strategy = Static>,
{
    type OnSuccess = ();

    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<Self::OnSuccess> {
        let capacity = self.list_imprinter.capacity;

        // Static item list don't need to initialize the lookup table
        let arena = self.list_imprinter.imprint(arena)?;

        // Ensure the arena is large enough
        arena.noser_split(capacity * A::static_size())?;

        Ok(())
    }

    #[inline]
    fn result_size(&self) -> crate::Ptr {
        self.list_imprinter.capacity * A::static_size() + LIST_LEN_SIZE
    }
}

impl<'a, A: Imprinter> Imprinter for DynamicItemListImprinter<'a, A> {
    type OnSuccess = ();

    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<Self::OnSuccess> {
        let arena = self.list_imprinter.imprint(arena)?;

        // let lookup_table_size = Ptr::static_size() * self.item_types.len() as ::Ptr;
        // let (lookup_table, right) = arena.noser_split(lookup_table_size)?;

        // let mut item_ptr = self.list_imprinter.result_size() + lookup_table_size;
        let cell = Cell::new(arena);

        // for (kind, chunk) in self
        //     .item_types
        //     .iter()
        //     .zip(lookup_table.chunks_mut(Ptr::static_size() as usize))
        // {
        //     let kind_size = kind.result_size();
        //     item_ptr = item_ptr
        //         .checked_add(kind_size)
        //         .ok_or(::NoserError::IntegerOverflow)?;

        //     Ptr::write(chunk, item_ptr);
        for kind in self.item_types {
            let (left, right) = cell.take().noser_split(kind.result_size())?;
            kind.imprint(left)?;
            cell.set(right);
        }

        Ok(())
    }

    #[inline]
    fn result_size(&self) -> crate::Ptr {
        self.items_sum_size + LIST_LEN_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Imprinter;
    use crate::Literal;

    #[test]
    fn list() {
        let mut arena = List::<Literal<'_, u8>>::with_capacity(10)
            .create_buffer()
            .unwrap();

        let mut owned: List<'_, Literal<'_, u8>> = List::create(&mut arena).unwrap();
        get!(owned, 0, |mut item| {
            item.write(10);
        });

        get!(owned, 9, |mut item| {
            item.write(11);
        });

        owned.borrow(0, |item| {
            assert_eq!(item.read(), 10);
        });

        owned.borrow(9, |item| {
            assert_eq!(item.read(), 11);
        });
    }

    #[test]
    fn nested_list() {
        let mut arena = List::from(&[
            List::<Literal<'_, u8>>::with_capacity(2),
            List::<Literal<'_, u8>>::with_capacity(2),
        ]).create_buffer()
        .unwrap();

        let mut owned: List<'_, List<'_, Literal<'_, u8>>> = List::create(&mut arena).unwrap();

        get!(owned, 0, |mut sublist| {
            get!(sublist, 0, |mut item| {
                item.write(10);
            });
        });

        get!(owned, 1, |mut sublist| {
            get!(sublist, 0, |mut item| {
                item.write(12);
            });
        });

        owned.borrow(0, |sublist| {
            sublist.borrow(0, |item| {
                assert_eq!(item.read(), 10);
            });
        });

        owned.borrow(1, |sublist| {
            sublist.borrow(0, |item| {
                assert_eq!(item.read(), 12);
            });
        });
    }

    #[test]
    fn undersized_arena() {
        let mut arena = List::from(&[
            List::<Literal<'_, u8>>::with_capacity(5),
            List::<Literal<'_, u8>>::with_capacity(5),
            List::<Literal<'_, u8>>::with_capacity(5),
        ]).create_buffer()
        .unwrap();

        let undersized = &mut arena[..23];

        let mut results = vec![];
        results.push(
            List::from(&[
                List::<Literal<'_, u8>>::with_capacity(5),
                List::<Literal<'_, u8>>::with_capacity(5),
                List::<Literal<'_, u8>>::with_capacity(5),
            ]).imprint(undersized),
        );

        results.push(List::<List<'_, Literal<'_, u8>>>::create(undersized).map(|_| ()));

        println!("{:?}", results);
        assert!(results.into_iter().all(|r| r.is_err()));
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_list() {
        let mut arena = List::from(&[
            List::<Literal<'_, u8>>::with_capacity(2),
            List::<Literal<'_, u8>>::with_capacity(2),
        ]).create_buffer()
        .unwrap();

        let owned = List::<List<'_, Literal<'_, u8>>>::create(&mut arena).unwrap();
        owned.borrow(2, |_| {});
    }

    #[test]
    fn in_bounds_list() {
        let mut arena = List::from(&[
            List::<Literal<'_, u8>>::with_capacity(2),
            List::<Literal<'_, u8>>::with_capacity(2),
        ]).create_buffer()
        .unwrap();

        let owned = List::<List<'_, Literal<'_, u8>>>::create(&mut arena).unwrap();
        owned.borrow(1, |_| {});
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_list2() {
        let mut arena = List::<Literal<'_, u8>>::with_capacity(50)
            .create_buffer()
            .unwrap();

        let owned = List::<Literal<'_, u8>>::create(&mut arena).unwrap();
        owned.borrow(50, |_| {});
    }

    #[test]
    fn in_bounds_list2() {
        let mut arena = List::<Literal<'_, u8>>::with_capacity(50)
            .create_buffer()
            .unwrap();

        let owned = List::<Literal<'_, u8>>::create(&mut arena).unwrap();
        owned.borrow(49, |_| {});
    }
}
