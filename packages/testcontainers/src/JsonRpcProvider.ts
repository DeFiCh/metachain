import { ethers } from 'ethers';
import { MetaChainContainer } from './MetaChainContainer';

export function getHttpProvider(container: MetaChainContainer) {
  const port = container.config.ports.rpc;
  new ethers.providers.JsonRpcProvider(
    `http://127.0.0.1:${this.startedContainer.getMappedPort(container.config.ports.rpc)}`,
    {
      chainId: CHAIN_ID,
      name: 'meta',
    },
  );
}

export function getWsProvider(container: MetaChainContainer) {
  const port = container.config.ports.ws;
}
