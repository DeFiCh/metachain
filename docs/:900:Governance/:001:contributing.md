---
title: Contributing Guide
---

> Thanks for contributing, appreciate all the help we can get. Feel free to make a pull-request, we will guide you along
> the way to make it merge-able.

## Setting up

### The `main` Branch

`DeFiCh/metachain` maintains one permanent branch: `main`. This is the branch that all pull requests should be made
against. Using shift-left development practices, all developers base their PR towards the `main` branch. All pull
request must be "feature complete" and accompanied by all changes necessary for it to go into production
(documentations, tests, release title via the PR).

### Rust toolchain

```shell
# Setup Rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# You will also need nightly and wasm32-unknown-unknown for compilation
rustup default nightly
rustup target add wasm32-unknown-unknown
```

### Building the project

```shell
cargo build
```

### Running locally

```shell
cargo run -p meta-node -- --chain=local --alice --base-path=.local/a --port=30334 --ws-port 9944 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external
```

Connecting to the Front-end for visualization.

```shell
open "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer"
```

Trigger to mine the block:

```shell
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "engine_createBlock", "params": [true, false, null]}' http://localhost:9933/
```