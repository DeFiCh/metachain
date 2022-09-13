import { MetaChainContainer, StartedMetaChainContainer } from './';
import { ethers } from 'ethers';
import { TestNet } from '@defimetachain/network';

describe('ethers.providers.JsonRpcProvider', function () {
  let container: StartedMetaChainContainer;
  let rpc: ethers.providers.JsonRpcProvider;

  beforeAll(async () => {
    container = await new MetaChainContainer().start();
  });

  afterAll(async () => {
    await container.stop();
  });

  it('should have 0 hashrate', async function () {
    expect(await rpc.send('eth_hashrate', [])).toStrictEqual('0x0');
  });

  it('should have chainId', async function () {
    expect(Number(await rpc.send('net_version', []))).toStrictEqual(TestNet.chainId);
  });

  it('should have no account', async function () {
    expect(await rpc.send('eth_accounts', [])).toStrictEqual([]);
  });

  it('block author should be 0x0000000000000000000000000000000000000000', async function () {
    expect(await rpc.send('eth_coinbase', [])).toStrictEqual('0x0000000000000000000000000000000000000000');
  });
});

describe('utility method', function () {
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
});
