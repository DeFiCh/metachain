export enum ChainSpec {
  META_DEV = 'meta-dev',
  META_LOCAL = 'meta-local',
  BIRTHDAY_DEV = 'birthday-dev',
  BIRTHDAY_LOCAL = 'birthday-local',
}

export interface NetworkConfig {
  chainId: 988 | 1988;
  chain: ChainSpec;
  ports: {
    p2p: number;
    rpc: number;
    ws: number;
  };
}

export const MainNet: NetworkConfig = {
  chainId: 988,
  chain: ChainSpec.META_LOCAL,
  ports: {
    p2p: 30333,
    rpc: 9333,
    ws: 9944,
  },
};

export const TestNet: NetworkConfig = {
  chainId: 1988,
  chain: ChainSpec.BIRTHDAY_LOCAL,
  ports: {
    p2p: 39333,
    rpc: 19933,
    ws: 19944,
  },
};
