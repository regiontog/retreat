#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::{List, Literal};

fuzz_target!(|data: &[u8]| {
    let mut v = data.to_vec();
    let data = v.as_mut_slice();

    if let Ok((_, list)) = List::<List<Literal<char>>>::build(data) {
        for i in 0..list.len() {
            for j in 0..list[i].len() {
                list[i][j].read();
            }
        }
    }
});
