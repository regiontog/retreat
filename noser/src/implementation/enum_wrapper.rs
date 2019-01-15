use crate::prelude::SliceExt;
use crate::traits::{StaticEnum, Tagged};

const U64LEN: usize = ::std::mem::size_of::<u64>();

#[inline]
pub fn write_var_len_int(buffer: &mut [u8], len: usize, int: u64) {
    let int = unsafe { ::std::mem::transmute::<u64, [u8; U64LEN]>(int.to_le()) };

    buffer[..len].clone_from_slice(&int[..len]);
}

#[inline]
pub fn read_var_len_int(buffer: &[u8], len: usize) -> u64 {
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
    E: StaticEnum<'a>,
{
    #[inline]
    pub fn reinterpret(mut self, to: &E::VariantEnum) -> Result<Self, (Self, crate::NoserError)> {
        let old_tag = read_var_len_int(self.variant_bytes, E::VARIANT_BYTES);

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
                write_var_len_int(self.variant_bytes, E::VARIANT_BYTES, to.variant_tag());
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
impl<'a, E> crate::traits::size::Sizeable for EnumWrapper<'a, E>
where
    E: StaticEnum<'a>,
{
    type Strategy = crate::traits::size::Static;

    #[inline]
    fn read_size(_: &[u8]) -> crate::traits::size::ReadReturn<Self> {
        Ok(E::static_size())
    }
}

unsafe impl<'a, E> crate::traits::Build<'a> for EnumWrapper<'a, E>
where
    E: StaticEnum<'a>,
{
    #[inline]
    fn build<'w>(arena: &'w mut [u8]) -> crate::Result<(&'w mut [u8], Self)>
    where
        'w: 'a,
    {
        let (variant_bytes, arena) = arena.noser_split(E::VARIANT_BYTES as crate::Ptr)?;
        let (arena, right) = arena.noser_split(*E::CONTENTS_SIZE.get_or_insert(0) as crate::Ptr)?;

        let tag = read_var_len_int(variant_bytes, E::VARIANT_BYTES);

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
        let (variant_bytes, arena) = arena.split_at_mut(E::VARIANT_BYTES);
        let (arena, right) = arena.split_at_mut(E::contents_size());

        let tag = read_var_len_int(variant_bytes, E::VARIANT_BYTES);

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
