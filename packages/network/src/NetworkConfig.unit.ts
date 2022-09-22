import { MainNet, TestNet } from './NetworkConfig';

it('MainNet config should not drift', () => {
  expect(MainNet).toStrictEqual({
    chainId: 988,
    ports: {
      p2p: 30333,
      rpc: 9333,
      ws: 9944,
    },
  });
});

it('TestNet config should not drift', () => {
  expect(TestNet).toStrictEqual({
    chainId: 1988,
    ports: {
      p2p: 39333,
      rpc: 19933,
      ws: 19944,
    },
  });
});
