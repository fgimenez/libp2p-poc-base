# libp2p-poc

## Rust Deps

```sh
# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# install wasm target
rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly

# Build the browser pkg (linked in root/package.json)
wasm-pack build --dev -- --features browser

# Build the native service
cargo build --bin bootnode

# Run the local service, take note of the multiaddr
target/debug/bootnode
```

## Frontend
```
rm -rf node_modules/wasm-net && yarn add file:./wasm-net/pkg && yarn && yarn serve
```
