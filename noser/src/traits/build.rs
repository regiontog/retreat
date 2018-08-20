pub trait Build<'a>: Sized {
    // TODO: Associated type for arena (AsRef, AsMut)
    fn build(&'a mut [u8]) -> Result<'a, Self>;

    #[inline]
    fn create(arena: &'a mut [u8]) -> ::Result<Self> {
        Self::build(arena).map(|(_, this)| this)
    }

    #[inline]
    fn read_size(arena: &[u8]) -> ::Result<::Ptr> {
        // We don't even bind the Self part of the result, so this should be safe
        let mut_arena: &mut [u8] = unsafe { &mut *(arena as *const [u8] as *mut [u8]) };

        let (right, _) = Self::build(mut_arena)?;
        Ok(arena.len() as ::Ptr - right.len() as ::Ptr)
    }
}

pub type Result<'a, T> = ::Result<(&'a mut [u8], T)>;
