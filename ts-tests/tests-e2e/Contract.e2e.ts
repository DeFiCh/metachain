import { MetaChainContainer, StartedMetaChainContainer } from '@defimetachain/testcontainers';
import { Network } from 'testcontainers';
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, CONTRACT_ADDRESS } from '../src/utils/constant';
import Test from '../artifacts/contracts/Test.sol/Test.json';
import { ethers } from 'ethers';

let container: StartedMetaChainContainer;
let rpc: ethers.providers.JsonRpcProvider;

beforeAll(async () => {
  const network = await new Network().start();
  container = await new MetaChainContainer().withNetwork(network).start();
  rpc = container.getEthersHttpProvider();
});

afterAll(async () => {
  await container.stop();
});

it('should create and call contract', async () => {
  // create contract
  const wallet = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, rpc);

  const factory = new ethers.ContractFactory(Test.abi, Test.bytecode, wallet);

  const contract = await factory.deploy();
  expect(contract.address).toStrictEqual(CONTRACT_ADDRESS);

  await container.createBlock();

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
  await container.createBlock();

  const c1 = (await contract.getCount()).toNumber();
  expect(c1).toStrictEqual(1);

  await contract.setCount(25);
  await container.createBlock();

  const c25 = (await contract.getCount()).toNumber();
  expect(c25).toStrictEqual(25);

  const promise = contract.max10(11);
  await expect(promise).rejects.toThrow('Value must not be greater than 10');
});
