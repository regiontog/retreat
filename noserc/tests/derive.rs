use noser::{List, Literal, crate_local_type_writer};
use noserc::{Build, WriteTypeInfo, SizableDynamic, SizableStatic};

crate_local_type_writer!();

#[allow(dead_code)]
#[derive(SizableDynamic, Build)]
struct Named<'a> {
    field_x: Literal<'a, u8>,
    field_y: List<'a, Literal<'a, u8>>,
}

#[allow(dead_code)]
#[derive(SizableStatic, Build, WriteTypeInfo)]
struct Unnamed<'a>(
    Literal<'a, u8>,
);

#[allow(dead_code)]
#[derive(Build, WriteTypeInfo)]
struct Unnamed2<'a>(
    Unnamed<'a>,
);

#[allow(dead_code)]
#[derive(Build)]
struct Unit;

#[allow(dead_code)]
#[derive(Build)]
enum Void {}

#[allow(dead_code)]
#[derive(Build)]
enum SingleVariantUnit {
    Val
}

#[allow(dead_code)]
#[derive(SizableDynamic, Build, WriteTypeInfo)]
enum SingleVariantNamed<'a> {
    Val {
        x: Literal<'a, u32>,
    },
    OtherVar,
}

//This should not compile
// #[derive(Build)]
// union TestUnion {
//     f: f32,
// }

#[test]
fn compiles() {
    use noser::traits::{WriteTypeInfo};

    IMPRINT_UNNAMED.create_buffer().unwrap();
}

#[test]
fn read_write_to_subfields() {
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = IMPRINT_UNNAMED.create_buffer().unwrap();
    let mut owned: Unnamed = Unnamed::create(&mut arena).unwrap();

    assert_eq!(owned.0.read(), 0);
    owned.0.write(123);
    assert_eq!(owned.0.read(), 123);
}

#[test]
fn enum_correct_variant_0() {
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = ImprintSingleVariantNamed::Val.create_buffer().unwrap();
    let owned: SingleVariantNamed = SingleVariantNamed::create(&mut arena).unwrap();

    assert!(freyr::matches!(owned, SingleVariantNamed::Val {..}));
}

#[test]
fn enum_correct_variant_1() {
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = ImprintSingleVariantNamed::OtherVar.create_buffer().unwrap();
    let owned: SingleVariantNamed = SingleVariantNamed::create(&mut arena).unwrap();

    assert!(freyr::matches!(owned, SingleVariantNamed::OtherVar));
}

/*
 * List static struct
 */

#[test]
fn list_static_struct() {
    use noser::{List, get};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = <List<Unnamed>>::with_capacity(10).create_buffer().unwrap();
    let owned = <List<Unnamed>>::create(&mut arena).unwrap();

    let mut item = get! { owned[0] };
    item.0.write(10);

    let mut item2 = get! { owned[9] };
    item2.0.write(11);

    assert_eq!(owned.borrow(0).0.read(), 10);
    assert_eq!(owned.borrow(9).0.read(), 11);
}

#[test]
fn nested_list_static_struct() {
    use noser::{List, get};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::<Unnamed>::with_capacity(2),
        &List::<Unnamed>::with_capacity(2),
    ])
    .create_buffer()
    .unwrap();

    println!("{:?}", arena);

    let owned: List<List<Unnamed>> = List::create(&mut arena).unwrap();

    let mut item = get! { owned[0][0] };
    item.0.write(10);

    let mut item = get! { owned[1][0] };
    item.0.write(12);

    assert_eq!(owned.borrow(0).borrow(0).0.read(), 10);
    assert_eq!(owned.borrow(1).borrow(0).0.read(), 12);
}

#[test]
fn undersized_arena_static_structa() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::<Unnamed>::with_capacity(5),
        &List::<Unnamed>::with_capacity(5),
        &List::<Unnamed>::with_capacity(5),
    ])
    .create_buffer()
    .unwrap();

    let undersized = &mut arena[..23];

    let mut results = vec![];
    results.push(
        List::from(&[
            &List::<Unnamed>::with_capacity(5),
            &List::<Unnamed>::with_capacity(5),
            &List::<Unnamed>::with_capacity(5),
        ])
        .imprint(undersized),
    );

    println!("{:?}", undersized);

    results.push(<List<List<Unnamed>>>::create(undersized).map(|_| ()));

    println!("{:?}", results);
    assert!(results.into_iter().all(|r| r.is_err()));
}

