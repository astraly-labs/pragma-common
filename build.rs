fn main() {
    // Notify Cargo to re-run this script if the schema file changes
    println!("cargo:rerun-if-changed=capnp/schema.capnp");

    // Compile the Cap'n Proto schema and output to OUT_DIR
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/schema.capnp")
        .run()
        .expect("Failed to compile Cap'n Proto schema");
}
