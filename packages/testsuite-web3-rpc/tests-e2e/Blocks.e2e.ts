import { ethers, BigNumber } from 'ethers';
import { MetaChainContainer, StartedMetaChainContainer } from '@defimetachain/testcontainers';

let container: StartedMetaChainContainer;
let rpc: ethers.providers.JsonRpcProvider;

beforeAll(async () => {
  container = await new MetaChainContainer().start();
  rpc = container.getEthersHttpProvider();
});

afterAll(async () => {
  await container.stop();
});

it('should generate', async () => {
  const b0 = await rpc.getBlockNumber();
  expect(b0).toStrictEqual(0);

  const block = await rpc.getBlock(0);
  expect(block).toStrictEqual({
    hash: expect.any(String),
    miner: '0x0000000000000000000000000000000000000000',
    parentHash: '0x0000000000000000000000000000000000000000000000000000000000000000',
    number: 0,
    timestamp: 0,
    nonce: '0x0000000000000000',
    difficulty: 0,
    gasLimit: BigNumber.from(75_000_000),
    gasUsed: BigNumber.from(0),
    extraData: '0x',
    transactions: [],
    baseFeePerGas: BigNumber.from(1_000_000_000),
    _difficulty: BigNumber.from(0),
  });

  await container.createBlock();

  const b1 = await rpc.getBlockNumber();
  expect(b1).toStrictEqual(1);
});
