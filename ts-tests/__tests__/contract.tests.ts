import { MetaDContainer } from '../containers';
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, CONTRACT_ADDRESS, CHAIN_ID } from '../utils/constant';
import Test from '../build/contracts/Test.json';
import { ContractFactory, ethers } from 'ethers';

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

  const factory: ContractFactory = new ethers.ContractFactory(Test.abi, Test.bytecode, wallet);

  const contract = await factory.deploy();
  expect(contract.address).toStrictEqual(CONTRACT_ADDRESS);

  await container.generate();

  // call contract
  expect(await contract.name()).toStrictEqual('Meta');

  const mul = (await contract.mul(3, 7)).toNumber();
  expect(mul).toStrictEqual(21);

  const currentBlock = (await contract.getCurrentBlock()).toNumber();
  expect(currentBlock).toStrictEqual(1);

  const blockHash = await contract.getBlockHash(currentBlock);
  expect(blockHash).toStrictEqual(expect.any(String));

  const gasLimit = (await contract.getGasLimit()).toNumber();
  expect(gasLimit).toStrictEqual(75000000);

  const msgSender = await contract.getMsgSender();
  expect(msgSender).toStrictEqual(GENESIS_ACCOUNT);

  const promise = contract.max10(11);
  await expect(promise).rejects.toThrow('Value must not be greater than 10');
});
