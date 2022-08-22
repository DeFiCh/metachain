import { MetaDContainer, genesis } from '../containers';

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should get balance', async () => {
  const bal = await container.web3?.eth.getBalance(genesis.address);
  expect(bal).toStrictEqual('340282366920938463463374607431768210955');
});
