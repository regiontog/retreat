use std::mem;

use traits::{
    find::{Find, StaticFind},
    Read, StaticSize, Write,
};

macro_rules! impl_rw {
    ($ty:ident, $($rw:tt)*) => {
        $($rw)*

        impl StaticSize for $ty {
            #[inline]
            fn size() -> ::Ptr {
                mem::size_of::<$ty>() as ::Ptr
            }
        }

        impl Find for $ty {
            type Strategy = StaticFind<$ty>;
        }
    };
}

macro_rules! transmutable_without_endianness_tansform {
    ($ty:ident) => {
        impl_rw!($ty,
            impl Write for $ty {
                #[inline]
                fn write(arena: &mut [u8], val: $ty) {
                    let mut_ptr = (&mut arena[..mem::size_of::<$ty>()]).as_mut_ptr() as *mut $ty;
                    unsafe { *mut_ptr = val }
                }
            }

            impl Read for $ty {
                type Output = $ty;

                #[inline]
                fn read(arena: &[u8]) -> $ty {
                    let p = (&arena[..mem::size_of::<$ty>()]).as_ptr() as *const $ty;
                    unsafe { *p }
                }
            }
        );
    };
}

macro_rules! transmutable {
    ($ty:ident) => {
        impl_rw!($ty,
            impl Write for $ty {
                #[inline]
                fn write(arena: &mut [u8], val: $ty) {
                    let mut_ptr = (&mut arena[..mem::size_of::<$ty>()]).as_mut_ptr() as *mut $ty;
                    unsafe { *mut_ptr = val.to_le() }
                }
            }

            impl Read for $ty {
                type Output = $ty;

                #[inline]
                fn read(arena: &[u8]) -> $ty {
                    let p = (&arena[..mem::size_of::<$ty>()]).as_ptr() as *const $ty;
                    $ty::from_le(unsafe { *p })
                }
            }
        );
    };
}

impl_rw!(u8,
    impl Write for u8 {
        #[inline]
        fn write(arena: &mut [u8], val: u8) {
            arena[0] = val
        }
    }

    impl Read for u8 {
        type Output = u8;

        #[inline]
        fn read(arena: &[u8]) -> u8 {
            arena[0]
        }
    }
);

transmutable_without_endianness_tansform!(bool);
transmutable_without_endianness_tansform!(i8);

transmutable!(i16);
transmutable!(u16);
transmutable!(i32);
transmutable!(u32);
transmutable!(i64);
transmutable!(u64);
#[cfg(feature = "i128")]
transmutable!{i128}
#[cfg(feature = "u128")]
transmutable!{u128}

impl_rw!(char,
    impl Write for char {
        #[inline]
        fn write(arena: &mut [u8], val: char) {
            u32::write(arena, val as u32)
        }
    }

    impl Read for char {
        type Output = Option<char>;

        #[inline]
        fn read(arena: &[u8]) -> Option<char> {
            ::std::char::from_u32(u32::read(arena))
        }
    }
);

impl_rw!(f32,
    impl Write for f32 {
        #[inline]
        fn write(arena: &mut [u8], val: f32) {
            u32::write(arena, unsafe { *(&val as *const f32 as *const u32) })
        }
    }

    impl Read for f32 {
        type Output = f32;

        #[inline]
        fn read(arena: &[u8]) -> f32 {
            unsafe { *(&u32::read(arena) as *const u32 as *const f32) }
        }
    }
);

impl_rw!(f64,
    impl Write for f64 {
        #[inline]
        fn write(arena: &mut [u8], val: f64) {
            u64::write(arena, unsafe { *(&val as *const f64 as *const u64) })
        }
    }

    impl Read for f64 {
        type Output = f64;

        #[inline]
        fn read(arena: &[u8]) -> f64 {
            unsafe { *(&u64::read(arena) as *const u64 as *const f64) }
        }
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rw_bool() {
        let ref mut arena = [0; 1];

        bool::write(arena, true);
        assert!(true == bool::read(arena));

        bool::write(arena, false);
        assert!(false == bool::read(arena));
    }

    #[test]
    fn rw_char() {
        let ref mut arena = [0; 4];

        char::write(arena, '💯');
        assert!(Some('💯') == char::read(arena));
    }

    #[test]
    fn rw_u8() {
        let ref mut arena = [0; 1];

        u8::write(arena, 246);
        assert!(246 == u8::read(arena));
    }

    #[test]
    fn rw_u32() {
        let ref mut arena = [0; 10];

        u32::write(arena, 3825345);
        assert!(3825345 == u32::read(arena));
    }

    #[test]
    fn rw_u64() {
        let ref mut arena = [0; 8];

        u64::write(arena, 246);
        assert!(246 == u64::read(arena));
    }

    #[test]
    fn rw_f32() {
        let ref mut arena = [0; 10];

        f32::write(arena, 3825345.);
        assert!(3825345. == f32::read(arena));
    }

    #[test]
    fn rw_f64() {
        let ref mut arena = [0; 8];

        f64::write(arena, 246.);
        assert!(246. == f64::read(arena));
    }
}
