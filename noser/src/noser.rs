#![deny(clippy::all)]

pub type Ptr = u32;

mod implementation;
pub mod traits;
pub mod writer;

pub use crate::implementation::*;

#[derive(Debug)]
pub enum NoserError {
    Undersized(usize, Vec<u8>),
    IntegerOverflow,
    Malformed,
}

pub type Result<T> = ::std::result::Result<T, NoserError>;

pub mod prelude {
    pub trait SliceExt {
        fn noser_split(&mut self, at: crate::Ptr) -> crate::Result<(&mut Self, &mut Self)>;
        fn noser_split_imut(&self, at: crate::Ptr) -> crate::Result<(&Self, &Self)>;
    }

    impl SliceExt for [u8] {
        #[inline]
        fn noser_split(&mut self, at: crate::Ptr) -> crate::Result<(&mut [u8], &mut [u8])> {
            let at = at as usize;

            if self.len() < at {
                return Err(crate::NoserError::Undersized(at, self.to_vec()));
            }

            Ok(self.split_at_mut(at))
        }

        #[inline]
        fn noser_split_imut(&self, at: crate::Ptr) -> crate::Result<(&[u8], &[u8])> {
            let at = at as usize;

            if self.len() < at {
                return Err(crate::NoserError::Undersized(at, self.to_vec()));
            }

            Ok(self.split_at(at))
        }
    }
}
