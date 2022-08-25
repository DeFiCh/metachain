import { AbiItem } from 'web3-utils';
import Test from '../build/contracts/Storage.json';
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY
} from '../utils/constant';
import { MetaDContainer } from '../containers';

const TEST_CONTRACT_BYTECODE = Test.bytecode;
const TEST_CONTRACT_ABI = Test.abi as AbiItem[];

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('eth_getStorageAt', async function () {
  const contract = new container.web3.eth.Contract(TEST_CONTRACT_ABI);

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

  await container.generate();
  let receipt0 = await container.web3.eth.getTransactionReceipt(
    tx.transactionHash!
  );
  let contractAddress = receipt0.contractAddress;

  let getStorage0 = await container.call('eth_getStorageAt', [
    contractAddress,
    '0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc',
    'latest'
  ]);

  expect(getStorage0).toStrictEqual(
    '0x0000000000000000000000000000000000000000000000000000000000000000'
  );

  const tx1 = await container.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      to: contractAddress,
      data: contract.methods
        .setStorage(
          '0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc',
          '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef'
        )
        .encodeABI(),
      value: '0x00',
      gasPrice: '0x3B9ACA00',
      gas: '0x500000'
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );

  await container.call('eth_sendRawTransaction', [tx1.rawTransaction]);

  let getStoragePending = await container.call('eth_getStorageAt', [
    contractAddress,
    '0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc',
    'pending'
  ]);

  const expectedStorage =
    '0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef';

  expect(getStoragePending).toStrictEqual(expectedStorage);

  await container.generate();

  let getStorage1 = await container.call('eth_getStorageAt', [
    contractAddress,
    '0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc',
    'latest'
  ]);

  expect(getStorage1).toStrictEqual(expectedStorage);
});
