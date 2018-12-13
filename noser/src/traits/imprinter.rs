pub trait Imprinter {
    type OnSuccess;

    fn imprint(&self, arena: &mut [u8]) -> ::Result<Self::OnSuccess>;

    fn result_size(&self) -> ::Ptr;

    fn create_buffer(&self) -> ::Result<Vec<u8>> {
        let mut buffer = vec![0; self.result_size() as usize];

        self.imprint(&mut buffer)?;
        Ok(buffer)
    }
}
