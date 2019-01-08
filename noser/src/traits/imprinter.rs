pub trait WriteTypeInfo<T> {
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<()>;

    fn result_size(&self) -> crate::Ptr;

    #[inline]
    fn create_buffer(&self) -> crate::Result<Vec<u8>> {
        let mut buffer = vec![0; self.result_size() as usize];

        self.imprint(&mut buffer)?;
        Ok(buffer)
    }
}

pub trait DefaultWriter: Sized {
    type Writer: WriteTypeInfo<Self> + 'static;

    fn writer() -> &'static Self::Writer;

    #[inline]
    fn write_type(arena: &mut [u8]) -> crate::Result<()>
    where
        Self::Writer: 'static,
    {
        Self::writer().imprint(arena)
    }

    #[inline]
    fn buffer() -> crate::Result<Vec<u8>>
    where
        Self::Writer: 'static,
    {
        Self::writer().create_buffer()
    }
}
