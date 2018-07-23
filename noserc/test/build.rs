extern crate noserc;

fn main() {
    noserc::NoserCompiler::new()
        .out_dir("test_out")
        .remove_prefix("schema")
        .file("schema/generated_source.noser")
        .run()
        .expect("noserc failed to compile");
}
