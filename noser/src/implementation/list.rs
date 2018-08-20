use ext::SliceExt;
use traits::find::{DynamicFind, Find, LTP_SIZE};
use traits::{Build, DynamicSize, Imprinter, Read, StaticSize, Write};
use Ptr;

use std::borrow::Borrow;
use std::cell::Cell;
use std::marker::PhantomData;

pub type ListLen = u32;
pub const LIST_LEN_SIZE: Ptr = ::std::mem::size_of::<ListLen>() as Ptr;

#[derive(Debug)]
pub struct CovariantList<'a, T> {
    phantom: PhantomData<T>,
    capacity: ListLen,
    lookup_table: &'a [u8],
}

#[derive(Debug)]
pub struct List<'a, T> {
    pub inner: CovariantList<'a, T>,
    pub arena: &'a mut [u8],
}

impl<'a, T> CovariantList<'a, T> {
    #[inline]
    pub fn get<'b>(&self, arena: &'b mut [u8], idx: ListLen) -> T
    where
        T: Build<'b> + Find,
    {
        T::create(self.item_slice(arena, idx))
            .expect("List should have verified the integrity of the buffer on List::build(...)")
    }

    #[inline]
    pub fn item_slice<'b>(&self, arena: &'b mut [u8], idx: ListLen) -> &'b mut [u8]
    where
        T: Find,
    {
        assert!(idx < self.capacity);

        &mut arena[T::find(self.lookup_table, idx) as usize..]
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
        T: Find + Build<'b>,
    {
        let mut_self: &mut Self = &mut *(self as *const Self as *mut Self);

        self.inner.get(mut_self.arena, idx)
    }

    #[inline]
    pub fn borrow<'b>(&'b self, idx: ListLen, cb: impl Fn(&T))
    where
        T: Find + Build<'b>,
    {
        // VERIFY: This should be fine as we only give access to a &T.
        cb(&unsafe { self.get_from_imut(idx) });
    }

    #[inline]
    pub fn borrowable<'b>(&'b self, idx: ListLen) -> impl Borrow<T>
    where
        T: Find + Build<'b>,
    {
        // VERIFY: This should be fine as we only give access to a &T.
        unsafe { self.get_from_imut(idx) }
    }

    /// Helper function to coerce a closure's input type to this instance's T
    #[inline]
    pub fn coerce<F: Fn(T)>(&self, cb: F) -> F
    where
        T: Build<'a> + Find,
    {
        cb
    }

    #[inline]
    pub fn from(item_types: &[T]) -> DynamicItemListImprinter<T>
    where
        T: DynamicSize,
    {
        DynamicItemListImprinter {
            list_imprinter: ListImprinter {
                capacity: item_types.len() as ListLen,
                phantom: PhantomData,
            },
            items_sum_size: item_types.iter().map(|item| item.dsize()).sum(),
            item_types: item_types,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: ListLen) -> StaticItemListImprinter<T>
    where
        T: StaticSize,
    {
        StaticItemListImprinter {
            list_imprinter: ListImprinter {
                capacity: capacity,
                phantom: PhantomData,
            }
        }
    }
}

impl<'a, T> Find for List<'a, T> {
    type Strategy = DynamicFind;
}

impl<'a, T: Find + Build<'a>> Build<'a> for List<'a, T> {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<(&mut [u8], Self)> {
        // First bytes of list is it's length
        let (left, right) = arena.noser_split(LIST_LEN_SIZE)?;
        let capacity = ListLen::read(left);

        // The next bytes are the lookup table iff T is dynamically sized.
        let (lookup_table, arena) = T::get_lookup_table(right, capacity)?;

        // The rest is the arena of this list

        // Figure out the length of each element andd write it to the lookup table,
        // as we could panic if the lookup table recieved is invalid.
        // Also return Err if arena is undersized here insted of in get's.

        let mut running_size: Ptr = 0;

