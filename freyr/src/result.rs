pub struct ResultIter<I, E> {
    iter: Option<I>,
    err: Option<E>,
}

impl<I, E> ResultIter<I, E> {
    pub fn new(result: Result<I, E>) -> Self {
        result
            .map(|iter| ResultIter {
                iter: Some(iter),
                err: None,
            })
            .unwrap_or_else(|e| ResultIter {
                iter: None,
                err: Some(e),
            })
    }
}

impl<I, E> Iterator for ResultIter<I, E>
where
    I: Iterator,
{
    type Item = Result<I::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .as_mut()
            .map(|iter| iter.next().map(Ok))
            .unwrap_or_else(|| self.err.take().map(|e| Err(e)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter
            .as_ref()
            .map(|iter| iter.size_hint())
            .unwrap_or((0, Some(1)))
    }
}
