struct MyType {
    field1: u8,
    list_of_things: ProtoList<u8>,
}

enum TypeK<T> {
    None,
    SomeThing(u8),
    OtherThing(Other<Some<T>>),
}

struct Other<T> {
    x: u8,
    y: u8,
    z: T,
}