use crate::prelude::SliceExt;
use crate::traits::{
    size::Sizeable,
    size::{Dynamic, SizeKind, Static},
    Build, Read, Write, WriteTypeInfo,
};
use crate::Ptr;

use std::marker::PhantomData;

// We cannot have a method &mut self -> T on List as &mut Self is invariant on Self.
// As such T's lifetime cannot be narrowed. We also cannot have a method
// &self -> &mut [u8] -> T on List as rust cannot borrow &self and &mut self.buffer
// simultaneously even though they are disjoint. Therefore we need a field builder,
// we can then use a macro that disjointly borrows &self.builder and &mut self.buffer.
// NOTE: Use mem::transmute to shorten invariant lifetime?
#[macro_export]
macro_rules! get {
    ($self_:ident[$idx:expr]) => {
        $self_.inner.get($self_.arena, $idx)
    };
    ($self_:ident[$idx:expr]$([$idxs:expr]) +) => {{
        let sublist = get! { $self_[$idx] };
        get! { sublist$([$idxs])* }
    }};
}

pub(crate) type ListLen = u32;

#[derive(Debug)]
pub struct CovariantList<T> {
    phantom: PhantomData<T>,
    capacity: ListLen,
}

#[derive(Debug)]
pub struct List<'l, T> {
    pub inner: CovariantList<T>,
    pub arena: &'l mut [u8],
}

pub struct DynamicItemListImprinter<'a, 'b, W> {
    list_imprinter: ListImprinter,
    item_types: &'a [&'b W],
    items_sum_size: Ptr,
}

pub struct StaticItemListImprinter<A> {
    list_imprinter: ListImprinter,
    phantom: PhantomData<A>,
}

struct ListImprinter {
    capacity: ListLen,
}

