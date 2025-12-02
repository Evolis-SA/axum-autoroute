#!/bin/bash

ROOTDIR=`cd $(dirname $0)/..; pwd`

CLIPPY_PEDANTIC=( \
    -D"clippy::pedantic" \
    -A"clippy::doc_markdown" \
    -A"clippy::wildcard_imports" \
    -A"clippy::module_name_repetitions" \
    -A"clippy::struct_excessive_bools" \
    -A"clippy::explicit_deref_methods" \
    -A"clippy::if_not_else" \
)
CLIPPY_CARGO=( \
    -D"clippy::cargo" \
    -A"clippy::multiple_crate_versions"
)

echo "Running: clippy $@ -- --no-deps -Dwarnings \${CLIPPY_PEDANTIC[*]} \${CLIPPY_CARGO[*]}"
cargo clippy "$@" -- --no-deps -Dwarnings ${CLIPPY_PEDANTIC[*]} ${CLIPPY_CARGO[*]}