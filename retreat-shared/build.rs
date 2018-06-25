extern crate capnpc;

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("src/schema")
        .file("src/schema/actions.capnp")
        .run().expect("schema compiler command");
}