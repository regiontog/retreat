use noser::{writer::list::WithCapacity, Literal};
use noserc::WriteTypeInfo;

#[allow(dead_code)]
#[derive(WriteTypeInfo)]
struct Unnamed<'a>(Literal<'a, u8>);

#[allow(dead_code)]
#[derive(WriteTypeInfo)]
struct Generic<'a, T> {
    x: Literal<'a, u8>,
    y: T,
}

#[allow(dead_code)]
#[derive(WriteTypeInfo)]
enum Enum<'a> {
    A,
    B(Literal<'a, u8>),
}

#[allow(dead_code)]
#[derive(WriteTypeInfo)]
enum Option<T> {
    Some(T),
    None,
}

#[test]
fn eh() {
    use noser::traits::{Build, WriteTypeInfo};
    use noser::List;

    let writer = WithCapacity::<Literal<u64>>::with_capacity(50);
    // let arena: noser::Result<Vec<u8>> = writer.create_buffer();
    let arena = writer.create_buffer();
    let mut arena = arena.unwrap();

    let owned = List::<Literal<u64>>::create(&mut arena).unwrap();
    owned.borrow(49);
}
