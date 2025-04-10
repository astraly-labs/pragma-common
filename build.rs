fn main() {
    // Notify Cargo to re-run this script if the schema file changes
    println!("cargo:rerun-if-changed=capnp/schema.capnp");

    // Compile the Cap'n Proto schema and output the generated code to OUT_DIR
    capnpc::CompilerCommand::new()
        .file("schema.capnp")
        .output_path("src/generated")
        .run()
        .expect("Failed to compile Cap'n Proto schema");
}
