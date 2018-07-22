#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::{List, Literal};

fuzz_target!(|data: &[u8]| {
    let mut v = data.to_vec();
    let data = v.as_mut_slice();

    if let Ok((_, list)) = List::<Literal<u8>>::build(data) {
        for i in 0..list.len() {
            list[i].read();
        }
    }
});

