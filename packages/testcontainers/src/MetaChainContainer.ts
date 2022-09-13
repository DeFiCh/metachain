import { ethers } from 'ethers';
import { GenericContainer, StartedTestContainer, Network, StartedNetwork } from 'testcontainers';
import { CHAIN_ID } from '../utils/constant';
import { NetworkConfig } from '@defimetachain/network';

const META_LOG = 'info';

type MetaDNetwork = 'mainnet' | 'testnet';

export interface StartOptions {
  port?: number;
  rpcPort?: number;
  wsPort?: number;
  timeout?: number;
  spec?: number;
}

export class MetaChainContainer {
  static readonly PREFIX = 'metachain-testcontainers-';

  static get image(): string {
    if (process?.env?.METACHAIN_DOCKER_IMAGE !== undefined) {
      return process.env.METACHAIN_DOCKER_IMAGE;
    }
    return 'ghcr.io/defich/metachain:af2e7d03b061352491d550c8923d1dfac4f65095';
  }

  static readonly MetaDPorts = {
    mainnet: {
      port: 30333,
      rpcPort: 9933,
      wsPort: 9944,
    },
    testnet: {
      port: 39333,
      rpcPort: 19933,
      wsPort: 19944,
    },
  };

  genericContainer: GenericContainer;
  startedContainer?: StartedTestContainer;
  startOptions?: StartOptions;
  protected network?: StartedNetwork;

  ethers!: ethers.providers.JsonRpcProvider;

  constructor(
    public readonly config: NetworkConfig,
    readonly metaDNetwork: MetaDNetwork = 'testnet',
    readonly image: string = MetaChainContainer.image,
    readonly provider: string = 'http',
  ) {
    this.genericContainer = new GenericContainer(image);
  }

  protected getCmd(opts: StartOptions): string[] {
    return [
      '--execution=Native', // Faster execution compare to `Wasm`
      '--no-telemetry', // disable connecting to substrate telemtry server
      '--no-prometheus', // do not expose a Prometheus exporter endpoint
      '--no-grandpa',
      `-l${META_LOG}`,
      `--port=${opts.port}`,
      `--rpc-port=${opts.rpcPort}`,
      `--ws-port=${opts.wsPort}`,
      '--rpc-external',
      '--ws-external',
      '--sealing=manual',
      '--force-authoring', // enable authoring even when offline
      '--rpc-cors=all',
      '--alice', // shortcut for `--name Alice --validator` with session keys for `Alice` added to keystore, required by manual sealing to author the blocks
      '--tmp', // run a temporary node
    ];
  }

  async start(startOptions: StartOptions = {}): Promise<void> {
    this.network = await new Network().start();

    this.startOptions = Object.assign(MetaChainContainer.MetaDPorts[this.metaDNetwork], startOptions);
    const timeout = this.startOptions.timeout ?? 100_000;

    this.startedContainer = await this.genericContainer
      .withName(this.generateName())
      .withNetworkMode(this.network.getName())
      .withCmd(this.getCmd(this.startOptions))
      .withExposedPorts(...Object.values(MetaChainContainer.MetaDPorts[this.metaDNetwork]))
      .withStartupTimeout(timeout)
      .start();

    this.ethers =
      this.provider !== 'http'
        ? new ethers.providers.JsonRpcProvider(
            `ws://127.0.0.1:${this.startedContainer.getMappedPort(
              MetaChainContainer.MetaDPorts[this.metaDNetwork].wsPort,
            )}`,
            {
              chainId: CHAIN_ID,
              name: 'meta',
            },
          )
        : new ethers.providers.JsonRpcProvider(
            `http://127.0.0.1:${this.startedContainer.getMappedPort(
              MetaChainContainer.MetaDPorts[this.metaDNetwork].rpcPort,
            )}`,
            {
              chainId: CHAIN_ID,
              name: 'meta',
            },
          );
  }

  getEthersHttpProvider() {
    const host = this.startedContainer!.getHost();
    const rpc = this.startedContainer!.getMappedPort(this.config.ports.rpc);
    return new ethers.providers.JsonRpcProvider(`http://${host}:${rpc}`, {
      chainId: this.config.chainId,
      name: 'meta',
    });
  }

  getEthersWsProvider() {
    const host = this.startedContainer!.getHost();
    const rpc = this.startedContainer!.getMappedPort(this.config.ports.ws);
    return new ethers.providers.JsonRpcProvider(`ws://${host}:${rpc}`, {
      chainId: this.config.chainId,
      name: 'meta',
    });
  }

  async stop(): Promise<void> {
    await this.startedContainer?.stop();
    await this.network?.stop();
  }

  async call(method: string, params: any[]): Promise<any> {
    try {
      return this.ethers.send(method, params);
    } catch (err: any) {
      const { error } = JSON.parse(err.body);
      throw new MetaChainRpcError(error);
    }
  }

  /**
   * Create a block and finalize it.
   * It will include all previously executed transactions since the last finalized block.
   */
  async generate(): Promise<string> {
    const result = await this.call('engine_createBlock', [true, true, null]);
    if (!result) {
      throw new Error(`Unexpected result: ${JSON.stringify(result)}`);
    }
    await new Promise((resolve) => setTimeout(() => resolve(0), 500));
    return result.hash;
  }

  private generateName(): string {
    const rand = Math.floor(Math.random() * 10000000);
    return `${MetaChainContainer.PREFIX}-${this.metaDNetwork}-${rand}`;
  }
}

/**
 * JSON RPC error from MetaChainContainer
 */
export class MetaChainRpcError extends Error {
  constructor(error: { code: number; message: string }) {
    super(`MetaChainRpcError: ' ${error.message}', code: ${error.code}`);
  }
}
