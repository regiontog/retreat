// use freyr::ReadOnly;

// struct Wrapper<'a> {
//     value: &'a u32,
// }

// #[test]
// fn should_not_compile() {
//     let ro = {
//         let block = [0; 12];

//         ReadOnly::new(Wrapper { value: &block[0] })
//     };

//     println!("{:?}", ro.value);
// }
