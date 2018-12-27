use crate::traits::WriteTypeInfo;
use std::mem;

use crate::traits::{
    size::{ReadReturn, Static},
    Read, Sizable, Write,
};

macro_rules! impl_rw {
    ($imp_name:ident, $ty:ident, $($rw:tt)*) => {
        $($rw)*

        #[allow(non_snake_case)]
        mod $imp_name {
            pub struct ScalarImprinter;
        }

        impl WriteTypeInfo<$ty> for $imp_name::ScalarImprinter {
            #[inline]
            fn imprint(&self, arena: &mut [u8]) -> crate::Result<()> {
                if arena.len() >= mem::size_of::<$ty>() {
                    // Scalars don't need to write any size information
                    Ok(())
                } else {
                    Err(crate::NoserError::Undersized(mem::size_of::<$ty>(), arena.to_vec()))
                }
            }

            #[inline]
            fn result_size(&self) -> crate::Ptr {
                mem::size_of::<$ty>() as crate::Ptr
            }
        }


        pub static $imp_name: $imp_name::ScalarImprinter = $imp_name::ScalarImprinter {};

        impl Default for &WriteTypeInfo<$ty> {
            fn default() -> &'static WriteTypeInfo<$ty> {
                &$imp_name
            }
        }

        impl Sizable for $ty {
            type Strategy = Static;

            #[inline]
            fn read_size(_: &[u8]) -> ReadReturn<crate::Ptr> {
                Ok(mem::size_of::<$ty>() as crate::Ptr)
            }
        }
    };
}

macro_rules! transmutable_without_endianness_transform {
    ($imp_name:ident, $ty:ident) => {
        impl_rw!($imp_name, $ty,
            impl Write for $ty {
                #[inline]
                fn write(arena: &mut [u8], val: $ty) {
                    #[allow(clippy::cast_ptr_alignment)]
                    let mut_ptr = (&mut arena[..mem::size_of::<$ty>()]).as_mut_ptr() as *mut $ty;
                    unsafe { std::ptr::write_unaligned(mut_ptr, val) }
                }
            }

            impl<'r> Read<'r> for $ty {
                type Output = $ty;

                #[inline]
                fn read<'a>(arena: &'a [u8]) -> $ty where 'a: 'r {
                    #[allow(clippy::cast_ptr_alignment)]
                    let p = (&arena[..mem::size_of::<$ty>()]).as_ptr() as *const $ty;
                    unsafe { std::ptr::read_unaligned(p) }
                }
            }
        );
    };
}

macro_rules! transmutable {
    ($imp_name:ident, $ty:ident) => {
        impl_rw!($imp_name, $ty,
            impl Write for $ty {
                #[inline]
                fn write(arena: &mut [u8], val: $ty) {
                    #[allow(clippy::cast_ptr_alignment)]
                    let mut_ptr = (&mut arena[..mem::size_of::<$ty>()]).as_mut_ptr() as *mut $ty;
                    unsafe { std::ptr::write_unaligned(mut_ptr, val.to_le()) }
                }
            }

            impl<'r> Read<'r> for $ty {
                type Output = $ty;

                #[inline]
                fn read<'a>(arena: &'a [u8]) -> $ty where 'a: 'r {
                    #[allow(clippy::cast_ptr_alignment)]
                    let p = (&arena[..mem::size_of::<$ty>()]).as_ptr() as *const $ty;
                    $ty::from_le(unsafe { std::ptr::read_unaligned(p) })
                }
            }
        );
    };
}

impl_rw!(IMPRINT_U8, u8,
    impl Write for u8 {
        #[inline]
        fn write(arena: &mut [u8], val: u8) {
            arena[0] = val
        }
    }

    impl<'r> Read<'r> for u8 {
        type Output = u8;

        #[inline]
        fn read<'a>(arena: &'a [u8]) -> u8 where 'a: 'r {
            arena[0]
        }
    }
);

transmutable_without_endianness_transform!(IMPRINT_BOOL, bool);
transmutable_without_endianness_transform!(IMPRINT_I8, i8);