impl<T> CovariantList<T> {
    #[inline]
    pub fn get<'a>(&self, arena: &'a mut [u8], idx: ListLen) -> T
    where
        T: Sizeable + Build<'a>,
    {
        T::unchecked_create(self.item_slice(arena, idx))
    }

    #[inline]
    fn item_slice<'a>(&self, arena: &'a mut [u8], idx: ListLen) -> &'a mut [u8]
    where
        T: Sizeable,
    {
        assert!(idx < self.capacity);

        // let arena = &mut arena[LIST_LEN_SIZE as usize..];

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

impl<'l, T> List<'l, T> {
    #[inline]
    pub fn capacity(&self) -> u32 {
        self.inner.capacity
    }

    // #[inline]
    // pub fn get<'t, 's>(&'s mut self, idx: ListLen) -> T
    // where
    //     T: Sizeable + Build<'t>,
    //     T: 's,
    //     'l: 't,
    //     // 's: 't,
    // {
    //     self.inner.get(self.arena, idx)
    // }

    #[inline]
    unsafe fn get_from_imut<'s>(&'s self, idx: ListLen) -> T
    where
        T: Sizeable + Build<'s>,
    {
        let mut_self: &mut Self = &mut *(self as *const Self as *mut Self);

        self.inner.get(mut_self.arena, idx)
    }

    #[inline]
    pub fn borrow<'s>(&'s self, idx: ListLen) -> freyr::ReadOnly<T>
    where
        T: Sizeable + Build<'s>,
    {
        freyr::ReadOnly::new(unsafe { self.get_from_imut(idx) })
    }

    #[inline]
    pub fn from<'b, W>(item_types: &'b [&'b W]) -> impl WriteTypeInfo<List<'b, T>>
    where
        W: WriteTypeInfo<T>,
    {
        DynamicItemListImprinter {
            list_imprinter: ListImprinter {
                capacity: item_types.len() as ListLen,
            },
            items_sum_size: item_types.iter().map(|&item| item.result_size()).sum(),
            item_types,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: ListLen) -> impl WriteTypeInfo<Self>
    where
        T: Sizeable<Strategy = Static>,
    {
        StaticItemListImprinter {
            list_imprinter: ListImprinter { capacity },
            phantom: PhantomData,
        }
    }
}

unsafe impl<'l, T> Build<'l> for List<'l, T>
where
    T: Sizeable + Build<'l>,
{
    #[inline]
    fn unchecked_build<'w>(arena: &'w mut [u8]) -> (&'w mut [u8], Self)
    where
        'w: 'l,
    {
        let capacity = ListLen::read(arena);
        let size = Self::read_size(arena).expect(
            "unchecked build needs to ensure the arena is correct before calling this method!",
        );

        let (left, right) = arena.split_at_mut(size as usize);
        (
            right,
            List {
                arena: &mut left[ListLen::OUT_SIZE..],
                inner: CovariantList {
                    capacity,
                    phantom: PhantomData,
                },
            },
        )
    }

    #[inline]
    fn build<'w>(arena: &'w mut [u8]) -> crate::Result<(&'w mut [u8], Self)>
    where
        'w: 'l,
    {
        // First bytes of list is it's length
        let (left, arena) = arena.noser_split(ListLen::OUT_SIZE as crate::Ptr)?;
        let capacity = ListLen::read(left);

        let unused = {
            // The rest is the arena of this list
            let mut arena = &arena[..];

            for _ in 0..capacity {
                arena = T::unused(arena)?;
            }

            arena.len()
        };

        // It's important this does not panic, we know it won't since arena.len() - unused
        // is always smaller than arena.len() furthermore unused is never going to exceed
        // arena.len() so the result is always positive as well.
        let (arena, right) = arena.split_at_mut(arena.len() - unused);

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

impl<T> Sizeable for List<'_, T>
where
    T: Sizeable,
{
    type Strategy = Dynamic;

    fn read_size(arena: &[u8]) -> crate::Result<crate::Ptr> {
        let capacity = ListLen::read_safe(arena)?;

        match T::size() {
            SizeKind::Exactly(size) => Ok(capacity
                .checked_mul(size)
                .and_then(|r| r.checked_add(ListLen::OUT_SIZE as crate::Ptr))
                .ok_or(crate::NoserError::IntegerOverflow)?),

            SizeKind::Dynamic => {
                let mut read_head = ListLen::OUT_SIZE as crate::Ptr;

                for _ in 0..capacity {
                    read_head = read_head
                        .checked_add(
                            T::read_size(&arena[read_head as usize..]).map_err(Into::into)?,
                        )
                        .ok_or(crate::NoserError::IntegerOverflow)?;
                }

                Ok(read_head)
            }
        }
    }
}

impl ListImprinter {
    #[inline]
    fn imprint<'a>(&self, arena: &'a mut [u8]) -> crate::Result<&'a mut [u8]> {
        // First write the capacity of the list
        let (left, right) = arena.noser_split(ListLen::OUT_SIZE as crate::Ptr)?;
        ListLen::write(left, self.capacity);

        Ok(right)
    }
}

// TODO: Rewrite to use Default WriteTypeInfo somehow? Problem with other crate's custom WriteTypeInfo?
impl<'a, A> WriteTypeInfo<List<'a, A>> for StaticItemListImprinter<A>
where
    A: Sizeable<Strategy = Static>,
{
    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<()> {
        let capacity = self.list_imprinter.capacity;

        // Static item list don't need to initialize the lookup table
        let arena = self.list_imprinter.imprint(arena)?;

        // Ensure the arena is large enough
        arena.noser_split(capacity * A::static_size())?;

        Ok(())
    }

    #[inline]
    fn result_size(&self) -> crate::Ptr {
        self.list_imprinter.capacity * A::static_size() + ListLen::OUT_SIZE as crate::Ptr
    }
}

impl<'a, W, T> WriteTypeInfo<List<'a, T>> for DynamicItemListImprinter<'_, '_, W>
where
    W: WriteTypeInfo<T>,
{
    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<()> {
        let mut arena = self.list_imprinter.imprint(arena)?;

        for &kind in self.item_types {
            let (left, right) = arena.noser_split(kind.result_size())?;
            kind.imprint(left)?;
            arena = right;
        }

        Ok(())
    }

    #[inline]
    fn result_size(&self) -> crate::Ptr {
        self.items_sum_size + ListLen::OUT_SIZE as crate::Ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Literal;

    #[test]
    fn list() {
        let mut arena = List::<Literal<'_, u8>>::with_capacity(10)
            .create_buffer()
            .unwrap();

        let owned: List<'_, Literal<'_, u8>> = List::create(&mut arena).unwrap();
        let mut item = get! { owned[0] };
        item.write(10);

        let mut item2 = get! { owned[9] };
        item2.write(11);

        assert_eq!(owned.borrow(0).read(), 10);
        assert_eq!(owned.borrow(9).read(), 11);
    }

    #[test]
    fn nested_list() {
        let mut arena = List::from(&[
            &List::<'_, Literal<'_, u8>>::with_capacity(2),
            &List::<'_, Literal<'_, u8>>::with_capacity(2),
        ])
        .create_buffer()
        .unwrap();

        let owned: List<'_, List<'_, Literal<'_, u8>>> = List::create(&mut arena).unwrap();

        let mut item = get! { owned[0][0] };
        item.write(10);

        let mut item = get! { owned[1][0] };
        item.write(12);

        assert_eq!(owned.borrow(0).borrow(0).read(), 10);
        assert_eq!(owned.borrow(1).borrow(0).read(), 12);
    }

    #[test]
    fn undersized_arena() {
        let mut arena = List::from(&[
            &List::<Literal<'_, u8>>::with_capacity(5),
            &List::<Literal<'_, u8>>::with_capacity(5),
            &List::<Literal<'_, u8>>::with_capacity(5),
        ])
        .create_buffer()
        .unwrap();

        let undersized = &mut arena[..23];

        let mut results = vec![];
        results.push(
            List::from(&[
                &List::<Literal<'_, u8>>::with_capacity(5),
                &List::<Literal<'_, u8>>::with_capacity(5),
                &List::<Literal<'_, u8>>::with_capacity(5),
            ])
            .imprint(undersized),
        );

        results.push(List::<List<'_, Literal<'_, u8>>>::create(undersized).map(|_| ()));

        assert!(results.into_iter().all(|r| r.is_err()));
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_list() {
        let mut arena = List::from(&[
            &List::<Literal<'_, u8>>::with_capacity(2),
            &List::<Literal<'_, u8>>::with_capacity(2),
        ])
        .create_buffer()
        .unwrap();

        let owned = List::<List<'_, Literal<'_, u8>>>::create(&mut arena).unwrap();
        owned.borrow(2);
    }

    #[test]
    fn in_bounds_list() {
        let mut arena = List::from(&[
            &List::<Literal<'_, u8>>::with_capacity(2),
            &List::<Literal<'_, u8>>::with_capacity(2),
        ])
        .create_buffer()
        .unwrap();

        let owned = List::<List<'_, Literal<'_, u8>>>::create(&mut arena).unwrap();
        owned.borrow(1);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_list2() {
        let mut arena = List::<Literal<'_, u8>>::with_capacity(50)
            .create_buffer()
            .unwrap();

        let owned = List::<Literal<'_, u8>>::create(&mut arena).unwrap();
        owned.borrow(50);
    }

    #[test]
    fn in_bounds_list2() {
        let mut arena = List::<Literal<'_, u8>>::with_capacity(50)
            .create_buffer()
            .unwrap();

        let owned = List::<Literal<'_, u8>>::create(&mut arena).unwrap();
        owned.borrow(49);
    }
}
