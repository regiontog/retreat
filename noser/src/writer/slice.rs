use crate::prelude::*;
use crate::traits::{LiteralInnerType, Write, WriteTypeInfo};
use crate::SliceType;

pub struct SliceWriter {
    capacity: crate::Ptr,
}

impl SliceWriter {
    pub fn with_capacity<T>(capacity: crate::Ptr) -> impl WriteTypeInfo<T>
    where
        SliceWriter: WriteTypeInfo<T>,
    {
        SliceWriter { capacity }
    }
}

macro_rules! slice_write_type_info {
    ($type:ty) => {
        impl WriteTypeInfo<$type> for SliceWriter {
            #[inline]
            fn imprint(&self, arena: &mut [u8]) -> crate::Result<()> {
                let (len_bytes, rest) = arena.noser_split(crate::Ptr::SIZE as crate::Ptr)?;
                rest.noser_split(self.capacity * <$type>::ELEM_SIZE as crate::Ptr)?;

                crate::Ptr::write(len_bytes, self.capacity);
                Ok(())
            }

            #[inline]
            fn result_size(&self) -> crate::Ptr {
                crate::Ptr::SIZE as crate::Ptr + self.capacity * <$type>::ELEM_SIZE as crate::Ptr
            }
        }
    };
}

slice_write_type_info! { &[u8] }
slice_write_type_info! { &mut [u8] }
