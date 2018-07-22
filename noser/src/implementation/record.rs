use traits::WithArena;

pub struct Record<'a, S> {
    size: usize,
    create_struct: ::boxfnonce::BoxFnOnce<'a, (&'a mut [u8],), S>,
}

impl<'a, S> Record<'a, S> {
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
    fn with_arena(self, arena: &'a mut [u8]) -> S {
        self.create_struct.call(arena)
    }
}