#[test]
#[should_panic]
fn out_of_bounds_list_static_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::<Unnamed>::with_capacity(2),
        &List::<Unnamed>::with_capacity(2),
    ])
    .create_buffer()
    .unwrap();

    let owned = List::<List<Unnamed>>::create(&mut arena).unwrap();
    owned.borrow(2);
}

#[test]
fn in_bounds_list_static_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::<Unnamed>::with_capacity(2),
        &List::<Unnamed>::with_capacity(2),
    ])
    .create_buffer()
    .unwrap();

    let owned = List::<List<Unnamed>>::create(&mut arena).unwrap();
    owned.borrow(1);
}

#[test]
#[should_panic]
fn out_of_bounds_list2_static_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::<Unnamed>::with_capacity(50)
        .create_buffer()
        .unwrap();

    let owned = List::<Unnamed>::create(&mut arena).unwrap();
    owned.borrow(50);
}

#[test]
fn in_bounds_list2_static_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::<Unnamed>::with_capacity(50)
        .create_buffer()
        .unwrap();

    let owned = List::<Unnamed>::create(&mut arena).unwrap();
    owned.borrow(49);
}

/*
 * List dynamic struct
 */

// TODO: macro for these kinds of impls?
struct NamedImprinter;
static NAMED_IMPRINTER: NamedImprinter = NamedImprinter {};

impl<'a> ::noser::traits::WriteTypeInfo<Named<'a>> for NamedImprinter {
    fn imprint(&self, arena: &mut [u8]) -> ::noser::Result<()> {
        let imprinter = <WriteTypeInfo<Literal<u8>>>::default();
        let size = imprinter.result_size();
        imprinter.imprint(arena)?;

        let imprinter = <List<Literal<u8>>>::with_capacity(3);
        imprinter.imprint(&mut arena[size as usize..])?;
        Ok(())
    }

    fn result_size(&self) -> ::noser::Ptr {
        let mut size = 0;
        size += <WriteTypeInfo<Literal<u8>>>::default().result_size();
        size += <List<Literal<u8>>>::with_capacity(3).result_size();
        size
    }
}

#[test]
fn list_dynamic_struct() {
    use noser::{List, get};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&std::iter::repeat(&NAMED_IMPRINTER).take(10).collect::<Vec<_>>()).create_buffer().unwrap();
    let owned = <List<Named>>::create(&mut arena).unwrap();

    let mut item = get! { owned[0] };
    item.field_x.write(10);

    let mut item2 = get! { owned[9] };
    item2.field_x.write(11);

    assert_eq!(owned.borrow(0).field_x.read(), 10);
    assert_eq!(owned.borrow(9).field_x.read(), 11);
}

#[test]
fn nested_list_dynamic_struct() {
    use noser::{List, get};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
    ])
    .create_buffer()
    .unwrap();

    println!("{:?}", arena);

    let owned: List<List<Named>> = List::create(&mut arena).unwrap();

    let mut item = get! { owned[0][0] };
    item.field_x.write(10);

    let mut item = get! { owned[1][0] };
    item.field_x.write(12);

    assert_eq!(owned.borrow(0).borrow(0).field_x.read(), 10);
    assert_eq!(owned.borrow(1).borrow(0).field_x.read(), 12);
}

#[test]
fn undersized_arena_dynamic_structa() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
    ])
    .create_buffer()
    .unwrap();

    let undersized = &mut arena[..23];

    let mut results = vec![];
    results.push(
        List::from(&[
            &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
            &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
        ])
        .imprint(undersized),
    );

    println!("{:?}", undersized);

    results.push(<List<List<Named>>>::create(undersized).map(|_| ()));

    println!("{:?}", results);
    assert!(results.into_iter().all(|r| r.is_err()));
}

#[test]
#[should_panic]
fn out_of_bounds_list_dynamic_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
    ])
    .create_buffer()
    .unwrap();

    let owned = List::<List<Named>>::create(&mut arena).unwrap();
    owned.borrow(2);
}

#[test]
fn in_bounds_list_dynamic_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
        &List::from(&[&NAMED_IMPRINTER, &NAMED_IMPRINTER]),
    ])
    .create_buffer()
    .unwrap();

    let owned = List::<List<Named>>::create(&mut arena).unwrap();
    owned.borrow(1);
}

