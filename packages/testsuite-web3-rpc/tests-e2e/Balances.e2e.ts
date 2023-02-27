import { ethers, BigNumber } from 'ethers';
import { MetaChainContainer, StartedMetaChainContainer } from '@defimetachain/testcontainers';
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  EXISTENTIAL_DEPOSIT,
} from '../src/utils/constant';

let container: StartedMetaChainContainer;
let rpc: ethers.providers.JsonRpcProvider;
const TEST_ACCOUNT = '0x1111111111111111111111111111111111111111';

const value = BigNumber.from(512);
const gasPrice = BigNumber.from(1_000_000_000);

// GENESIS_ACCOUNT_BALANCE - (21000 * gasPrice) - value;
const expectedGenesisBalance = BigNumber.from(GENESIS_ACCOUNT_BALANCE).sub(gasPrice.mul(21_000)).sub(value); // 0x0fffece68e75ae00

beforeAll(async () => {
  container = await new MetaChainContainer().start();
  rpc = container.getEthersHttpProvider();
});

afterAll(async () => {
  await container.stop();
});

it('should genesis balance setup correctly', async () => {
  const bal = await rpc.getBalance(GENESIS_ACCOUNT);
  expect(bal.toString()).toStrictEqual(GENESIS_ACCOUNT_BALANCE);
});

it('should transfer balance', async () => {
  const gBalBefore = await rpc.getBalance(GENESIS_ACCOUNT);
  expect(gBalBefore._hex).toStrictEqual(0x1152921504606846976);

  const tBalBefore = await rpc.getBalance(TEST_ACCOUNT);
  expect(tBalBefore).toStrictEqual(BigNumber.from(0));

  const wallet = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, rpc);

  await wallet.sendTransaction({
    to: TEST_ACCOUNT,
    value: value, // 512
    gasPrice: gasPrice,
    gasLimit: 0x100000,
  });

  await container.createBlock();

  const gBalAfter = await rpc.getBalance(GENESIS_ACCOUNT);
  expect(gBalAfter).toStrictEqual(expectedGenesisBalance);

  const tBalAfter = await rpc.getBalance(TEST_ACCOUNT);
  expect(tBalAfter).toStrictEqual(BigNumber.from(512));
});
