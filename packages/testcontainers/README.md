---
title: Testcontainers
---

`@defimetachain/testcontainers`

Provides easy to use and test lightweight, throwaway instances of MetaChain provisioned automatically in a Docker
container.

`MetaChainContainer` and `StartedMetaChainContainer` follows the convention defined
in [testcontainers/testcontainers-node](https://github.com/testcontainers/testcontainers-node)

```ts
let container: StartedMetaChainContainer;
let rpc: ethers.providers.JsonRpcProvider;

beforeEach(async () => {
  container = await new MetaChainContainer().start();
});

afterEach(async () => {
  await container.stop();
});

it('should createBlock', async function () {
  await container.createBlock();
  expect(await rpc.getBlockNumber()).toStrictEqual(1);
});
```
