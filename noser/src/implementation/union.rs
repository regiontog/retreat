use boxfnonce::BoxFnOnce;
use traits::{Build, Variants, WithArena};

#[inline]
pub fn write_var_len_int(buffer: &mut [u8], len: usize, int: u64) {
    const U64LEN: usize = ::std::mem::size_of::<u64>();
    let int = unsafe { ::std::mem::transmute::<u64, [u8; U64LEN]>(int.to_le()) };

    for i in 0..len {
        buffer[i] = int[i];
    }
}

#[inline]
pub fn read_var_len_int(buffer: &[u8], len: usize) -> u64 {
    const U64LEN: usize = ::std::mem::size_of::<u64>();
    let mut int = [0; U64LEN];

    for i in 0..len {
        int[i] = buffer[i];
    }

    u64::from_le(unsafe { ::std::mem::transmute::<[u8; U64LEN], u64>(int) })
}

impl<'a, V: Variants<'a>> Build<'a> for V {
    #[inline]
    fn build(arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], Self)> {
        let variant_bytes = Self::bytes_for_n_variants();
        let (left, right) = arena.split_at_mut(variant_bytes);

        Self::variant(read_var_len_int(left, variant_bytes), right)
    }
}

pub struct Union<'a, E: Variants<'a>> {
    val: BoxFnOnce<'a, (&'a mut [u8],), ::Result<'a, (&'a mut [u8], E)>>,
}

impl<'a, E: Variants<'a>> Union<'a, E> {
    #[inline]
    pub fn new<F: 'a + FnOnce(&'a mut [u8]) -> ::Result<'a, (&'a mut [u8], E)>>(
        val: F,
    ) -> ::Union<'a, E> {
        ::Union {
            val: BoxFnOnce::new(|arena: &'a mut [u8]| {
                let (left, right) = arena.split_at_mut(E::bytes_for_n_variants());
                let (right, variant) = val(right)?;
                write_var_len_int(left, E::bytes_for_n_variants(), variant.ord());
                Ok((right, variant))
            }),
        }
    }

    #[inline]
    pub fn with_variant(into_variant: impl Into<Union<'a, E>>) -> Self {
        into_variant.into()
    }
}

impl<'a, E: Variants<'a>> WithArena<'a, E> for Union<'a, E> {
    #[inline]
    fn with_arena(self, arena: &'a mut [u8]) -> ::Result<'a, (&'a mut [u8], E)> {
        self.val.call(arena)
    }
}
