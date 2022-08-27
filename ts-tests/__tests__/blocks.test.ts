import { MetaDContainer } from '../containers';

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should generate', async () => {
  const b0 = await container.web3.eth.getBlockNumber();
  expect(b0).toStrictEqual(0);

  const block = await container.web3.eth.getBlock(0);
  expect(block).toStrictEqual({
    author: '0x0000000000000000000000000000000000000000',
    baseFeePerGas: 1000000000,
    difficulty: '0',
    extraData: '0x',
    gasLimit: 75000000,
    gasUsed: 0,
    hash: expect.any(String),
    logsBloom:
      '0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000',
    miner: '0x0000000000000000000000000000000000000000',
    nonce: '0x0000000000000000',
    number: 0,
    parentHash:
      '0x0000000000000000000000000000000000000000000000000000000000000000',
    receiptsRoot: expect.any(String),
    sha3Uncles: expect.any(String),
    size: 505,
    stateRoot: expect.any(String),
    timestamp: 0,
    totalDifficulty: '0',
    transactions: [],
    transactionsRoot: expect.any(String),
    uncles: []
  });

  await container.generate();

  const b1 = await container.web3.eth.getBlockNumber();
  expect(b1).toStrictEqual(1);
});