        for i in 0..capacity as usize {
            let size = T::read_size(&arena[running_size as usize..])?;
            running_size = running_size.checked_add(size).ok_or(::NoserError::IntegerOverflow)?;

            T::write_lookup_ptr(lookup_table, i, running_size);
        }

        let (arena, right) = arena.noser_split(running_size)?;

        Ok((
            right,
            List {
                arena,
                inner: CovariantList {
                    capacity,
                    lookup_table,
                    phantom: PhantomData,
                },
            },
        ))
    }
}

pub struct DynamicItemListImprinter<'a, A: 'a + DynamicSize> {
    list_imprinter: ListImprinter<A>,
    item_types: &'a [A],
    items_sum_size: Ptr,
}

pub struct StaticItemListImprinter<A: StaticSize> {
    list_imprinter: ListImprinter<A>,
}

struct ListImprinter<A> {
    capacity: ListLen,
    phantom: PhantomData<A>,
}

impl<'a, A: Find> Imprinter<'a> for ListImprinter<A> {
    type OnSuccess = (&'a mut [u8], &'a mut [u8]);

    #[inline]
    fn imprint(&self, arena: &'a mut [u8]) -> ::Result<Self::OnSuccess> {
        // First write the capacity of the list
        let (left, right) = arena.noser_split(LIST_LEN_SIZE)?;
        ListLen::write(left, self.capacity);

        // Then return the lookup table and arena for further initialization.
        Ok(A::get_lookup_table(right, self.capacity)?)
    }
}

impl<'a, A> Imprinter<'a> for StaticItemListImprinter<A> where A: StaticSize + Find {
    type OnSuccess = ::std::slice::ChunksMut<'a, u8>;

    #[inline]
    fn imprint(&self, arena: &'a mut [u8]) -> ::Result<Self::OnSuccess> {
        let capacity = self.list_imprinter.capacity;

        // Static item list don't need to initialize the lookup table
        let (_, right) = self.list_imprinter.imprint(arena)?;

        // Ensure the arena is large enough
        let (arena, _) = right.noser_split(capacity * A::size())?;

        // TODO: Use exact_chunks_mut when stable #47115
        Ok(arena.chunks_mut(A::size() as usize))
    }
}

impl<'a, 'b, A: DynamicSize + Find + Imprinter<'a>> Imprinter<'a> for DynamicItemListImprinter<'b, A> {
    type OnSuccess = ();

    #[inline]
    fn imprint(&self, arena: &'a mut [u8]) -> ::Result<Self::OnSuccess> {
        let (lookup_table, right) = self.list_imprinter.imprint(arena)?;

        let mut running_size = 0;

        // Fill the lookup table
        for (kind, chunk) in self.item_types.iter().zip(lookup_table.chunks_mut(Ptr::size() as usize)) {
            running_size += kind.dsize();
            Ptr::write(chunk, running_size);
        }

        let cell = Cell::new(right);

        // Call nested imprinters
        for kind in self.item_types {
            let (left, right) = cell.take().noser_split(kind.dsize())?;
            kind.imprint(left)?;
            cell.set(right);
        }

        // Ok(self.item_types().map())
        Ok(())
    }
}


impl<'a, A: DynamicSize> DynamicSize for DynamicItemListImprinter<'a, A> {
    #[inline]
    fn dsize(&self) -> Ptr {
        // items_size + lookup table + len
        self.items_sum_size + self.list_imprinter.capacity as Ptr * LTP_SIZE + LIST_LEN_SIZE
    }
}

impl<'a, A: DynamicSize> Find for DynamicItemListImprinter<'a, A> {
    type Strategy = DynamicFind;
}

impl<A: StaticSize> DynamicSize for StaticItemListImprinter<A> {
    #[inline]
    fn dsize(&self) -> Ptr {
        // arena + capacity
        self.list_imprinter.capacity * A::size() + LIST_LEN_SIZE
    }
}

impl<A: StaticSize> Find for StaticItemListImprinter<A> {
    type Strategy = DynamicFind;
}
