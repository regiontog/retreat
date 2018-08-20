use std::mem;

use traits::{
    find::{Find, StaticFind}, Read, StaticSize, Write,
};

impl Write for u8 {
    #[inline]
    fn write(arena: &mut [u8], val: u8) {
        arena[0] = val
    }
}

impl Read for u8 {
    #[inline]
    fn read(arena: &[u8]) -> u8 {
        arena[0]
    }
}

impl StaticSize for u8 {
    #[inline]
    fn size() -> ::Ptr {
        mem::size_of::<u8>() as ::Ptr
    }
}

impl Find for u8 {
    type Strategy = StaticFind<u8>;
}

macro_rules! transmutable {
    (endianless: $Int:ident) => {
        impl Write for $Int {
            #[inline]
            fn write(arena: &mut [u8], val: $Int) {
                unsafe { mem::transmute::<&mut [u8], &mut [$Int]>(arena)[0] = val }
            }
        }

        impl Read for $Int {
            #[inline]
            fn read(arena: &[u8]) -> $Int {
                unsafe { mem::transmute::<&[u8], &[$Int]>(arena)[0] }
            }
        }

        impl StaticSize for $Int {
            #[inline]
            fn size() -> ::Ptr {
                mem::size_of::<$Int>() as ::Ptr
            }
        }

        impl Find for $Int {
            type Strategy = StaticFind<u8>;
        }
    };

    (endianaware: $Int:ident) => {
        impl Write for $Int {
            #[inline]
            fn write(arena: &mut [u8], val: $Int) {
                unsafe { mem::transmute::<&mut [u8], &mut [$Int]>(arena)[0] = val.to_le() }
            }
        }

        impl Read for $Int {
            #[inline]
            fn read(arena: &[u8]) -> $Int {
                unsafe { $Int::from_le(mem::transmute::<&[u8], &[$Int]>(arena)[0]) }
            }
        }

        impl StaticSize for $Int {
            #[inline]
            fn size() -> ::Ptr {
                mem::size_of::<$Int>() as ::Ptr
            }
        }

        impl Find for $Int {
            type Strategy = StaticFind<$Int>;
        }
    };
}

transmutable!{endianless:  bool}
transmutable!{endianless:  char}
transmutable!{endianless:  i8}
transmutable!{endianaware: i16}
transmutable!{endianaware: u16}
transmutable!{endianaware: i32}
transmutable!{endianaware: u32}
transmutable!{endianless:  f32}
transmutable!{endianaware: i64}
transmutable!{endianaware: u64}
transmutable!{endianless:  f64}
transmutable!{endianaware: i128}
transmutable!{endianaware: u128}
