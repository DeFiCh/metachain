# DeFi Meta Chain

`DeFiCh/metachain` is a codename research & development
for [DFIP 2111-B: VOC: Ethereum Virtual Machine (EVM) Support](https://github.com/DeFiCh/dfips/issues/96)
. Proposed as a DFIP on Nov 2021; DFIP 2111-B provided DeFiChain with more flexibility to think
beyond what is possible today. It introduced a new dimension to the DeFiChain ecosystem, allowing us
to stretch the definition of Native DeFi.

- Smart contract capability through a turing-complete environment for faster paced innovation
- Embrace the multi-chain future enabling easier cross compatibility and extensibility.

## Developing & Contributing

`DeFiCh/metachain` maintains one permanent branch: `main`. Using shift-left development approaches,
all developers base their PR towards on `main` branch.

Thanks for contributing, appreciate all the help we can get. Feel free to make a pull-request, we
will guide you along the way to make it merge-able.

## Building
in project root directory, run

```bash
$ cargo build --release
```

## Run Locally

### Run Alice
```bash
$ ./target/release/meta-node --chain=local --alice --base-path=.local/a --port=30334 --ws-port 9944 --ws-external --rpc-cors=all --rpc-methods=Unsafe --rpc-external
```

### Connect to FE
```bash
https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
```

### Trigger to mine a block
```bash
$ curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "engine_createBlock", "params": [true, false, null]}' http://localhost:9933/
```

## Security issues

If you discover a security vulnerability in `DeFiCh/metachain`
, [please see submit it privately](https://github.com/DeFiCh/.github/blob/main/SECURITY.md).

## License & Disclaimer

By using `DeFiCh/metachain` (this repo), you (the user) agree to be bound
by [the terms of this license](LICENSE).
