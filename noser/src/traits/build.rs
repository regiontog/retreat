pub trait Build<'a>: Sized {
    // TODO: Associated type for arena (AsRef, AsMut)
    fn build(&'a mut [u8]) -> Result<'a, Self>;

    unsafe fn unchecked_build(&'a mut [u8]) -> Self;

    #[inline]
    fn create(arena: &'a mut [u8]) -> ::Result<Self> {
        Self::build(arena).map(|(_, this)| this)
    }
}

pub type Result<'a, T> = ::Result<(&'a mut [u8], T)>;
