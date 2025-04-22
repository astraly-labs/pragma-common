fn main() {
    if std::env::var("CARGO_FEATURE_PROTO").is_ok() {
        prost_build::compile_protos(&["schema/entries.proto"], &["schema/"]).unwrap();
    }
}
