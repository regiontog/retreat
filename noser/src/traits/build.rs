pub trait Build<'a>: Sized {
    // TODO: Associated type for arena (AsRef, AsMut)
    fn build(_: &'a mut [u8]) -> Result<'a, Self>;

    unsafe fn unchecked_build(_: &'a mut [u8]) -> Self;

    #[inline]
    fn create(arena: &'a mut [u8]) -> crate::Result<Self> {
        Self::build(arena).map(|(_, this)| this)
    }
}

pub type Result<'a, T> = crate::Result<(&'a mut [u8], T)>;
