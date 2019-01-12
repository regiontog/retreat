use crate::prelude::SliceExt;
use crate::traits::{size::Dynamic, size::Sizeable, Build, LiteralInnerType, Read};

pub(crate) trait SliceType {
    type ElemType;
    const ELEM_SIZE: usize = std::mem::size_of::<Self::ElemType>();
}

impl SliceType for &'_ [u8] {
    type ElemType = u8;
}

impl SliceType for &'_ mut [u8] {
    type ElemType = u8;
}

macro_rules! slice_sizable {
    ($type:ty) => {
        impl Sizeable for $type {
            type Strategy = Dynamic;

            #[inline]
            fn read_size(arena: &[u8]) -> crate::Result<crate::Ptr> {
                Ok(crate::Ptr::read_safe(arena)?
                    .checked_mul(<$type>::ELEM_SIZE as crate::Ptr)
                    .and_then(|r| r.checked_add(crate::Ptr::SIZE as crate::Ptr))
                    .ok_or(crate::NoserError::IntegerOverflow)?)
            }
        }
    };
}

macro_rules! build_slice {
    ($lt:lifetime, $slice_ty:ty) => {
        unsafe impl<$lt> Build<$lt> for $slice_ty {
            fn build<'a>(arena: &'a mut [u8]) -> crate::Result<(&'a mut [u8], Self)>
            where
                'a: 'b,
            {
                let (len_bytes, arena) = arena.noser_split(crate::Ptr::SIZE as crate::Ptr)?;
                let len = crate::Ptr::read(len_bytes);

                let (this, right) = arena.noser_split(len)?;
                Ok((right, this))
            }

            fn unchecked_build<'a>(arena: &'a mut [u8]) -> (&'a mut [u8], Self)
            where
                'a: 'b,
            {
                let len = crate::Ptr::read(arena);
                let (this, right) = arena.split_at_mut(len as usize);

                (right, &mut this[crate::Ptr::SIZE..])
            }
        }
    };
}

slice_sizable! { &[u8] }
slice_sizable! { &mut [u8] }
build_slice! { 'b, &'b [u8] }
build_slice! { 'b, &'b mut [u8] }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::WriteTypeInfo;
    use crate::writer::slice::*;

    #[test]
    fn rw_byte_slice() {
        let mut arena = SliceWriter::with_capacity::<&mut [u8]>(50)
            .create_buffer()
            .unwrap();

        {
            let bytes = <&mut [u8]>::create(&mut arena).unwrap();
            bytes.copy_from_slice(&[10; 50]);
        }

        let result = <&mut [u8]>::create(&mut arena).unwrap();
        assert!(result.iter().all(|byte| *byte == 10));
        assert!(result.len() == 50);
    }

    #[test]
    fn rw_str_slice() {
        let str = "こんにちは";
        let mut arena = SliceWriter::with_capacity::<&mut [u8]>(str.as_bytes().len() as crate::Ptr)
            .create_buffer()
            .unwrap();

        {
            let bytes = <&mut [u8]>::create(&mut arena).unwrap();
            bytes.copy_from_slice(str.as_bytes());
        }

        let result = <&mut [u8]>::create(&mut arena).unwrap();
        assert!(Ok("こんにちは") == std::str::from_utf8(result));
    }
}
