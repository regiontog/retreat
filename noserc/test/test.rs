extern crate noser;

use noser::{List, Literal};

include!("test_out/generated_source.rs");

#[cfg(test)]
mod tests {
    use noser::traits::{Build, WithArena};
    use *;

    use traits::Build;

    enum Enumm<'a> {
        None,
        Some(List<'a, List<'a, Literal<'a, u8>>>),
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

        fn variant(var: u64, arena: &'a mut [u8]) -> Result<'a, (&'a mut [u8], Self)> {
            match var {
                0 => Ok((arena, Enumm::None)),
                1 => {
                    let (right, inner) = List::build(arena)?;
                    Ok((right, Enumm::Some(inner)))
                }
                _ => unreachable!(),
            }
        }
    }

    impl<'a, T: 'a> From<T> for ::Union<'a, Enumm<'a>>
    where
        T: ::traits::WithArena<'a, List<List<Literal<'a, u8>>>>,
    {
        fn from(dynamic_type: T) -> Self {
            ::Union::new(|arena| {
                let (right, inner) = dynamic_type.with_arena(arena)?;
                Ok((right, Enumm::Some(inner)))
            })
        }
    }

    impl<'a> From<Enumm<'a>> for ::Union<'a, Enumm<'a>> {
        fn from(variant: Enumm<'a>) -> Self {
            ::Union::new(|arena| Ok((arena, variant)))
        }
    }

    #[test]
    fn record() {
        let ref mut arena = [0; 20];

        {
            let desc = TestStruct::with_fields(10, noser::List::capacity(5));
            let (_, mut st) = desc.with_arena(arena).unwrap();

            st.field_y[0].write(1);
            st.field_y[1].write(2);
            st.field_y[2].write(3);
        }

        println!("{:?}", arena);

        {
            let (_, st) = TestStruct::build(arena).unwrap();
            assert_eq!(10, st.field_x.read());
            assert_eq!(1, st.field_y[0].read());
            assert_eq!(2, st.field_y[1].read());
            assert_eq!(3, st.field_y[2].read());
        }
    }

    #[test]
    fn union() {
        let ref mut arena = [0; 20];

        {
            Union::with_variant(List::from(vec![List::<Literal<u8>>::with_capacity(2)]))
                .imprint(arena)
                .unwrap();

            if let Enumm::Some(mut list) = Enumm::create(arena).unwrap() {
                list[0][0].write(7);
            }
        }

        println!("{:?}", arena);

        {
            let lit = Enumm::create(arena).unwrap();
            assert_eq!(
                7,
                match lit {
                    Enumm::None => 2,
                    Enumm::Some(list) => list[0][0].read(),
                },
            );
        }
    }
}