#[test]
#[should_panic]
fn out_of_bounds_list2_dynamic_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&std::iter::repeat(&NAMED_IMPRINTER).take(50).collect::<Vec<_>>()).create_buffer().unwrap();

    let owned = List::<Named>::create(&mut arena).unwrap();
    owned.borrow(50);
}

#[test]
fn in_bounds_list2_dynamic_struct() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&std::iter::repeat(&NAMED_IMPRINTER).take(50).collect::<Vec<_>>()).create_buffer().unwrap();

    let owned = List::<Named>::create(&mut arena).unwrap();
    owned.borrow(49);
}

/*
 * List dynamic enum
 */

#[test]
fn list_dynamic_enum() {
    use noser::{List, get};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&std::iter::repeat(&ImprintSingleVariantNamed::Val).take(10).collect::<Vec<_>>()).create_buffer().unwrap();
    let owned = <List<SingleVariantNamed>>::create(&mut arena).unwrap();

    match get! { owned[0] } {
        SingleVariantNamed::Val { mut x } => {
            x.write(10);
        },
        _ => unreachable!()
    }

    match get! { owned[9] } {
        SingleVariantNamed::Val { mut x } => {
            x.write(11);
        },
        _ => unreachable!()
    }

    assert_eq!(match *owned.borrow(0) {
        SingleVariantNamed::Val { ref x } => {
            x.read()
        },
        _ => unreachable!()
    }, 10);

    assert_eq!(match *owned.borrow(9) {
        SingleVariantNamed::Val { ref x } => {
            x.read()
        },
        _ => unreachable!()
    }, 11);
}

#[test]
fn nested_list_dynamic_enum() {
    use noser::{List, get};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
    ])
    .create_buffer()
    .unwrap();

    println!("{:?}", arena);

    let owned: List<List<SingleVariantNamed>> = List::create(&mut arena).unwrap();

    match get! { owned[0][0] } {
        SingleVariantNamed::Val { mut x } => {
            x.write(10);
        },
        _ => unreachable!()
    }

    match get! { owned[1][0] } {
        SingleVariantNamed::Val { mut x } => {
            x.write(12);
        },
        _ => unreachable!()
    }

    assert_eq!(match *owned.borrow(0).borrow(0) {
        SingleVariantNamed::Val { ref x } => {
            x.read()
        },
        _ => unreachable!()
    }, 10);

    assert_eq!(match *owned.borrow(1).borrow(0) {
        SingleVariantNamed::Val { ref x } => {
            x.read()
        },
        _ => unreachable!()
    }, 12);
}

#[test]
fn undersized_arena_dynamic_enuma() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
    ])
    .create_buffer()
    .unwrap();

    let undersized = &mut arena[..23];

    let mut results = vec![];
    results.push(
        List::from(&[
            &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
            &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
            &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
        ])
        .imprint(undersized),
    );

    println!("{:?}", undersized);

    results.push(<List<List<SingleVariantNamed>>>::create(undersized).map(|_| ()));

    println!("{:?}", results);
    assert!(results.into_iter().all(|r| r.is_err()));
}

#[test]
#[should_panic]
fn out_of_bounds_list_dynamic_enum() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
    ])
    .create_buffer()
    .unwrap();

    let owned = List::<List<SingleVariantNamed>>::create(&mut arena).unwrap();
    owned.borrow(2);
}

#[test]
fn in_bounds_list_dynamic_enum() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&[
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
        &List::from(&[&ImprintSingleVariantNamed::Val, &ImprintSingleVariantNamed::Val]),
    ])
    .create_buffer()
    .unwrap();

    let owned = List::<List<SingleVariantNamed>>::create(&mut arena).unwrap();
    owned.borrow(1);
}

#[test]
#[should_panic]
fn out_of_bounds_list2_dynamic_enum() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&std::iter::repeat(&ImprintSingleVariantNamed::Val).take(50).collect::<Vec<_>>()).create_buffer().unwrap();

    let owned = List::<SingleVariantNamed>::create(&mut arena).unwrap();
    owned.borrow(50);
}

#[test]
fn in_bounds_list2_dynamic_enum() {
    use noser::{List};
    use noser::traits::{WriteTypeInfo, Build};

    let mut arena = List::from(&std::iter::repeat(&ImprintSingleVariantNamed::Val).take(50).collect::<Vec<_>>()).create_buffer().unwrap();

    let owned = List::<SingleVariantNamed>::create(&mut arena).unwrap();
    owned.borrow(49);
}