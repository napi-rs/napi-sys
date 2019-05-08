# napi-sys

[![Travis Build Status][travis-badge]][travis-url]
[![AppVeyor Build Status][appveyor-badge]][appveyor-url]

Low-level N-API bindings for Node.js addons written in Rust.

See the [napi][] crate for the high-level API.

[napi]: https://github.com/napi-rs/napi
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/c3j49iki8o83q6ey/branch/master?svg=true
[appveyor-url]: https://ci.appveyor.com/project/aqrln/napi-sys
[travis-badge]: https://travis-ci.org/napi-rs/napi-sys.svg?branch=master
[travis-url]: https://travis-ci.org/napi-rs/napi-sys

## Building

Requirements:

- Install bindgen (`cargo install bindgen`) and [its requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)
- Install rustfmt (`rustup component add rustfmt`)
- Install [node.js](https://nodejs.org)

After doing that, you can build a new version of the bindings like so:

    ./scripts/update-bindings.sh
