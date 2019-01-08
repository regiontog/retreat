/// The build and unchecked_build functions must not mutate the arena even though it has a mut ref,
/// mutation is only allowed through the methods and members of Self.
/// This is because the MutRefInput may be unsafely converted from &Self::Input in order to
/// create a [ReadOnly<Self>](freyr::utils::ReadOnly<Self>) or to find the unused portion of some Self::Input.
pub unsafe trait Build<'a>: Sized {
    fn build<'n>(_: &'n mut [u8]) -> crate::Result<(&'n mut [u8], Self)>
    where
        'n: 'a;

    /// May panic
    fn unchecked_build<'n>(_: &'n mut [u8]) -> (&'n mut [u8], Self)
    where
        'n: 'a;

    #[inline]
    fn create<'n>(input: &'n mut [u8]) -> crate::Result<Self>
    where
        'n: 'a,
    {
        Self::build(input).map(|(_, this)| this)
    }

    #[inline]
    fn unused<'b>(input: &'b [u8]) -> crate::Result<&'b [u8]> {
        // Self::build MUST not mutate the arena itself. Because we drop Self immediately nobody
        // else can, the only one who might is Self::build.
        let input = unsafe { &mut *(input as *const [u8] as *mut [u8]) };

        Self::build(input).map(|(unused, _)| &unused[..])
    }

    /// May panic
    #[inline]
    fn unchecked_create<'n>(input: &'n mut [u8]) -> Self
    where
        'n: 'a,
    {
        let (_, this) = Self::unchecked_build(input);
        this
    }

    #[inline]
    fn create_read_only(input: &'a [u8]) -> crate::Result<freyr::ReadOnly<Self>> {
        // Self::build MUST not mutate the arena itself. Because we return ReadOnly<Self> nobody
        // else can, the only one who might is Self::build.

        let input = unsafe { &mut *(input as *const [u8] as *mut [u8]) };

        let mut_this = Self::create(input)?;
        Ok(freyr::ReadOnly::new(mut_this))
    }
}
