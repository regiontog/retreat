use traits::{DynamicSize, WithArena};

pub struct Record<'a, S> {
    size: usize,
    create_struct: ::boxfnonce::BoxFnOnce<'a, (&'a mut [u8],), S>,
}

impl<'a, S> Record<'a, S> {
    #[inline]
    pub fn new<F: 'a>(size: usize, create: F) -> Record<'a, S>
    where
        F: FnOnce(&'a mut [u8]) -> S,
    {
        Record {
            size: size,
            create_struct: ::boxfnonce::BoxFnOnce::new(create),
        }
    }
}

impl<'a, S> WithArena<'a, S> for Record<'a, S> {
    #[inline]
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, S> {
        Ok(self.create_struct.call(arena))
    }
}

impl<'a, S> DynamicSize for Record<'a, S> {
    #[inline]
    fn dsize(&self) -> usize {
        self.size
    }
}
