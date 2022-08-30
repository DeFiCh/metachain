import { MetaDContainer } from '../containers';
import {
  BLOCK_GAS_LIMIT,
  BLOCK_HASH_COUNT,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  FIRST_CONTRACT_ADDRESS
} from '../utils/constant';
import Test from '../build/contracts/Test.json';
import { AbiItem } from 'web3-utils';

const TEST_CONTRACT_BYTECODE = Test.bytecode;
const TEST_CONTRACT_ABI = Test.abi as AbiItem[];

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();

  // create contract
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
  await container.call('eth_sendRawTransaction', [tx.rawTransaction]);
  await container.generate();
});

afterAll(async () => {
  await container.stop();
});

it('should get tx by hash', async () => {
  const latest = await container.web3.eth.getBlock('latest');
  expect(latest.transactions.length).toStrictEqual(1);

  const hash = latest.transactions[0];
  const tx = await container.web3.eth.getTransaction(hash);
  expect(tx.hash).toStrictEqual(hash);
});

it('should return contract method result', async function () {
  const contract = new container.web3.eth.Contract(
    TEST_CONTRACT_ABI,
    FIRST_CONTRACT_ADDRESS,
    {
      from: GENESIS_ACCOUNT,
      gasPrice: '0x3B9ACA00'
    }
  );

  expect(await contract.methods.multiply(3).call()).toStrictEqual('21');
});

it('should get correct environmental block number', async function () {
  // Solidity `block.number` is expected to return the same height at which the runtime call was made.
  const contract = new container.web3.eth.Contract(
    TEST_CONTRACT_ABI,
    FIRST_CONTRACT_ADDRESS,
    {
      from: GENESIS_ACCOUNT,
      gasPrice: '0x3B9ACA00'
    }
  );
  let block = await container.web3.eth.getBlock('latest');
  expect(await contract.methods.currentBlock().call()).toStrictEqual(
    block.number.toString()
  );
  await container.generate();
  block = await container.web3.eth.getBlock('latest');
  expect(await contract.methods.currentBlock().call()).toStrictEqual(
    block.number.toString()
  );
});

// long test which looping 2400 blocks
it.skip('should get correct environmental block hash', async function () {
  // Solidity `blockhash` is expected to return the ethereum block hash at a given height.
  const contract = new container.web3.eth.Contract(
    TEST_CONTRACT_ABI,
    FIRST_CONTRACT_ADDRESS,
    {
      from: GENESIS_ACCOUNT,
      gasPrice: '0x3B9ACA00'
    }
  );
  let number = (await container.web3.eth.getBlock('latest')).number;
  let last = number + BLOCK_HASH_COUNT;
  for (let i = number; i <= last; i++) {
    let hash = (await container.web3.eth.getBlock('latest')).hash;
    expect(await contract.methods.blockHash(i).call()).toStrictEqual(hash);
    await container.generate();
  }
  // should not store more than `BLOCK_HASH_COUNT` hashes
  expect(await contract.methods.blockHash(number).call()).toStrictEqual(
    '0x0000000000000000000000000000000000000000000000000000000000000000'
  );
});

it('should get correct environmental block gaslimit', async function () {
  const contract = new container.web3.eth.Contract(
    TEST_CONTRACT_ABI,
    FIRST_CONTRACT_ADDRESS,
    {
      from: GENESIS_ACCOUNT,
      gasPrice: '0x3B9ACA00' // 1000000000
    }
  );
  expect(await contract.methods.gasLimit().call()).toStrictEqual(
    BLOCK_GAS_LIMIT.toString()
  );
});

it('should fail for missing parameters', async function () {
  const contract = new container.web3.eth.Contract(
    [{ ...TEST_CONTRACT_ABI[0], inputs: [] }],
    FIRST_CONTRACT_ADDRESS,
    {
      from: GENESIS_ACCOUNT,
      gasPrice: '0x3B9ACA00' // 1000000000
    }
  );
  await contract.methods
    .multiply()
    .call()
    .catch((err: Error) =>
      expect(err.message).toStrictEqual(
        `Returned error: VM Exception while processing transaction: revert`
      )
    );
});

it('should fail for too many parameters', async function () {
  const contract = new container.web3.eth.Contract(
    [
      {
        ...TEST_CONTRACT_ABI[0],
        inputs: [
          { internalType: 'uint256', name: 'a', type: 'uint256' },
          { internalType: 'uint256', name: 'b', type: 'uint256' }
        ]
      }
    ],
    FIRST_CONTRACT_ADDRESS,
    {
      from: GENESIS_ACCOUNT,
      gasPrice: '0x3B9ACA00'
    }
  );
  await contract.methods
    .multiply(3, 4)
    .call()
    .catch((err: Error) =>
      expect(err.message).toStrictEqual(
        `Returned error: VM Exception while processing transaction: revert`
      )
    );
});

it('should fail for invalid parameters', async function () {
  const contract = new container.web3.eth.Contract(
    [
      {
        ...TEST_CONTRACT_ABI[0],
        inputs: [{ internalType: 'address', name: 'a', type: 'address' }]
      }
    ],
    FIRST_CONTRACT_ADDRESS,
    { from: GENESIS_ACCOUNT, gasPrice: '0x3B9ACA00' }
  );
  await contract.methods
    .multiply('0x0123456789012345678901234567890123456789')
    .call()
    .catch((err: Error) =>
      expect(err.message).toStrictEqual(
        `Returned error: VM Exception while processing transaction: revert`
      )
    );
});
