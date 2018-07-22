extern crate noser;

use noser::{List, Literal};

include!(concat!(env!("OUT_DIR"), "/generated_source.rs"));

#[cfg(test)]
mod tests {
    use noser::traits::{Build, WithArena};
    use *;

    #[test]
    fn record() {
        let ref mut arena = [0; 20];

        {
            let desc = TestStruct::with_fields(10, noser::List::with_capacity(5));
            let mut st = desc.with_arena(arena);

            st.field_y[0].write(1);
            st.field_y[1].write(2);
            st.field_y[2].write(3);
        }

        println!("{:?}", arena);

        {
            let (_, st) = TestStruct::build(arena);
            assert_eq!(10, st.field_x.read());
            assert_eq!(1, st.field_y[0].read());
            assert_eq!(2, st.field_y[1].read());
            assert_eq!(3, st.field_y[2].read());
        }
    }
}
