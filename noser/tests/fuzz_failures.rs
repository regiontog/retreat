

use noser::traits::*;
use noser::{List, Literal};

#[test]
fn crash() {
    if let Ok(list) =
        List::<List<'_, Literal<'_, char>>>::create(&mut [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    {
        list.borrow(0, |_| {});
    }
}

#[test]
fn crash2() {
    if let Ok(list) = List::<List<'_, Literal<'_, char>>>::create(&mut [
        0x01, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]) {
        list.borrow(0, |_| {});
    }
}
