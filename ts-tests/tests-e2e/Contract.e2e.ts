import { MetaDContainer } from '../src/containers';
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, CONTRACT_ADDRESS } from '../src/utils/constant';
import Test from '../artifacts/contracts/Test.sol/Test.json';
import { ethers } from 'ethers';

const container = new MetaDContainer();

beforeAll(async () => {
  await container.start();
});

afterAll(async () => {
  await container.stop();
});

it('should create and call contract', async () => {
  // create contract
  const wallet = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, container.ethers);

  const factory = new ethers.ContractFactory(Test.abi, Test.bytecode, wallet);

  const contract = await factory.deploy();
  expect(contract.address).toStrictEqual(CONTRACT_ADDRESS);

  await container.generate();

  // call contract
  expect(await contract.name()).toStrictEqual('Meta');
  expect(await contract.owner()).toStrictEqual(GENESIS_ACCOUNT);

  // test environmental
  const currentBlock = (await contract.getCurrentBlock()).toNumber();
  expect(currentBlock).toStrictEqual(1);

  const blockHash = await contract.getBlockHash(currentBlock);
  expect(blockHash).toStrictEqual(expect.any(String));

  const gasLimit = (await contract.getGasLimit()).toNumber();
  expect(gasLimit).toStrictEqual(75000000);

  // test functional
  const mul = (await contract.mul(3, 7)).toNumber();
  expect(mul).toStrictEqual(21);

  const c0 = (await contract.getCount()).toNumber();
  expect(c0).toStrictEqual(0);

  await contract.incr();
  await container.generate();

  const c1 = (await contract.getCount()).toNumber();
  expect(c1).toStrictEqual(1);

  await contract.setCount(25);
  await container.generate();

  const c25 = (await contract.getCount()).toNumber();
  expect(c25).toStrictEqual(25);

  const promise = contract.max10(11);
  await expect(promise).rejects.toThrow('Value must not be greater than 10');
});