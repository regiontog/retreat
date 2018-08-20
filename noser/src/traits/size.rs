pub trait StaticSize {
    fn size() -> ::Ptr;

    fn create_buffer<T, E>(
        self,
        cb: impl Fn(Self, &mut Vec<u8>) -> Result<T, E>,
    ) -> Result<Vec<u8>, E>
    where
        Self: Sized,
    {
        let mut buffer = vec![0; Self::size() as usize];

        cb(self, &mut buffer)?;
        Ok(buffer)
    }
}

pub trait DynamicSize {
    fn dsize(&self) -> ::Ptr;

    fn create_buffer<T, E>(
        self,
        cb: impl Fn(Self, &mut Vec<u8>) -> Result<T, E>,
    ) -> Result<Vec<u8>, E>
    where
        Self: Sized,
    {
        let mut buffer = vec![0; self.dsize() as usize];

        cb(self, &mut buffer)?;
        Ok(buffer)
    }
}
