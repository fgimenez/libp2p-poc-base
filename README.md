[![Test](https://github.com/fgimenez/libp2p-poc/actions/workflows/test.yml/badge.svg)](https://github.com/fgimenez/libp2p-poc/actions/workflows/test.yml)

# libp2p-poc

Based on https://github.com/shazow/rnges.us/tree/feature/vue-setup/wasm-net

Connectivity tests between libp2p nodes: browser <-> desktop, browser <-> browser.

## Prerrequisites
```sh
# install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# install wasm target
rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly

# Install nodejs

# Install yarn
```

## Running automated tests:
```
./scripts/run-tests.sh
```

## Running the example app locally
### Sync submodules and apply patches
```sh
./scripts/apply-patches.sh
```

### Backend node
On a shell console run:
```sh
./scripts/run-backend.sh
```
You wil see the logs from the desktop node.

### Frontend
On a different shell console:
```sh
./scripts/run-backend.sh
```
Once the app is running open http://localhost:8080 in a browser (tested on Chrome)
and click on `Connect`.

This will trigger the connection from the browser to the backend node, you will
start seeing log entries about the interaction in the backend node shell console
open previously.

You can also open the browser console to check the logs printed by the browser node.
