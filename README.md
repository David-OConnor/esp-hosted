# ESP Hosted
For connecting to an [ESP-Hosted-MCU](https://github.com/espressif/esp-hosted-mcu) from a Host MCU with firmware
written in rust.

Compatible with ESP-Hosted-MCU 2.0.6.

For details on ESP-HOSTED-MCU's protocol see [this document](/esp_hosted_protocol.md).

This library currently only implements a small subset of available functionality. We plan to expand this after creating
a suitable workflow that decodes the protobuf `esp_hosted_rpc.proto`.

Running the application in the `build_proto` subfolder builds a rust module from the .proto file.

Example use:
```rust
fn init(uart: &mut Uart) {
    // Write could also be SPI, dma etc.
    let mut write = |buf: &[u8]| {
        uart.write(buf).map_err(|e| {
            println!("Uart write error: {:?}", e);
            EspError::Comms
        })
    };

    let heartbeat_cfg = RpcReqConfigHeartbeat {
        enable: true,
        duration: 2,
    };

    if esp_hosted::cfg_heartbeat(&mut write, &heartbeat_cfg).is_err() {
       // A/R.
    }
}
```

Then in your UART, SPI etc ISR:

```rust
#[interrupt]
fn USART2() {
    // ...
    match esp_hosted::parse_msg(rx_buf) {
        Ok((header, rpc, data_buf)) => {
            println!("\nHeader: {:?}", header);
            println!("RPC: {:?}", rpc);
            println!("Data buf: {:?}", data_buf);
            
            if rpc.msg_id == RpcId::EventHeartbeat {
                // A/R. You could then parse `data_buf` into the appropriate response type.
            }
        }
    }
}
```