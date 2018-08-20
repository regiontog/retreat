#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::{List, Literal};

fuzz_target!(|data: &[u8]| {
    let mut v = data.to_vec();
    let data = v.as_mut_slice();

    if let Ok(list) = List::<List<Literal<char>>>::create(data) {
        for i in 0..list.capacity() {
            list.borrow(i, |sublist| {
                for j in 0..sublist.capacity() {
                    sublist.borrow(j, |item| {
                        item.read();
                    });
                }
            });
        }
    }
});
