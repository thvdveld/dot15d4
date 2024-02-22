# dot15d4

`dot15d4` is a IEEE 802.15.4 frame parsing library written in Rust.
It uses the same style of parsing and emitting as the [`smoltcp`](https://github.com/smoltcp-rs/smoltcp) library.

Another library that is similar to `dot15d4` is the [`ieee802154`](https://github.com/rust-iot/rust-ieee802.15.4) library.
However, `dot15d4` is more focused on implementing MAC layer functionality (unslotted CSMA and TSCH), and parsing frames.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
dot15d4 = "0.1"
```

For more information, see the [API documentation](https://docs.rs/dot15d4).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
