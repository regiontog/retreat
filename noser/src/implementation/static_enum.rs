use crate::prelude::SliceExt;
use crate::traits::{StaticEnum, Tagged};

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

pub struct EnumWrapper<'a, E> {
    inner: freyr::AliasGuard<&'a mut [u8], E>,
    variant_bytes: &'a mut [u8],
}

impl<'a, E> EnumWrapper<'a, E>
where
    E: StaticEnum,
{
    #[inline]
    pub fn reinterpret(mut self, to: &E::VariantEnum) -> Result<Self, (Self, crate::NoserError)> {
        let old_tag = read_var_len_int(self.variant_bytes, E::variant_bytes());

        let inner = freyr::AliasGuard::try_new(self.inner.move_first(), |arena| {
            E::construct_variant(to, arena)
        })
        .map_err(|(arena, e)| {
            (
                freyr::AliasGuard::new(arena, |arena| {
                    E::construct_variant(
                        &E::VariantEnum::from_tag(old_tag).expect(
                            "Since we had a instance of Self the old tag value must be valid!",
                        ),
                        arena,
                    )
                    .expect("Since we had a instance of Self the arena must be valid!")
                }),
                e,
            )
        });

        match inner {
            Ok(inner) => {
                write_var_len_int(self.variant_bytes, E::variant_bytes(), to.variant_tag());
                self.inner = inner;
                Ok(self)
            }
            Err((inner, e)) => {
                self.inner = inner;
                Err((self, e))
            }
        }
    }
}
impl<E> crate::traits::size::Sizeable for EnumWrapper<'_, E>
where
    E: StaticEnum,
{
    type Strategy = crate::traits::size::Static;

    #[inline]
    fn read_size(_: &[u8]) -> crate::traits::size::ReadReturn<Self> {
        Ok((E::variant_bytes() + E::static_size()) as crate::Ptr)
    }
}

unsafe impl<'a, E> crate::traits::Build<'a> for EnumWrapper<'a, E>
where
    E: StaticEnum,
{
    #[inline]
    fn build<'w>(arena: &'w mut [u8]) -> crate::Result<(&'w mut [u8], Self)>
    where
        'w: 'a,
    {
        let (variant_bytes, arena) = arena.noser_split(E::variant_bytes() as crate::Ptr)?;
        let (arena, right) = arena.noser_split(E::static_size() as crate::Ptr)?;

        let tag = read_var_len_int(variant_bytes, E::variant_bytes());

        let inner = freyr::AliasGuard::try_new(arena, |arena| {
            E::construct_variant(&E::VariantEnum::from_tag(tag)?, arena)
        })
        .map_err(|(_, e)| e)?;

        Ok((
            right,
            EnumWrapper {
                inner,
                variant_bytes,
            },
        ))
    }

    #[inline]
    fn unchecked_build<'w>(arena: &'w mut [u8]) -> (&'w mut [u8], Self)
    where
        'w: 'a,
    {
        let (variant_bytes, arena) = arena.split_at_mut(E::variant_bytes());
        let (arena, right) = arena.split_at_mut(E::static_size());

        let tag = read_var_len_int(variant_bytes, E::variant_bytes());

        let inner = freyr::AliasGuard::new(arena, |arena| {
            E::unchecked_construct_variant(&E::VariantEnum::from_tag(tag).unwrap(), arena)
        });

        (
            right,
            EnumWrapper {
                inner,
                variant_bytes,
            },
        )
    }
}
