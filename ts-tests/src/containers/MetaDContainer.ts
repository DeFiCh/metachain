import { ethers } from 'ethers';
import { GenericContainer, StartedTestContainer, Network, StartedNetwork } from 'testcontainers';
import { CHAIN_ID } from '../utils/constant';
import { META_LOG } from '../utils/constant';

type MetaDNetwork = 'mainnet' | 'testnet';

export enum ChainSpec {
  META_DEV = 'meta-dev',
  META_LOCAL = 'meta-local',
  BIRTHDAY_DEV = 'birthday-dev',
  BIRTHDAY_LOCAL = 'birthday-local',
}

export interface StartOptions {
  chain?: ChainSpec;
  port?: number;
  rpcPort?: number;
  wsPort?: number;
  timeout?: number;
  spec?: number;
}

export class MetaDContainer {
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

  static readonly MetaDChainId = {
    mainnet: {
      chainId: 988,
      name: 'meta',
    },
    testnet: {
      chainId: 1988,
      name: 'birthday',
    },
  };

  genericContainer: GenericContainer;
  startedContainer?: StartedTestContainer;
  startOptions?: StartOptions;
  protected network?: StartedNetwork;

  ethers!: ethers.providers.JsonRpcProvider;

  constructor(
    readonly metaDNetwork: MetaDNetwork = 'testnet',
    readonly image: string = MetaDContainer.image,
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
      `--chain=${opts.chain || ChainSpec.BIRTHDAY_LOCAL}`,
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

    this.startOptions = Object.assign(startOptions, MetaDContainer.MetaDPorts[this.metaDNetwork]);
    const timeout = this.startOptions.timeout ?? 100_000;

    this.startedContainer = await this.genericContainer
      .withName(this.generateName())
      .withNetworkMode(this.network.getName())
      .withCmd(this.getCmd(this.startOptions))
      .withExposedPorts(...Object.values(MetaDContainer.MetaDPorts[this.metaDNetwork]))
      .withStartupTimeout(timeout)
      .start();

    this.ethers =
      this.provider !== 'http'
        ? new ethers.providers.JsonRpcProvider(
            `ws://127.0.0.1:${this.startedContainer.getMappedPort(
              MetaDContainer.MetaDPorts[this.metaDNetwork].wsPort,
            )}`,
            MetaDContainer.MetaDChainId[this.metaDNetwork],
          )
        : new ethers.providers.JsonRpcProvider(
            `http://127.0.0.1:${this.startedContainer.getMappedPort(
              MetaDContainer.MetaDPorts[this.metaDNetwork].rpcPort,
            )}`,
            MetaDContainer.MetaDChainId[this.metaDNetwork],
          );
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
      throw new MetaDRpcError(error);
    }
  }

  // Create a block and finalize it.
  // It will include all previously executed transactions since the last finalized block.
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
    return `${MetaDContainer.PREFIX}-${this.metaDNetwork}-${rand}`;
  }
}

/**
 * RPC error from container
 */
export class MetaDRpcError extends Error {
  constructor(error: { code: number; message: string }) {
    super(`MetaDRpcError: ' ${error.message}', code: ${error.code}`);
  }
}
