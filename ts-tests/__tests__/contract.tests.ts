import { MetaDContainer } from '../containers';
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  FIRST_CONTRACT_ADDRESS
} from '../utils/constant';
import Test from '../build/contracts/Test.json';

const TEST_CONTRACT_BYTECODE = Test.bytecode;
const TEST_CONTRACT_DEPLOYED_BYTECODE = Test.deployedBytecode;

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should create contract', async () => {
  const tx = await container.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      data: TEST_CONTRACT_BYTECODE,
      value: '0x00',
      gasPrice: '0x3B9ACA00',
      gas: '0x100000'
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );

  expect(
    await container.call('eth_sendRawTransaction', [tx.rawTransaction])
  ).toStrictEqual(expect.any(String));

  // Verify the contract is not yet stored
  expect(
    await container.call('eth_getCode', [FIRST_CONTRACT_ADDRESS])
  ).toStrictEqual('0x');

  // Verify the contract is in the pending state
  expect(
    await container.call('eth_getCode', [FIRST_CONTRACT_ADDRESS, 'pending'])
  ).toStrictEqual(TEST_CONTRACT_DEPLOYED_BYTECODE);
});

it('should call contract', async () => {
  expect(
    await container.web3.eth.call({
      data: TEST_CONTRACT_BYTECODE
    })
  ).toStrictEqual(TEST_CONTRACT_DEPLOYED_BYTECODE);
});

it('eth_call at missing block returns error', async function () {
  const nonExistingBlockNumber = '999999';
  return expect(
    container.web3.eth.call(
      {
        data: TEST_CONTRACT_BYTECODE
      },
      nonExistingBlockNumber
    )
  ).rejects.toThrow('header not found');
});