transmutable!(IMPRINT_I16, i16);
transmutable!(IMPRINT_U16, u16);
transmutable!(IMPRINT_I32, i32);
transmutable!(IMPRINT_U32, u32);
transmutable!(IMPRINT_I64, i64);
transmutable!(IMPRINT_U64, u64);
#[cfg(feature = "i128")]
transmutable!(IMPRINT_I128, i128);
#[cfg(feature = "u128")]
transmutable!(IMPRINT_U128, u128);

impl_rw!(IMPRINT_CHAR, char,
    impl Write for char {
        #[inline]
        fn write(arena: &mut [u8], val: char) {
            u32::write(arena, val as u32)
        }
    }

    impl<'r> Read<'r> for char {
        type Output = Option<char>;

        #[inline]
        fn read<'a>(arena: &'a [u8]) -> Option<char> where 'a: 'r {
            ::std::char::from_u32(u32::read(arena))
        }
    }
);

impl_rw!(IMPRINT_F32, f32,
    impl Write for f32 {
        #[inline]
        fn write(arena: &mut [u8], val: f32) {
            u32::write(arena, unsafe { *(&val as *const f32 as *const u32) })
        }
    }

    impl<'r> Read<'r> for f32 {
        type Output = f32;

        #[inline]
        fn read<'a>(arena: &'a [u8]) -> f32 where 'a: 'r {
            unsafe { *(&u32::read(arena) as *const u32 as *const f32) }
        }
    }
);

impl_rw!(IMPRINT_F64, f64,
    impl Write for f64 {
        #[inline]
        fn write(arena: &mut [u8], val: f64) {
            u64::write(arena, unsafe { *(&val as *const f64 as *const u64) })
        }
    }

    impl<'r> Read<'r> for f64 {
        type Output = f64;

        #[inline]
        fn read<'a>(arena: &'a [u8]) -> f64 where 'a: 'r {
            unsafe { *(&u64::read(arena) as *const u64 as *const f64) }
        }
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rw_bool() {
        let arena = &mut [0; 1];

        bool::write(arena, true);
        assert!(bool::read(arena));

        bool::write(arena, false);
        assert!(!bool::read(arena));
    }

    #[test]
    fn rw_char() {
        let arena = &mut [0; 4];

        char::write(arena, '💯');
        assert!(Some('💯') == char::read(arena));
    }

    #[test]
    fn rw_u8() {
        let arena = &mut [0; 1];

        u8::write(arena, 246);
        assert!(246 == u8::read(arena));
    }

    #[test]
    fn rw_u32() {
        let arena = &mut [0; 10];

        u32::write(arena, 3_825_345);
        assert!(3_825_345 == u32::read(arena));
    }

    #[test]
    fn rw_u64() {
        let arena = &mut [0; 8];

        u64::write(arena, 246);
        assert!(246 == u64::read(arena));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn rw_f32() {
        let arena = &mut [0; 10];

        f32::write(arena, ::std::f32::NAN);
        assert!(f32::read(arena).is_nan());

        f32::write(arena, ::std::f32::INFINITY);
        assert!(::std::f32::INFINITY == f32::read(arena));

        f32::write(arena, ::std::f32::NEG_INFINITY);
        assert!(::std::f32::NEG_INFINITY == f32::read(arena));

        f32::write(arena, 984_524.2);
        assert!(984_524.2_f32 == f32::read(arena));
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn rw_f64() {
        let arena = &mut [0; 8];

        f64::write(arena, ::std::f64::NAN);
        assert!(f64::read(arena).is_nan());

        f64::write(arena, ::std::f64::INFINITY);
        assert!(::std::f64::INFINITY == f64::read(arena));

        f64::write(arena, ::std::f64::NEG_INFINITY);
        assert!(::std::f64::NEG_INFINITY == f64::read(arena));

        f64::write(arena, 98_452_345.238_494_5);
        assert!(98_452_345.238_494_5 == f64::read(arena));
    }
}
