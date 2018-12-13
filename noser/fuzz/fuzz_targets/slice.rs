#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate noser;

use noser::traits::Build;
use noser::Literal;

fuzz_target!(|data: &[u8]| {
    let byte_arr = <&[u8]>::read(data);

    let mut temp;

    if let Ok(lit) = Literal::<&[u8]>::create(&mut data) {
        for byte in lit.read() {
            temp = *byte;
        }
    }
});
