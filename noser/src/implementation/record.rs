use traits::WithArena;

pub struct Record<'a, S> {
    create_struct: ::boxfnonce::BoxFnOnce<'a, (&'a mut [u8],), ::Result<'a, (&'a mut [u8], S)>>,
}

impl<'a, S> Record<'a, S> {
    #[inline]
    pub fn new<F: 'a>(create: F) -> Record<'a, S>
    where
        F: FnOnce(&'a mut [u8]) -> ::Result<'a, (&'a mut [u8], S)>,
    {
        Record {
            create_struct: ::boxfnonce::BoxFnOnce::new(create),
        }
    }
}

impl<'a, S> WithArena<'a, S> for Record<'a, S> {
    #[inline]
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], S)> {
        self.create_struct.call(arena)
    }
}
