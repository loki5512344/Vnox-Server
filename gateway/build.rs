fn main() {
    prost_build::compile_protos(&["../protocol/lnex.proto"], &["../protocol/"]).unwrap();
}
