pub trait Build<'b>: Sized {
    // TODO: Associated type for arena (AsRef, AsMut)
    fn build<'a>(_: &'a mut [u8]) -> Result<'a, Self>
    where
        'a: 'b;

    unsafe fn unchecked_build<'a>(_: &'a mut [u8]) -> Self
    where
        'a: 'b;

    #[inline]
    fn create<'a>(arena: &'a mut [u8]) -> crate::Result<Self>
    where
        'a: 'b,
    {
        Self::build(arena).map(|(_, this)| this)
    }
}

pub type Result<'a, T> = crate::Result<(&'a mut [u8], T)>;
