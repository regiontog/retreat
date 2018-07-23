use ext::SliceExt;
use traits::{Build, Read, StaticSize, WithArena, Write};

use std::cell::Cell;
use std::ops::{Index, IndexMut};

use boxfnonce::BoxFnOnce;

type ListLen = u32;

#[derive(Debug)]
pub struct List<T> {
    items: Vec<T>,
}

pub struct ListFactory<'a, T> {
    items_factory: BoxFnOnce<'a, (&'a mut [u8],), Result<(&'a mut [u8], Vec<T>), ::NoserError<'a>>>,
}

impl<T> Index<ListLen> for List<T> {
    type Output = T;

    #[inline]
    fn index(&self, idx: ListLen) -> &T {
        &self.items[idx as usize]
    }
}

impl<T> IndexMut<ListLen> for List<T> {
    #[inline]
    fn index_mut(&mut self, idx: ListLen) -> &mut T {
        &mut self.items[idx as usize]
    }
}

impl<'a, R> WithArena<'a, List<R>> for ListFactory<'a, R> {
    #[inline]
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], List<R>)> {
        let (right, items) = self.items_factory.call(arena)?;
        Ok((right, List { items: items }))
    }
}

impl<T> List<T> {
    #[inline]
    pub fn len(&self) -> u32 {
        self.items.len() as u32
    }

    #[inline]
    pub fn with<'a, A: 'a + WithArena<'a, T>>(item_types: Vec<A>) -> ListFactory<'a, T> {
        ListFactory {
            items_factory: BoxFnOnce::from(move |arena: &'a mut [u8]| {
                // First bytes of list is it's length
                let (left, right) = arena.noser_split(ListLen::size() as usize)?;
                ListLen::write(left, item_types.len() as ListLen);

                // Rest is it's contents
                let mut items = Vec::with_capacity(item_types.len());
                let cell = Cell::new(right);

                for kind in item_types {
                    let (right, item) = kind.with_arena(cell.take())?;
                    cell.set(right);
                    items.push(item);
                }

                Ok((cell.into_inner(), items))
            }),
        }
    }
}

impl<'a, T: Build<'a>> Build<'a> for List<T> {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], Self)> {
        // First bytes of list is it's length
        let (left, right) = arena.noser_split(ListLen::size() as usize)?;
        let len = ListLen::read(left);

        // Rest is it's contents
        let (right, items) = Self::get_items(right, len as usize)?;
        Ok((right, List { items: items }))
    }
}

impl<'a, T: Build<'a>> List<T> {
    #[inline]
    pub fn with_capacity(len: ListLen) -> ListFactory<'a, T> {
        ListFactory {
            items_factory: BoxFnOnce::from(move |arena: &'a mut [u8]| {
                // First bytes of list is it's length
                let (left, right) = arena.noser_split(ListLen::size() as usize)?;
                ListLen::write(left, len);

                // Rest is it's contents
                Self::get_items(right, len as usize)
            }),
        }
    }

    #[inline]
    fn get_items(arena: &'a mut [u8], len: usize) -> ::Result<'a, (&'a mut [u8], Vec<T>)> {
        // Guard for invalid data where reported length implies a size larger than the arena's length
        // Since we don't know the actual size of T we assume it is the smallest possible size (1)

        if arena.len() < len {
            return Err(::NoserError::Undersized(len, arena));
        }

        let mut items = Vec::with_capacity(len);
        let cell = Cell::new(arena);

        for _ in 0..len {
            let (right, item) = T::build(cell.take())?;

            items.push(item);
            cell.set(right);
        }

        Ok((cell.into_inner(), items))
    }
}
