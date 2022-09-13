import { BigNumber } from 'ethers';
import { MetaDContainer } from '../src/containers';

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should generate', async () => {
  const b0 = await container.ethers.getBlockNumber();
  expect(b0).toStrictEqual(0);

  const block = await container.ethers.getBlock(0);
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

  await container.generate();

  const b1 = await container.ethers.getBlockNumber();
  expect(b1).toStrictEqual(1);
});
