extern crate noserc;

fn main() {
    noserc::NoserCompiler::new()
        .remove_prefix("schema")
        // .noser_path("::tests::noser")
        .file("schema/generated_source.noser")
        .run()
        .expect("noserc failed to compile");
}
