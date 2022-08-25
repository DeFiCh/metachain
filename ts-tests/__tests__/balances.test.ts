import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  EXISTENTIAL_DEPOSIT,
  MetaDContainer
} from '../containers';

const container = new MetaDContainer();
const TEST_ACCOUNT = '0x1111111111111111111111111111111111111111';

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should genesis balance setup correctly', async () => {
  const bal = await container.web3.eth.getBalance(GENESIS_ACCOUNT);
  expect(bal).toStrictEqual(GENESIS_ACCOUNT_BALANCE);
});

it('should transfer balance', async () => {
  const value = '0x200'; // 512, must be higher than ExistentialDeposit
  const gasPrice = '0x3B9ACA00'; // 1000000000
  const tx = await container.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      to: TEST_ACCOUNT,
      value: value,
      gasPrice: gasPrice,
      gas: '0x100000'
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  await container.call('eth_sendRawTransaction', [tx?.rawTransaction]);

  // GENESIS_ACCOUNT_BALANCE - (21000 * gasPrice) - value;
  const expectedGenesisBalance = (
    BigInt(GENESIS_ACCOUNT_BALANCE) -
    BigInt(21000) * BigInt(gasPrice) -
    BigInt(value)
  ).toString();
  const expectedTestBalance = (Number(value) - EXISTENTIAL_DEPOSIT).toString();
  expect(
    await container.web3.eth.getBalance(GENESIS_ACCOUNT, 'pending')
  ).toStrictEqual(expectedGenesisBalance);
  expect(
    await container.web3.eth.getBalance(TEST_ACCOUNT, 'pending')
  ).toStrictEqual(expectedTestBalance);

  await container.generate();

  expect(await container.web3.eth.getBalance(GENESIS_ACCOUNT)).toStrictEqual(
    expectedGenesisBalance
  );
  expect(await container.web3.eth.getBalance(TEST_ACCOUNT)).toStrictEqual(
    expectedTestBalance
  );
});
