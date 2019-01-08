pub struct ReadOnly<T> {
    inner: T,
}

impl<T> ReadOnly<T> {
    pub fn new(value: T) -> Self {
        ReadOnly { inner: value }
    }

    pub unsafe fn from<'a, A>(from: &'a A, func: impl Fn(&'a mut A) -> T) -> Self {
        let unsafe_mut = &mut *(from as *const A as *mut A);

        Self::new(func(unsafe_mut))
    }
}

impl<T> std::ops::Deref for ReadOnly<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}
