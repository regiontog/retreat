#![feature(test)]
extern crate boxfnonce;
extern crate test;

mod implementation;
mod traits;

pub use implementation::*;

#[cfg(test)]
mod tests {
    use test::Bencher;

    use traits::{Build, WithArena, Write};
    use {List, ListFactory, Literal};

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

    #[bench]
    fn bench_nested_list(b: &mut Bencher) {
        let ref mut arena = [0; 28];

        b.iter(|| {
            {
                let desc = ListFactory::with(vec![
                    ListFactory::<Literal<i8>>::with_capacity(2),
                    ListFactory::with_capacity(2),
                ]);

                let mut owned = desc.with_arena(arena);

                owned[0][0].write(-10);
                owned[0][1].write(-11);
                owned[1][0].write(-12);
                owned[1][1].write(13);
            }

            {
                let (_, owned): (_, List<List<Literal<i8>>>) = List::build(arena);
                assert_eq!(owned[0][0].read(), -10);
                assert_eq!(owned[0][1].read(), -11);
                assert_eq!(owned[1][0].read(), -12);
                assert_eq!(owned[1][1].read(), 13);
            }
        });
    }
}
