#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct TakeExactly<I> {
    inner: I,
    took: usize,
}

impl<I> TakeExactly<I> {
    pub fn new(inner: I, took: usize) -> Self {
        TakeExactly { inner, took }
    }
}

impl<I> std::ops::Deref for TakeExactly<I> {
    type Target = I;

    fn deref(&self) -> &I {
        &self.inner
    }
}

impl<I> Iterator for TakeExactly<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.took, Some(self.took))
    }

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<I> ExactSizeIterator for TakeExactly<I> where I: Iterator {}
