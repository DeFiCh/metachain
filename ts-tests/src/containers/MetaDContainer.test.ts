import { MetaDContainer } from './index';

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should have 0 hashrate', async function () {
  expect(await container.call('eth_hashrate', [])).toStrictEqual('0x0');
});

it('should have chainId', async function () {
  expect(Number(await container.call('net_version', []))).toStrictEqual(MetaDContainer.MetaDChainId['testnet'].chainId);
});

it('should have no account', async function () {
  expect(await container.call('eth_accounts', [])).toStrictEqual([]);
});

it('block author should be 0x0000000000000000000000000000000000000000', async function () {
  expect(await container.call('eth_coinbase', [])).toStrictEqual('0x0000000000000000000000000000000000000000');
});
