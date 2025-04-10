# Alloy Reth State DB provider
This crate implements the `StateProviderFactory` and related traits from [reth](https://github.com/paradigmxyz/reth). But instead of using a DB it fetches all state using RPC. This can be useful to not be dependent on a real database when testing e.g. reth ExEx.

## Example
See [evm_call.rs](./examples/evm_call.rs) for a complete example.

Quckstart:
```rust
// Init the provider
let provider = ProviderBuilder::new().on_http("https://eth.merkle.io".parse().unwrap());
// Init the db provider
let db_provider = AlloyRethProvider::new(provider, EthPrimitives::default());
// Use the StateProviderFactory
let state = db_provider.state_by_block_id(BlockId::number(16148323)).unwrap();
```

## Versioning
The version is always matching the compatible reth version. If this crate receives an update a suffix .e.g. `-v2` is added to the version.

## Acknowledgements
Many, many thanks to the team of [reth](https://github.com/paradigmxyz/reth) and [alloy-rs](https://github.com/alloy-rs/alloy) for the awesome work they do. Some parts of the trait implementation are taken from reth. Also, many thanks to [revm](https://github.com/bluealloy/revm). The `alloy_db` part is a copy/paste from revm because this part is not included in the latest revm version, and it makes it easier to be in sync with the latest Alloy version.

## License
This project is licensed under the [Apache 2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT). The part in `alloy_db` is licensed as [revm](https://github.com/bluealloy/revm) only under [MIT](./LICENSE-MIT).