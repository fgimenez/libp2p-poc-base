#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
BASE_DIR="${SCRIPT_DIR}/.."

main() {
    {
        cd "${BASE_DIR}" && ./scripts/compile-backend.sh

        RUST_LOG=debug wasm-net/target/debug/bootnode
    }
}

main "${@}"
