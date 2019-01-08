use noser::Literal;
use noserc::StaticEnum;

#[allow(dead_code)]
#[derive(StaticEnum)]
enum StaticEnum<'a> {
    Val { x: Literal<'a, u32> },
    OtherVar,
}