#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::Literal;

fuzz_target!(|data: &[u8]| {
    let mut data = data.to_vec();

    if let Ok(lit) = Literal::<i128>::create(&mut data) {
        lit.read();
    }
});
