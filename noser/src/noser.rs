#![feature(test, never_type, cell_update)]
extern crate boxfnonce;

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

mod implementation;
pub mod traits;

pub use implementation::*;

#[derive(Debug)]
pub enum NoserError {
    Undersized(usize, Vec<u8>),
}

fn nth<T: traits::StaticSize>(idx: usize) -> ::std::ops::Range<usize> {
    let start = idx * T::size() as usize;
    (start..(start + T::size() as usize))
}

pub type Result<T> = ::std::result::Result<T, NoserError>;

mod ext {
    pub trait SliceExt {
        fn noser_split(&mut self, at: ::Ptr) -> ::Result<(&mut Self, &mut Self)>;
    }

    impl SliceExt for [u8] {
        #[inline]
        fn noser_split(&mut self, at: ::Ptr) -> ::Result<(&mut [u8], &mut [u8])> {
            let at = at as usize;

            if self.len() < at {
                return Err(::NoserError::Undersized(at, self.to_vec()));
            }

            Ok(self.split_at_mut(at))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use self::test::Bencher;

    use traits::{Build, DynamicSize, Imprinter, Write};
    use {List, Literal};

    #[bench]
    fn bench_u8(b: &mut Bencher) {
        let ref mut arena = [0; 10];

        b.iter(|| u8::write(arena, 224));
    }

    #[bench]
    fn bench_u32(b: &mut Bencher) {
        let ref mut arena = [0; 10];

        b.iter(|| u32::write(arena, 982413412));
    }

    #[bench]
    fn bench_u64(b: &mut Bencher) {
        let ref mut arena = [0; 10];

        b.iter(|| u64::write(arena, 23459982413412));
    }

    #[test]
    fn fuzzer_crash() {
        let ref mut arena = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        if let Ok(list) = List::<List<Literal<char>>>::create(arena) {
            list.borrow(0, |_| {});
        }
    }

    #[test]
    fn fuzzer_crash2() {
        let ref mut arena = [0x01, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00];
        if let Ok(list) = List::<List<Literal<char>>>::create(arena) {
            list.borrow(0, |_| {});
        }
    }

    #[bench]
    fn bench_list(b: &mut Bencher) {
        let mut arena = List::<Literal<u8>>::with_capacity(10)
            .create_buffer(|kind, buffer| kind.imprint_disregard_result(buffer))
            .unwrap();

        let mut owned: List<Literal<u8>> = List::create(&mut arena).unwrap();
        b.iter(|| {
            {
                get!(owned, 0, |mut item| {
                    item.write(10);
                });

                get!(owned, 9, |mut item| {
                    item.write(11);
                });
            }

            {
                owned.borrow(0, |item| {
                    assert_eq!(item.read(), 10);
                });

                owned.borrow(9, |item| {
                    assert_eq!(item.read(), 11);
                });
            }
        });
    }

    #[bench]
    fn bench_nested_list(b: &mut Bencher) {
        let mut arena = List::from(&[
            List::<Literal<u8>>::with_capacity(2),
            List::<Literal<u8>>::with_capacity(2),
        ]).create_buffer(|kind, buffer| kind.imprint(buffer))
            .unwrap();

        b.iter(|| {
            {
                let mut owned: List<List<Literal<u8>>> = List::create(&mut arena).unwrap();

                get!(owned, 0, |mut sublist| {
                    get!(sublist, 0, |mut item| {
                        item.write(10);
                    });

                    get!(sublist, 1, |mut item| {
                        item.write(11);
                    });
                });

                get!(owned, 1, |mut sublist| {
                    get!(sublist, 0, |mut item| {
                        item.write(12);
                    });

                    get!(sublist, 1, |mut item| {
                        item.write(13);
                    });
                });
            }

            {
                let owned: List<List<Literal<u8>>> = List::create(&mut arena).unwrap();

                owned.borrow(0, |sublist| {
                    sublist.borrow(0, |item| {
                        assert_eq!(item.read(), 10);
                    });

                    sublist.borrow(1, |item| {
                        assert_eq!(item.read(), 11);
                    });
                });

                owned.borrow(1, |sublist| {
                    sublist.borrow(0, |item| {
                        assert_eq!(item.read(), 12);
                    });

                    sublist.borrow(1, |item| {
                        assert_eq!(item.read(), 13);
                    });
                });
            }
        });
    }

    #[test]
    fn undersized_arena() {
        let mut arena = List::from(&[
            List::<Literal<u8>>::with_capacity(2),
            List::<Literal<u8>>::with_capacity(2),
        ]).create_buffer(|kind, buffer| kind.imprint(buffer))
            .unwrap();

        let undersized = &mut arena[..23];

        let mut results = vec![];
        results.push(
            List::from(&[
                List::<Literal<u8>>::with_capacity(2),
                List::<Literal<u8>>::with_capacity(2),
            ]).imprint(undersized),
        );

        results.push(List::<List<Literal<u8>>>::create(undersized).map(|_| ()));

        eprintln!("{:?}", results);
        assert!(results.into_iter().all(|r| r.is_err()));
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_list() {
        let mut arena = List::from(&[
            List::<Literal<u8>>::with_capacity(2),
            List::<Literal<u8>>::with_capacity(2),
        ]).create_buffer(|kind, buffer| kind.imprint(buffer))
            .unwrap();

        let owned = List::<List<Literal<u8>>>::create(&mut arena).unwrap();
        owned.borrow(2, |_| {});
    }

    #[test]
    fn in_bounds_list() {
        let mut arena = List::from(&[
            List::<Literal<u8>>::with_capacity(2),
            List::<Literal<u8>>::with_capacity(2),
        ]).create_buffer(|kind, buffer| kind.imprint(buffer))
            .unwrap();

        let owned = List::<List<Literal<u8>>>::create(&mut arena).unwrap();
        owned.borrow(1, |_| {});
    }

    #[test]
    #[should_panic]
    fn out_of_bounds_list2() {
        let mut arena = List::<Literal<u8>>::with_capacity(50)
            .create_buffer(|kind, buffer| kind.imprint_disregard_result(buffer))
            .unwrap();

        let owned = List::<Literal<u8>>::create(&mut arena).unwrap();
        owned.borrow(50, |_| {});
    }

    #[test]
    fn in_bounds_list2() {
        let mut arena = List::<Literal<u8>>::with_capacity(50)
            .create_buffer(|kind, buffer| kind.imprint_disregard_result(buffer))
            .unwrap();

        let owned = List::<Literal<u8>>::create(&mut arena).unwrap();
        owned.borrow(49, |_| {});
    }
}
