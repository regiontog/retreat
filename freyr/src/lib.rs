mod alias_guard;
mod either;
mod lazy;
mod read_only;
mod repeat;
mod result;

pub use crate::alias_guard::*;
pub use crate::either::*;
pub use crate::lazy::*;
pub use crate::read_only::*;
pub use crate::repeat::*;
pub use crate::result::*;

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
    use crate::{ResultIter, TakeExactly};

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

    pub trait RepeatPrelude {
        type InnerIter;

        fn take_exactly(self, n: usize) -> TakeExactly<Self::InnerIter>;
    }

    impl<T> RepeatPrelude for std::iter::Repeat<T>
    where
        T: Clone,
    {
        type InnerIter = std::iter::Take<Self>;

        fn take_exactly(self, n: usize) -> TakeExactly<std::iter::Take<Self>> {
            TakeExactly::new(self.take(n), n)
        }
    }
}
