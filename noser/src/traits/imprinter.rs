pub trait WriteTypeInfo<T> {
    fn imprint(&self, arena: &mut [u8]) -> crate::Result<()>;

    fn result_size(&self) -> crate::Ptr;

    fn create_buffer(&self) -> crate::Result<Vec<u8>> {
        let mut buffer = vec![0; self.result_size() as usize];

        self.imprint(&mut buffer)?;
        Ok(buffer)
    }
}
