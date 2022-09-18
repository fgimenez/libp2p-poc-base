#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
BASE_DIR="${SCRIPT_DIR}/.."
{
    cd "${BASE_DIR}"
    git submodule foreach git reset --hard
    git submodule init
    git submodule sync
    for patch in patches/*; do
        echo "applying patch: ${patch}"
        git apply "${patch}"
    done;
}
