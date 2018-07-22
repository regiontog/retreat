#![feature(test)]
extern crate boxfnonce;
extern crate test;

mod implementation;

pub mod traits;
pub use implementation::*;

pub mod union {
    pub use implementation::union::{read_var_len_int, write_var_len_int};
}

use traits::Build;

enum Enumm<'a> {
    None,
    Some(List<List<Literal<'a, u8>>>),
}

impl<'a> ::traits::Variants<'a> for Enumm<'a> {
    fn variants() -> u64 {
        2
    }

    fn ord(&self) -> u64 {
        match self {
            Enumm::None => 0,
            Enumm::Some(_) => 1,
        }
    }

    fn variant(var: u64, arena: &'a mut [u8]) -> (&'a mut [u8], Self) {
        match var {
            0 => (arena, Enumm::None),
            1 => {
                let (right, inner) = List::build(arena);
                (right, Enumm::Some(inner))
            }
            _ => unreachable!(),
        }
    }
}

impl<'a, T: 'a> From<T> for ::Union<'a, Enumm<'a>>
where
    T: ::traits::WithArena<'a, List<List<Literal<'a, u8>>>> + ::traits::DynamicSize,
{
    fn from(dynamic_type: T) -> Self {
        ::Union::new(dynamic_type.dsize(), |arena| {
            Enumm::Some(dynamic_type.with_arena(arena))
        })
    }
}

impl<'a> From<Enumm<'a>> for ::Union<'a, Enumm<'a>> {
    fn from(variant: Enumm<'a>) -> Self {
        match variant {
            Enumm::None => ::Union::new(0, |arena| variant),
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use test::Bencher;

    use traits::{Build, WithArena, Write};
    use *;
    use {List, ListFactory, Literal};

    #[bench]
    fn bench_record(b: &mut Bencher) {
        let ref mut arena = [0; 20];

        {
            let desc = Struuc::with_fields(10, List::with_capacity(5));
            let mut st = desc.with_arena(arena);

            st.field_y[0].write(1);
            st.field_y[1].write(2);
            st.field_y[2].write(3);
        }

        println!("{:?}", arena);
        panic!();

        {
            let (_, st) = Struuc::build(arena);
            assert_eq!(10, st.field_x.read());
            assert_eq!(1, st.field_y[0].read());
            assert_eq!(2, st.field_y[1].read());
            assert_eq!(3, st.field_y[2].read());
        }
    }

    #[bench]
    fn bench_union(b: &mut Bencher) {
        let ref mut arena = [0; 20];

        {
            let desc = Union::with_variant(List::with(vec![List::<Literal<u8>>::with_capacity(2)]));
            // let desc = Union::with_variant(Enumm::None);

            if let Enumm::Some(mut list) = desc.with_arena(arena) {
                list[0][0].write(7);
            }
        }

        println!("{:?}", arena);

        {
            let (_, lit) = Enumm::build(arena);
            assert_eq!(
                7,
                match lit {
                    Enumm::None => 2,
                    Enumm::Some(list) => list[0][0].read(),
                },
            );
        }
    }

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
                let desc = List::with(vec![
                    List::<Literal<i8>>::with_capacity(2),
                    List::with_capacity(2),
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
