# soulfire
Interactive Fiction and AI experience app.

A single-user, local, BYOK desktop/mobile app being ported to Tauri v2 + React with a Rust core.
See [`docs/OG_PARITY_ROADMAP.md`](docs/OG_PARITY_ROADMAP.md) for the current roadmap,
[`specs/`](specs/) for the design source of truth, and [`AGENTS.md`](AGENTS.md) for how it's built.

The current Rust workspace is intentionally small: `soulfire-core` contains the product/domain core,
and `ai-client` remains as a provider transport adapter.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
