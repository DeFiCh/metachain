import Web3 from 'web3';
import { JsonRpcResponse } from 'web3-core-helpers';
import {
  GenericContainer,
  Network,
  StartedNetwork,
  StartedTestContainer
} from 'testcontainers';
import { ethers } from 'ethers';
import { Keyring } from '@polkadot/api';

const keyringEth = new Keyring({ type: 'ethereum' });

// constant
export const CHAIN_ID = 988;
export const GENESIS_ACCOUNT = '0x6be02d1d3665660d22ff9624b7be0551ee1ac91b';
export const GENESIS_ACCOUNT_PRIVATE_KEY =
  '0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342';
export const genesis = keyringEth.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY);

type MetaDNetwork = 'mainnet' | 'testnet';

export interface StartOptions {
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
    return 'canonbrother/metachain:pr';
  }

  static readonly MetaDPorts = {
    mainnet: {
      port: 30333,
      rpcPort: 9933,
      wsPort: 9944
    },
    testnet: {
      port: 39333,
      rpcPort: 19933,
      wsPort: 19944
    }
  };

  genericContainer: GenericContainer;
  startedContainer?: StartedTestContainer;
  startOptions?: StartOptions;
  private network?: StartedNetwork;

  web3?: Web3;
  ethersjs?: ethers.providers.JsonRpcProvider;

  constructor(
    readonly metaDNetwork: MetaDNetwork = 'testnet',
    readonly image: string = MetaDContainer.image,
    readonly provider: string = 'http'
  ) {
    this.genericContainer = new GenericContainer(image);
  }

  protected getCmd(opts: StartOptions): string[] {
    return [
      '--execution=Native', // Faster execution compare to `Wasm`
      '--no-telemetry', // disable connecting to substrate telemtry server
      '--no-prometheus', // do not expose a Prometheus exporter endpoint
      '--no-grandpa',
      // TODO(canonbrother): set up chain spec exclusively for test
      // '--chain= ./spec.json',
      // `-l${FRONTIER_LOG}`,
      `--port=${opts.port}`,
      `--rpc-port=${opts.rpcPort}`,
      `--ws-port=${opts.wsPort}`,
      '--rpc-external',
      '--ws-external',
      '--sealing=manual',
      '--force-authoring', // enable authoring even when offline
      '--rpc-cors=all',
      '--alice', // shortcut for `--name Alice --validator` with session keys for `Alice` added to keystore
      '--tmp' // run a temporary node
    ];
  }

  async start(startOptions: StartOptions = {}): Promise<void> {
    this.network = await new Network().start();

    this.startOptions = Object.assign(
      MetaDContainer.MetaDPorts[this.metaDNetwork],
      startOptions
    );
    const timeout = this.startOptions.timeout ?? 20000;

    this.startedContainer = await this.genericContainer
      .withName(this.generateName())
      .withNetworkMode(this.network.getName())
      .withCmd(this.getCmd(this.startOptions))
      .withExposedPorts(
        ...Object.values(MetaDContainer.MetaDPorts[this.metaDNetwork])
      )
      .withStartupTimeout(timeout)
      .start();

    const ip = this.startedContainer.getIpAddress(this.network.getName());

    this.web3 =
      this.provider !== 'http'
        ? new Web3(
            `ws://${ip}:${MetaDContainer.MetaDPorts[this.metaDNetwork].wsPort}`
          )
        : new Web3(
            `http://${ip}:${
              MetaDContainer.MetaDPorts[this.metaDNetwork].rpcPort
            }`
          );

    this.ethersjs = new ethers.providers.StaticJsonRpcProvider(
      `http://${ip}:${MetaDContainer.MetaDPorts[this.metaDNetwork].rpcPort}`,
      {
        chainId: CHAIN_ID,
        name: 'meta'
      }
    );
  }

  async stop(): Promise<void> {
    await this.startedContainer?.stop();
    await this.network?.stop();
  }

  async call(method: string, params: any[]): Promise<JsonRpcResponse> {
    return new Promise<JsonRpcResponse>((resolve, reject) => {
      (this.web3?.currentProvider as any).send(
        {
          jsonrpc: '2.0',
          id: Math.floor(Math.random() * 100000000000000),
          method,
          params
        },
        (error: Error | null, result: JsonRpcResponse) => {
          if (error) {
            reject(
              `Failed to send custom request (${method} (${params.join(
                ','
              )})): ${error.message || error.toString()}`
            );
          }
          resolve(result);
        }
      );
    });
  }

  // Create a block and finalize it.
  // It will include all previously executed transactions since the last finalized block.
  async generate(): Promise<string> {
    const response = await this.call('engine_createBlock', [true, true, null]);
    if (!response.result) {
      throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
    }
    await new Promise((resolve) => setTimeout(() => resolve(0), 500));
    return response.result.hash;
  }

  private generateName(): string {
    const rand = Math.floor(Math.random() * 10000000);
    return `${MetaDContainer.PREFIX}-${this.metaDNetwork}-${rand}`;
  }
}
