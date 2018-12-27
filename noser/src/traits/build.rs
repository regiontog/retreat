pub trait Build<'b>: Sized {
    /// This function must not mutate the arena even though it has a mut ref,
    /// mutation is only allowed through the methods and members of Self.
    /// This is because the mut ref may be a unsafe imut ref in order to
    /// create a [ReadOnly<Self>](freyr::utils::ReadOnly<Self>).
    fn build<'a>(_: &'a mut [u8]) -> Result<'a, Self>
    where
        'a: 'b;

    /// This function must not mutate the arena even though it has a mut ref,
    /// mutation is only allowed through the methods and members of Self.
    /// This is because the mut ref may be a unsafe imut ref in order to
    /// create a [ReadOnly<Self>](freyr::utils::ReadOnly<Self>).
    unsafe fn unchecked_build<'a>(_: &'a mut [u8]) -> (&'a mut [u8], Self)
    where
        'a: 'b;

    #[inline]
    fn create<'a>(arena: &'a mut [u8]) -> crate::Result<Self>
    where
        'a: 'b,
    {
        Self::build(arena).map(|(_, this)| this)
    }

    #[inline]
    fn unused(arena: &[u8]) -> crate::Result<&[u8]> {
        // Self::build MUST not mutate the arena itself. Because we drop Self immediately nobody
        // else can, the only one who might is Self::build.
        let mut_arena = unsafe { &mut *(arena as *const [u8] as *mut [u8]) };

        Self::build(mut_arena).map(|(unused, _)| &unused[..])
    }

    #[inline]
    unsafe fn unchecked_create<'a>(arena: &'a mut [u8]) -> Self
    where
        'a: 'b,
    {
        let (_, this) = Self::unchecked_build(arena);
        this
    }

    #[inline]
    fn create_read_only<'a>(arena: &'a [u8]) -> crate::Result<freyr::utils::ReadOnly<Self>>
    where
        'a: 'b,
    {
        // Self::build MUST not mutate the arena itself. Because we return ReadOnly<Self> nobody
        // else can, the only one who might is Self::build.
        let mut_arena = unsafe { &mut *(arena as *const [u8] as *mut [u8]) };
        let mut_this = Self::create(mut_arena)?;
        Ok(freyr::utils::ReadOnly::new(mut_this))
    }
}

pub type Result<'a, T> = crate::Result<(&'a mut [u8], T)>;
