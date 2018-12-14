#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::{List, Literal};

fuzz_target!(|data: &[u8]| {
    let mut data = data.to_vec();

    if let Ok(list) = List::<List<Literal<char>>>::create(&mut data) {
        for i in 0..list.capacity() {
            let sublist = list.borrow(i);

            for j in 0..sublist.capacity() {
                sublist.borrow(j).read();
            }
        }
    }
});
