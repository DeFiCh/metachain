---
title: Contributing Code
---

Thanks for contributing, appreciate all the help we can get. Feel free to make a pull-request, we will guide you along
the way to make it merge-able.

## Setting up

### The `main` Branch

`DeFiCh/metachain` maintains one permanent branch: `main`. This is the branch that all pull requests should be made
against. Using shift-left development practices, all developers base their PR towards the `main` branch. All pull
request must be "feature complete" and accompanied by all changes necessary for it to go into production.

### What is a Feature Complete PR?

- Updated Documentation
- Accompanying Tests
  - Unit Test (Rust)
  - Integration (Rust)
  - End-to-end (TypeScript)
- Release Notes & Fully Detailed PR (via "Semantic PR" with release drafter)

## Rust & Node Toolchains?

`DeFiCh/metachain` is a monolithic repository that features two different toolchain for different purposes, but they are
contextually placed in to same repository to maintain a singular source of truth.

## Rust toolchain

`./meta/*` produces the binary that contain the consensus logic

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

## Node toolchain

In `./packages/*`, it contains E2E test and packages for using and building on DeFi Meta Chain.

```shell
pnpm install
```

### Installing pnpm

If you have the brew package manager installed, you can install pnpm using the following command `brew install pnpm`.
Otherwise, you should use the [standalone script to install](https://pnpm.io/installation#using-a-standalone-script). It
is not advised to use `npm install` to install pnpm as it can be overridden when you switch environments
(e.g. `nvm use`).

### Why did we use pnpm?

With pnpm, it creates a [non-flat node_modules](https://pnpm.io/motivation#creating-a-non-flat-node_modules-directory)
directory. When installing dependencies with npm or Yarn Classic, all packages are hoisted to the root of the modules'
directory. As a result, the source code has access to dependencies that are not added as dependencies to the project.

Hosting is an incredibly challenging problem in mono-repo projects, node module resolver needs to know where to look for
dependencies. When project dependencies are not hoisted automatically due to version conflicts (see below), it creates
multiple copies of a dependency at different layers of your package hierarchy, makes it impossible to manage.

```text
├── package.json
├── services/
│   └── service-a
│       ├── AService
│       └── node_modules
│           └── @birthdaycloud/deps-b@1.0.0
└── node_modules
    └── @birthdaycloud/deps-a@1.0.0
```

Given the above, we're using the node default package resolution technique, and `@foo/deps-a` depends on `@foo/deps-b`
and you require `@foo/deps-a` for `service-a`. Since `@foo/deps-b` is a dependency of `@foo/deps-a` and it isn't
intrinsically defined within `service-a/package.json`, the default npm behavior would be to deduplicate by hoisting it
to the root `./node_modules`. The runtime would panic as it won't know how to resolve `@foo/deps-b`
from `service-a/node_modules` since it's hoisted up while `@foo/deps-b` isn't.
