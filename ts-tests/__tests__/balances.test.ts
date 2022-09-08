import { ethers, BigNumber } from 'ethers';
import { MetaDContainer } from '../containers';
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  EXISTENTIAL_DEPOSIT
} from '../utils/constant';

const container = new MetaDContainer();
const TEST_ACCOUNT = '0x1111111111111111111111111111111111111111';

const value = BigNumber.from(512); // 512, must be higher than ExistentialDeposit
const gasPrice = BigNumber.from(1_000_000_000); // 1000000000

// GENESIS_ACCOUNT_BALANCE - (21000 * gasPrice) - value;
const expectedGenesisBalance = BigNumber.from(GENESIS_ACCOUNT_BALANCE).sub(gasPrice.mul(21_000)).sub(value);
const expectedTestBalance = value.sub(EXISTENTIAL_DEPOSIT);

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should genesis balance setup correctly', async () => {
  const bal = await container.ethers.getBalance(GENESIS_ACCOUNT);
  expect(bal.toString()).toStrictEqual(GENESIS_ACCOUNT_BALANCE);
});

it('should transfer balance', async () => {
  const wallet = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, container.ethers);

  await wallet.sendTransaction({
    to: TEST_ACCOUNT,
    value: value,
    gasPrice: gasPrice,
    gasLimit: 0x100000
  });

  await container.generate();

  const gBal = await container.ethers.getBalance(GENESIS_ACCOUNT);
  expect(gBal).toStrictEqual(expectedGenesisBalance);

  const tBal = await container.ethers.getBalance(TEST_ACCOUNT);
  expect(tBal).toStrictEqual(expectedTestBalance);
});
