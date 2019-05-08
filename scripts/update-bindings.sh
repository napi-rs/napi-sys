#!/bin/bash

set -e

# make sure bindgen and rustfmt are available
# otherwise throw an error
which bindgen > /dev/null || (echo "`bindgen` not available, please install it with `cargo install bindgen`" && exit 1)
(rustup component list | grep rustfmt > /dev/null) || (echo "`rustfmt` not available, please install it with `rustup component add rustfmt`" && exit 1)

NAPI_HEADER=`realpath "$(dirname $(which node))/../include/node/node_api.h"`

echo "Building bindings with header file '$NAPI_HEADER'..."

bindgen -o src/bindings.rs \
        --whitelist-function 'napi_.+' \
        --whitelist-type 'napi_.+' \
        --default-enum-style 'rust' \
        "$NAPI_HEADER"

cargo fmt -- --check src/bindings.rs
cargo test

echo "Bindings generated successfully."
