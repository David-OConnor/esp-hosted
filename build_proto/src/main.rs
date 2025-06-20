use micropb_gen::{Config, EncodeDecode, Generator};

fn main() {
    let mut gen_ = Generator::new();
    gen_.use_container_heapless();

    // todo?
    gen_.configure(".", Config::new().max_bytes(32).max_len(8));

    gen_.configure(".wifi_country.cc", Config::new().max_bytes(2));

    // STA list – allow up to eight entries
    gen_.configure(".wifi_sta_list.sta", Config::new().max_len(50));



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
