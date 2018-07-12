extern crate boxfnonce;

use std::cell::Cell;
use std::marker::PhantomData;

trait DynamicSize {
    fn size(&self) -> u32;
}

trait StaticSize {
    fn size() -> u32;
}

trait NoserWithArena<'a> {
    fn with_arena(&'a mut [u8]) -> Self;
}

trait NoserDynamicWithArena<'a, T> {
    fn with_arena(self, &'a mut [u8]) -> T;
}

struct NoserBorrowed<'a, T> {
    arena: &'a mut [u8],
    phantom: PhantomData<T>,
}

impl<'a, T: StaticSize> StaticSize for NoserBorrowed<'a, T> {
    fn size() -> u32 {
        T::size()
    }
}

impl<'a, T> NoserWithArena<'a> for NoserBorrowed<'a, T> {
    fn with_arena(arena: &'a mut [u8]) -> Self {
        NoserBorrowed {
            arena: arena,
            phantom: PhantomData,
        }
    }
}

trait NoserType<T> {
    fn write(&mut self, val: T);
    fn read(&self) -> T;
}

impl<'a> NoserType<u8> for NoserBorrowed<'a, u8> {
    fn write(&mut self, val: u8) {
        self.arena[0] = val;
    }

    fn read(&self) -> u8 {
        self.arena[0]
    }
}

impl StaticSize for u8 {
    fn size() -> u32 {
        1
    }
}

struct NoserList<T> {
    items: Vec<T>,
}

use boxfnonce::BoxFnOnce;

struct NoserListFactory<'a, T> {
    size: u32,
    items_factory: BoxFnOnce<'a, (&'a mut [u8],), Vec<T>>,
}

impl<'a, T> DynamicSize for NoserListFactory<'a, T> {
    fn size(&self) -> u32 {
        self.size
    }
}

impl<'a, T> NoserDynamicWithArena<'a, NoserList<T>> for NoserListFactory<'a, T> {
    fn with_arena(self, arena: &'a mut [u8]) -> NoserList<T> {
        NoserList {
            items: self.items_factory.call(arena),
        }
    }
}

impl<'a, R> NoserListFactory<'a, R> {
    fn with<A: 'a + DynamicSize + NoserDynamicWithArena<'a, R>>(
        items: Vec<A>,
    ) -> NoserListFactory<'a, R> {
        NoserListFactory {
            size: items.iter().map(|item| item.size()).sum(),
            items_factory: BoxFnOnce::from(move |arena: &'a mut [u8]| {
                let cell = Cell::new(arena);
                items
                    .into_iter()
                    .map(|item| {
                        let (left, right) = cell.take().split_at_mut(item.size() as usize);

                        cell.set(right);
                        item.with_arena(left)
                    })
                    .collect()
            }),
        }
    }
}

impl<'a, T: StaticSize + NoserWithArena<'a>> NoserListFactory<'a, T> {
    fn with_capacity(len: u32) -> Self {
        let size = T::size();

        NoserListFactory {
            size: len * size,
            items_factory: BoxFnOnce::from(move |arena: &'a mut [u8]| {
                let mut items = Vec::with_capacity(len as usize);
                let cell = Cell::new(arena);

                for _ in 0..(len / size) as usize {
                    let (left, right) = cell.take().split_at_mut(size as usize);

                    items.push(T::with_arena(left));
                    cell.set(right);
                }

                items
            }),
        }
    }
}

impl<T> NoserList<T> {
    fn get(&mut self, idx: u32) -> &mut T {
        &mut self.items[idx as usize]
    }
}

#[cfg(test)]
mod test {
    use *;

    #[test]
    fn test() {
        let ref mut arena = [0; 10];

        {
            let mut l = NoserListFactory::with(vec![
                NoserListFactory::<NoserBorrowed<u8>>::with_capacity(2),
                NoserListFactory::with_capacity(2),
            ]).with_arena(arena);

            l.get(0).get(0).write(10);
            assert_eq!(l.get(0).get(0).read(), 10);
            l.get(0).get(1).write(11);
            assert_eq!(l.get(0).get(0).read(), 10);
            assert_eq!(l.get(0).get(1).read(), 11);
            l.get(1).get(0).write(12);
            assert_eq!(l.get(0).get(0).read(), 10);
            assert_eq!(l.get(0).get(1).read(), 11);
            assert_eq!(l.get(1).get(0).read(), 12);
            l.get(1).get(1).write(13);
            assert_eq!(l.get(0).get(0).read(), 10);
            assert_eq!(l.get(0).get(1).read(), 11);
            assert_eq!(l.get(1).get(0).read(), 12);
            assert_eq!(l.get(1).get(1).read(), 13);
        }

        println!("{:?}", arena);
        panic!();
    }
}
