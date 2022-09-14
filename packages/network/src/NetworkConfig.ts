export interface NetworkConfig {
  chainId: 988;
  ports: {
    p2p: number;
    rpc: number;
    ws: number;
  };
}

export const MainNet: NetworkConfig = {
  chainId: 988,
  ports: {
    p2p: 30333,
    rpc: 9333,
    ws: 9944,
  },
};

export const TestNet: NetworkConfig = {
  chainId: 988,
  ports: {
    p2p: 39333,
    rpc: 19933,
    ws: 19944,
  },
};
