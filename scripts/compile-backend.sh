#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
BASE_DIR="${SCRIPT_DIR}/../wasm-net"

main() {
    {
        cd "${BASE_DIR}"
        cargo build && \
            wasm-pack build --dev -- --features browser
    }
}

main "${@}"
