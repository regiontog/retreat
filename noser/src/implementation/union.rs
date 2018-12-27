#[inline]
pub fn write_var_len_int(buffer: &mut [u8], len: usize, int: u64) {
    const U64LEN: usize = ::std::mem::size_of::<u64>();
    let int = unsafe { ::std::mem::transmute::<u64, [u8; U64LEN]>(int.to_le()) };

    buffer[..len].clone_from_slice(&int[..len]);
}

#[inline]
pub fn read_var_len_int(buffer: &[u8], len: usize) -> u64 {
    const U64LEN: usize = ::std::mem::size_of::<u64>();
    let mut int = [0; U64LEN];

    int[..len].clone_from_slice(&buffer[..len]);

    u64::from_le(unsafe { ::std::mem::transmute::<[u8; U64LEN], u64>(int) })
}

// impl<'a, V: Variants<'a>> Build for V {
//     #[inline]
//     fn build(arena: &'a mut [u8]) -> crate::Result<(&'a mut [u8], Self)> {
//         let variant_bytes = Self::bytes_for_n_variants();
//         let (left, right) = arena.split_at_mut(variant_bytes);

//         Self::variant(read_var_len_int(left, variant_bytes), right)
//     }

//     #[inline]
//     unsafe fn unchecked_build(_arena: &'a mut [u8]) -> Self {
//         unimplemented!()
//     }
// }
