import { MainNet, TestNet } from '@defimetachain/network';
import { ethers } from 'ethers';

import { MetaChainContainer, StartedMetaChainContainer } from '.';

describe('Testnet ethers.providers.JsonRpcProvider', () => {
  let container: StartedMetaChainContainer;
  let rpc: ethers.providers.JsonRpcProvider;

  beforeAll(async () => {
    container = await new MetaChainContainer().start();
    rpc = container.getEthersHttpProvider();
  });

  afterAll(async () => {
    await container.stop();
  });

  it('should have 0 hashrate', async () => {
    expect(await rpc.send('eth_hashrate', [])).toStrictEqual('0x0');
  });

  it('should have chainId', async () => {
    expect(Number(await rpc.send('net_version', []))).toStrictEqual(TestNet.chainId);
  });

  it('should have no account', async () => {
    expect(await rpc.send('eth_accounts', [])).toStrictEqual([]);
  });

  it('block author should be 0x0000000000000000000000000000000000000000', async () => {
    expect(await rpc.send('eth_coinbase', [])).toStrictEqual('0x0000000000000000000000000000000000000000');
  });
});

describe('MainNet ethers.providers.JsonRpcProvider', () => {
  let container: StartedMetaChainContainer;
  let rpc: ethers.providers.JsonRpcProvider;

  beforeAll(async () => {
    container = await new MetaChainContainer().withNetworkConfig(MainNet).start();
    rpc = container.getEthersHttpProvider();
  });

  afterAll(async () => {
    await container.stop();
  });

  it('should have chainId', async () => {
    expect(Number(await rpc.send('net_version', []))).toStrictEqual(MainNet.chainId);
  });
});

describe('utility method', () => {
  let container: StartedMetaChainContainer;
  let rpc: ethers.providers.JsonRpcProvider;

  beforeEach(async () => {
    container = await new MetaChainContainer().start();
    rpc = container.getEthersHttpProvider();
  });

  afterEach(async () => {
    await container.stop();
  });

  it('should createBlock', async () => {
    await container.createBlock();
    expect(await rpc.getBlockNumber()).toStrictEqual(1);
  });
});
