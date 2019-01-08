pub trait Read<'r> {
    type Output;
    const OUT_SIZE: usize = std::mem::size_of::<Self::Output>();

    fn read<'a>(_: &'a [u8]) -> Self::Output
    where
        'a: 'r;

    fn read_safe<'a>(arena: &'a [u8]) -> crate::Result<Self::Output>
    where
        'a: 'r,
    {
        if arena.len() < Self::OUT_SIZE {
            Err(crate::NoserError::Undersized(
                Self::OUT_SIZE,
                arena.to_vec(),
            ))
        } else {
            Ok(Self::read(arena))
        }
    }
}
