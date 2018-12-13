// #![feature(test, never_type, cell_update)]

// We cannot have a method &mut self -> T on List as &mut Self is invariant on Self.
// As such T's lifetime cannot be narrowed. We also cannot have a method
// &self -> &mut [u8] -> T on List as rust cannot borrow &self and &mut self.buffer
// simultaneously even though they are disjoint. Therefore we need a field builder,
// we can then use a macro that disjointly borrows &self.builder and &mut self.buffer.
// Use mem::transmute to shorten invariant lifetime?
#[macro_export]
macro_rules! get {
    ($this:expr, $idx:expr, $cb:expr) => {{
        // To get lint hints about self's mutability
        let ref mut this = $this;

        // To avoid manually typing the callback's input type
        let cb = this.coerce($cb);

        cb(this.inner.get(this.arena, $idx));
    }};
}

pub type Ptr = u32;
pub const PTR_SIZE: Ptr = ::std::mem::size_of::<Ptr>() as Ptr;

mod implementation;
pub mod traits;

pub use crate::implementation::*;

#[derive(Debug)]
pub enum NoserError {
    Undersized(usize, Vec<u8>),
    IntegerOverflow,
}

// fn nth<T: traits::StaticSize>(idx: usize) -> ::std::ops::Range<usize> {
//     let start = idx * T::size() as usize;
//     (start..(start + T::size() as usize))
// }

pub type Result<T> = ::std::result::Result<T, NoserError>;

mod ext {
    pub trait SliceExt {
        fn noser_split(&mut self, at: crate::Ptr) -> crate::Result<(&mut Self, &mut Self)>;
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
    }
}
