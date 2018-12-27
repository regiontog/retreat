#[macro_export]
macro_rules! matches {
    ($e:expr, $($p:pat)|*) => {
        match $e {
            $($p)|+ => true,
            _ => false
        }
    };
}

pub mod prelude {
    use crate::utils::ResultIter;

    pub trait ResultPrelude<T, E> {
        fn flip_inner_iter(self) -> ResultIter<T, E>
        where
            T: Iterator;
    }

    impl<T, E> ResultPrelude<T, E> for Result<T, E> {
        fn flip_inner_iter(self) -> ResultIter<T, E>
        where
            T: Iterator,
        {
            ResultIter::new(self)
        }
    }
}

pub mod utils {
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
                .unwrap_or(self.err.take().map(|e| Err(e)))
        }
    }

    pub struct ReadOnly<T> {
        // Accessing this inner value is unsafe!
        inner: T,
    }

    impl<T> ReadOnly<T> {
        pub fn new(value: T) -> Self {
            ReadOnly { inner: value }
        }
    }

    impl<T> std::ops::Deref for ReadOnly<T> {
        type Target = T;

        fn deref(&self) -> &T {
            &self.inner
        }
    }
}
