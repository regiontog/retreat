use traits::{Build, DynamicSize, Read, StaticSize, WithArena, Write};

use std::cell::Cell;
use std::ops::{Index, IndexMut};

use boxfnonce::BoxFnOnce;

type ListLen = u32;

#[derive(Debug)]
pub struct List<T> {
    items: Vec<T>,
}

pub struct ListFactory<'a, T> {
    size: usize,
    items_factory: BoxFnOnce<'a, (&'a mut [u8],), Result<Vec<T>, ::NoserError<'a>>>,
}

use std::fmt::Debug;

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

impl<'a, T> DynamicSize for ListFactory<'a, T> {
    #[inline]
    fn dsize(&self) -> usize {
        self.size
    }
}

impl<'a, R> WithArena<'a, List<R>> for ListFactory<'a, R> {
    #[inline]
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, List<R>> {
        Ok(List {
            items: self.items_factory.call(arena)?,
        })
    }
}

impl<T> List<T> {
    #[inline]
    pub fn len(&self) -> u32 {
        self.items.len() as u32
    }

    #[inline]
    pub fn with<'a, A: 'a + DynamicSize + WithArena<'a, T>>(items: Vec<A>) -> ListFactory<'a, T> {
        let size = ListLen::size() + items.iter().map(|item| item.dsize()).sum::<usize>();
        ListFactory {
            size: size,
            items_factory: BoxFnOnce::from(move |arena: &'a mut [u8]| {
                if arena.len() < size {
                    return Err(::NoserError::Undersized(size, arena));
                }

                // First bytes of list is it's length
                let (left, right) = arena.split_at_mut(ListLen::size() as usize);
                ListLen::write(left, items.len() as ListLen);

                // Rest is it's contents
                let cell = Cell::new(right);
                items
                    .into_iter()
                    .map(|item| {
                        let (left, right) = cell.take().split_at_mut(item.dsize() as usize);

                        cell.set(right);
                        item.with_arena(left)
                    })
                    .collect()
            }),
        }
    }
}

impl<'a, T: StaticSize + Build<'a>> List<T> {
    #[inline]
    pub fn with_capacity(len: ListLen) -> ListFactory<'a, T> {
        let size = T::size();

        ListFactory {
            size: ListLen::size() + len as usize * size,
            items_factory: BoxFnOnce::from(move |arena: &'a mut [u8]| {
                if arena.len() < size {
                    return Err(::NoserError::Undersized(size, arena));
                }

                // First bytes of list is it's length
                let (left, right) = arena.split_at_mut(ListLen::size() as usize);
                ListLen::write(left, len);

                // Rest is it's contents
                let mut items = Vec::with_capacity(len as usize);
                let cell = Cell::new(right);

                for _ in 0..(len / size as ListLen) {
                    let (right, item) = T::build(cell.take())?;

                    items.push(item);
                    cell.set(right);
                }

                Ok(items)
            }),
        }
    }
}

impl<'a, T: Debug + Build<'a>> Build<'a> for List<T> {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], Self)> {
        if arena.len() < ListLen::size() {
            return Err(::NoserError::Undersized(ListLen::size(), arena));
        }

        let (left, right) = arena.split_at_mut(ListLen::size());
        let len = ListLen::read(left);

        if right.len() < len as usize {
            return Err(::NoserError::Undersized(len as usize, right));
        }

        let cell = Cell::new(right);
        let mut items = Vec::with_capacity(len as usize);

        for _ in 0..len {
            let (right, item) = T::build(cell.take())?;

            cell.set(right);
            items.push(item)
        }

        Ok((cell.into_inner(), List { items: items }))
    }
}
