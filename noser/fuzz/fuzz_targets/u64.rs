#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::Literal;

fuzz_target!(|data: &[u8]| {
    let mut v = data.to_vec();
    let data = v.as_mut_slice();

    if let Ok((_, lit)) = Literal::<u64>::build(data) {
        lit.read();
    }
});