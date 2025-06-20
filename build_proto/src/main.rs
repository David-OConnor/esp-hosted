use micropb_gen::{Config, EncodeDecode, Generator};

fn main() {
    let mut gen_ = Generator::new();
    gen_.use_container_heapless();
    // .encode_decode(EncodeDecode::Both)
    // .configure(
    //     ".",
    //     Config::new()
    //         // .no_clone_impl(true)
    //         // .no_accessors(true),
    // )

    gen_.compile_protos(&["esp_hosted_rpc.proto"], "../src/esp_hosted_proto.rs")
        .unwrap();
}
