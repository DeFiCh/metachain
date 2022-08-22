import { MetaDContainer, CHAIN_ID } from './';

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should have 0 hashrate', async function () {
  expect(await container.web3?.eth.getHashrate()).toStrictEqual(0);
});

it('should have chainId', async function () {
  expect(await container.web3?.eth.getChainId()).toStrictEqual(CHAIN_ID);
});

it('should have no account', async function () {
  expect(await container.web3?.eth.getAccounts()).toStrictEqual([]);
});

it('block author should be 0x0000000000000000000000000000000000000000', async function () {
  // This address `0x1234567890` is hardcoded into the runtime find_author
  // as we are running manual sealing consensus.
  expect(await container.web3?.eth.getCoinbase()).toStrictEqual(
    '0x0000000000000000000000000000000000000000'
  );
});
