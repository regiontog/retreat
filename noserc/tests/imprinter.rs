use noser::traits::DefaultWriter;
use noser::Literal;
use noserc::WriteTypeInfo;

#[allow(dead_code)]
#[derive(WriteTypeInfo)]
struct Unnamed<'a>(Literal<'a, u8>);

#[allow(dead_code)]
#[derive(WriteTypeInfo)]
struct Generic<'a, T> {
    x: Literal<'a, u8>,
    y: T,
}
