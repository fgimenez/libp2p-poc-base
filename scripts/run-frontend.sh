#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
BASE_DIR="${SCRIPT_DIR}/.."

main() {
    {
        cd "${BASE_DIR}"
        rm -rf node_modules/wasm-net && \
            yarn add file:./wasm-net/pkg && \
            yarn && \
            yarn serve
    }
}

main "${@}"
