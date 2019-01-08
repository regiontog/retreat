use freyr::Lazy;

#[test]
fn only_called_once() {
    let mut side_effect = 0;

    let mut lazy_val = Lazy::new(|| {
        side_effect += 1;
        f32::ln(256.) + 2.
    });

    lazy_val.get();
    lazy_val.get();
    assert_eq!(side_effect, 1);
}

#[test]
fn correct_val() {
    let mut lazy_val = Lazy::new(|| f32::ln(256.) + 2.);

    assert_eq!(*lazy_val.get(), f32::ln(256.) + 2.);
}

#[test]
fn correct_cache() {
    let mut lazy_val = Lazy::new(|| f32::ln(256.) + 2.);

    lazy_val.get();
    assert_eq!(*lazy_val.get(), f32::ln(256.) + 2.);
}
