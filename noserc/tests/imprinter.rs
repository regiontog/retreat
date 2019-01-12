use noser::traits::DefaultWriter;
use noser::Literal;
use noserc::WriteTypeInfo;

#[allow(dead_code)]
// #[derive(WriteTypeInfo)]
struct Unnamed<'a>(Literal<'a, u8>);

pub(crate) struct ImprintUnnamed;
impl <'a> ::noser::traits::WriteTypeInfo<Unnamed<'a>> for ImprintUnnamed {
    #[inline]
    fn imprint(&self, arena: &mut [u8]) -> ::noser::Result<()> {
        use noser::prelude::SliceExt;
        let imprinter: &::noser::traits::WriteTypeInfo<Literal<'a, u8>> =
            <Literal<'a, u8> as ::noser::traits::DefaultWriter>::writer();
        let (left, arena) = arena.noser_split(imprinter.result_size())?;
        imprinter.imprint(left)?;
        Ok(())
    }
    #[inline]
    fn result_size(&self) -> ::noser::Ptr {
        let mut size = 0;
        let imprinter = Literal::<'a, u8>::writer();

        size += imprinter.result_size();
        size
    }
}
pub(crate) static IMPRINT_UNNAMED: ImprintUnnamed = ImprintUnnamed{};
impl <'a> ::noser::traits::DefaultWriter for Unnamed<'a> {
    #[inline]
    fn writer() -> &'static ::noser::traits::WriteTypeInfo<Self> { &IMPRINT_UNNAMED }
}

// #[allow(dead_code)]
// #[derive(WriteTypeInfo)]
// struct Generic<'a, T> {
//     x: Literal<'a, u8>,
//     y: T,
// }

#[test]
fn eh() {
    use freyr::prelude::*;
    use noser::traits::{Build, WriteTypeInfo};
    use noser::writer::list::ListWriter;
    use noser::List;

    let writer: &dyn WriteTypeInfo<Literal<u64>> = Literal::<u64>::writer();
    let writer = ListWriter::new(std::iter::repeat(writer).take_exactly(50));
    // let arena: noser::Result<Vec<u8>> = writer.create_buffer();
    let arena = writer.create_buffer();
    let mut arena = arena.unwrap();

    let owned = List::<Literal<u64>>::create(&mut arena).unwrap();
    owned.borrow(49);
}
