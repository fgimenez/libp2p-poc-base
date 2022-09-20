#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
BASE_DIR="${SCRIPT_DIR}/.."

main() {
    {
        cd "${BASE_DIR}" && ./scripts/compile-backend.sh

        ./wasm-net/target/debug/bootnode &
        BOOTNODE_PID=$!
        trap EXIT "kill -9 ${BOOTNODE_PID}"

        cd wasm-net && wasm-pack test --chrome --headless
    }
}

main "${@}"
