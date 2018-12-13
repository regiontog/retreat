#![feature(test)]
extern crate test;

#[macro_use]
extern crate noser;

use test::Bencher;

use noser::traits::*;
use noser::{List, Literal};

#[bench]
fn write_u64(b: &mut Bencher) {
    let ref mut arena = [0; 10];

    b.iter(|| u64::write(arena, 23459982413412));
}

#[bench]
fn read_u64(b: &mut Bencher) {
    let ref mut arena = [0; 10];
    u64::write(arena, 23459982413412);

    b.iter(|| u64::read(arena));
}

#[bench]
fn list_write4(b: &mut Bencher) {
    let mut arena = List::<Literal<'_, u8>>::with_capacity(10)
        .create_buffer()
        .unwrap();

    let mut owned: List<'_, Literal<'_, u8>> = List::create(&mut arena).unwrap();
    b.iter(|| {
        get!(owned, 0, |mut item| {
            item.write(10);
        });

        get!(owned, 9, |mut item| {
            item.write(19);
        });

        get!(owned, 1, |mut item| {
            item.write(11);
        });

        get!(owned, 2, |mut item| {
            item.write(12);
        });
    });
}

#[bench]
fn list_read4(b: &mut Bencher) {
    let mut arena = List::<Literal<'_, u8>>::with_capacity(10)
        .create_buffer()
        .unwrap();

    let mut owned: List<'_, Literal<'_, u8>> = List::create(&mut arena).unwrap();

    b.iter(|| {
        get!(owned, 0, |item| {
            item.read();
        });

        get!(owned, 9, |item| {
            item.read();
        });

        get!(owned, 1, |item| {
            item.read();
        });

        get!(owned, 2, |item| {
            item.read();
        });
    });
}

#[bench]
fn nested_list_write_value_in_4_sublists(b: &mut Bencher) {
    let mut arena = List::from(&[
        List::<Literal<'_, u8>>::with_capacity(2),
        List::<Literal<'_, u8>>::with_capacity(2),
        List::<Literal<'_, u8>>::with_capacity(2),
        List::<Literal<'_, u8>>::with_capacity(2),
    ]).create_buffer()
    .unwrap();

    let mut owned: List<'_, List<'_, Literal<'_, u8>>> = List::create(&mut arena).unwrap();

    b.iter(|| {
        get!(owned, 0, |mut sublist| {
            get!(sublist, 0, |mut item| {
                item.write(10);
            });
        });

        get!(owned, 1, |mut sublist| {
            get!(sublist, 0, |mut item| {
                item.write(12);
            });
        });

        get!(owned, 2, |mut sublist| {
            get!(sublist, 0, |mut item| {
                item.write(10);
            });
        });

        get!(owned, 3, |mut sublist| {
            get!(sublist, 0, |mut item| {
                item.write(12);
            });
        });
    });
}

#[bench]
fn nested_list_read_value_in_4_sublists(b: &mut Bencher) {
    let mut arena = List::from(&[
        List::<Literal<'_, u8>>::with_capacity(2),
        List::<Literal<'_, u8>>::with_capacity(2),
        List::<Literal<'_, u8>>::with_capacity(2),
        List::<Literal<'_, u8>>::with_capacity(2),
    ]).create_buffer()
    .unwrap();

    let owned: List<'_, List<'_, Literal<'_, u8>>> = List::create(&mut arena).unwrap();

    b.iter(|| {
        owned.borrow(0, |sublist| {
            sublist.borrow(0, |item| {
                item.read();
            });
        });

        owned.borrow(1, |sublist| {
            sublist.borrow(0, |item| {
                item.read();
            });
        });

        owned.borrow(2, |sublist| {
            sublist.borrow(0, |item| {
                item.read();
            });
        });

        owned.borrow(3, |sublist| {
            sublist.borrow(0, |item| {
                item.read();
            });
        });
    });
}
