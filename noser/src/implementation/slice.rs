use std::mem;

use traits::{size::Dynamic, Read, Sizable, Write};

macro_rules! impl_slice_rw {
    ($ty:ident, $($rw:tt)*) => {
        $($rw)*

        impl<'a> Sizable for &'a [$ty] {
            type Strategy = Dynamic;

            #[inline]
            fn read_size(arena: &[u8]) -> ::Result<::Ptr> {
                let len = ::Ptr::read_safe(arena)? as usize;
                Ok((len * mem::size_of::<$ty>()) as ::Ptr)
            }
        }
    };
}

impl_slice_rw!(u8,
    impl<'a> Write for &'a [u8] {
        /// Performs a copy of each element in the slice.
        #[inline]
        fn write(arena: &mut [u8], val: Self) {
            let len = val.len();
            ::Ptr::write(arena, len as ::Ptr);
            unsafe { ::std::ptr::copy_nonoverlapping(val.as_ptr(), arena[mem::size_of::<::Ptr>()..len].as_mut_ptr(), len) }
        }
    }

    impl<'a> Read<'a> for &'a [u8] {
        type Output = Self;

        #[inline]
        /// Unsafe if reported length is larger than bytes in arena.
        fn read(arena: &'a [u8]) -> Self {
            let len = ::Ptr::read(arena) as usize;
            &arena[mem::size_of::<::Ptr>()..len+mem::size_of::<::Ptr>()]
        }
    }
);

impl<'a> Write for &'a str {
    #[inline]
    fn write(arena: &mut [u8], val: Self) {
        <&[u8]>::write(arena, val.as_bytes())
    }
}

impl<'a> Read<'a> for &'a str {
    type Output = Result<Self, ::std::str::Utf8Error>;

    #[inline]
    fn read(arena: &'a [u8]) -> Self::Output {
        ::std::str::from_utf8(<&[u8]>::read(arena))
    }
}

impl<'a> Sizable for &'a str {
    type Strategy = Dynamic;

    #[inline]
    fn read_size(arena: &[u8]) -> ::Result<::Ptr> {
        <&[u8]>::read_size(arena)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rw_byte_slice() {
        let ref mut arena = [0; 256];

        <&[u8]>::write(arena, &[10; 50]);

        let result = <&[u8]>::read(arena);
        assert!(result.iter().all(|byte| *byte == 10));
        assert!(result.len() == 50);
    }

    #[test]
    fn rw_str_slice() {
        let ref mut arena = [0; 256];

        <&str>::write(arena, "こんにちは");
        println!("{:?}", <&str>::read(arena));
        assert!(Ok("こんにちは") == <&str>::read(arena));
    }
}
