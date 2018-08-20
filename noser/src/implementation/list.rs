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
            .expect("List should have verified the length of the buffer on initialization.")
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
    pub fn from(item_types: Vec<T>) -> DynamicItemListImprinter<T>
    where
        T: DynamicSize,
    {
        DynamicItemListImprinter {
            items_sum_size: item_types.iter().map(|item| item.dsize()).sum(),
            item_types: item_types,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: ListLen) -> StaticItemListImprinter
    where
        T: StaticSize,
    {
        StaticItemListImprinter {
            item_size: T::size(),
            capacity: capacity,
        }
    }
}

impl<'a, T> Find for List<'a, T> {
    type Strategy = DynamicFind;
}

impl<'a, T: Find> Build<'a> for List<'a, T> {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<(&mut [u8], Self)> {
        // First bytes of list is it's length
        let (left, right) = arena.noser_split(LIST_LEN_SIZE)?;
        let capacity = ListLen::read(left);

        // The next bytes are the lookup table iff T is dynamically sized.
        let (lookup_table, arena) = T::get_lookup_table(right, capacity)?;

        // The rest is the arena of this list
        let size = T::find(lookup_table, capacity);
        let (arena, right) = arena.noser_split(size)?;

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

pub struct DynamicItemListImprinter<T: DynamicSize> {
    item_types: Vec<T>,
    items_sum_size: Ptr,
}

pub struct StaticItemListImprinter {
    capacity: ListLen,
    item_size: Ptr,
}

impl<'a, A: DynamicSize + Imprinter<'a>> Imprinter<'a> for DynamicItemListImprinter<A> {
    #[inline]
    fn imprint(self, arena: &'a mut [u8]) -> ::Result<()> {
        let cap = self.item_types.len() as ListLen;

        // First write the capacity of the list
        let (left, right) = arena.noser_split(LIST_LEN_SIZE)?;
        ListLen::write(left, cap);

        // Then the lookup table
        let lookup_table_size = cap * LIST_LEN_SIZE;
        let (lookup_table, right) = right.noser_split(lookup_table_size)?;

        let mut running_size = 0;
        let mut ptr = 0;

        let cell = Cell::new(right);
        for kind in self.item_types {
            // TODO: Cleanup
            // Write a Ptr to each T in the lookup table
            let size = kind.dsize();
            running_size += size;
            Ptr::write(&mut lookup_table[(ptr as usize)..], running_size);
            ptr += LTP_SIZE;

            // Run nested imprinters
            let (left, right) = cell.take().noser_split(size)?;
            kind.imprint(left)?;
            cell.set(right);
        }

        Ok(())
    }
}

impl<'a> Imprinter<'a> for StaticItemListImprinter {
    #[inline]
    fn imprint(self, arena: &'a mut [u8]) -> ::Result<()> {
        // First bytes of list is it's length
        let (left, right) = arena.noser_split(LIST_LEN_SIZE as Ptr)?;
        ListLen::write(left, self.capacity);

        for chunk in right
            .chunks_mut(self.item_size as usize)
            .take(self.capacity as usize)
        {
            // TODO: nested imprints
        }

        right.validate_size(self.item_size as Ptr * self.capacity)?;
        Ok(())
    }
}

impl<A: DynamicSize> DynamicSize for DynamicItemListImprinter<A> {
    #[inline]
    fn dsize(&self) -> Ptr {
        // items_size + lookup table + len
        self.items_sum_size + self.item_types.len() as Ptr * LTP_SIZE + LIST_LEN_SIZE
    }
}

impl DynamicSize for StaticItemListImprinter {
    #[inline]
    fn dsize(&self) -> Ptr {
        // arena + capacity
        self.capacity * self.item_size + LIST_LEN_SIZE
    }
}
