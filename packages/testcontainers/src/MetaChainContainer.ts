import { NetworkConfig, TestNet } from '@defimetachain/network';
import { ethers } from 'ethers';
import { GenericContainer, Network, StartedTestContainer } from 'testcontainers';
import { AbstractStartedContainer } from 'testcontainers/dist/modules/abstract-started-container';

export class MetaChainContainer extends GenericContainer {
  constructor(protected readonly config: NetworkConfig = TestNet, image: string = MetaChainContainer.image) {
    super(image);
  }

  static get image(): string {
    if (process?.env?.METACHAIN_DOCKER_IMAGE !== undefined) {
      return process.env.METACHAIN_DOCKER_IMAGE;
    }
    return 'ghcr.io/defich/metachain:cc77218f794ac2c05e76007ca2c8b4e890686903';
  }

  protected getCmd(): string[] {
    return [
      '--execution=Native', // Faster execution compare to `Wasm`
      '--no-telemetry', // disable connecting to substrate telemetry server
      '--no-prometheus', // do not expose a Prometheus exporter endpoint
      '--no-grandpa',
      `--chain=${this.config.spec}`,
      `--port=${this.config.ports.p2p}`,
      `--rpc-port=${this.config.ports.rpc}`,
      `--ws-port=${this.config.ports.ws}`,
      '--rpc-external',
      '--ws-external',
      '--sealing=manual',
      '--force-authoring', // enable authoring even when offline
      '--rpc-cors=all',
      '--alice', // shortcut for `--name Alice --validator` with session keys for `Alice` added to keystore, required by manual sealing to author the blocks
      '--tmp', // run a temporary node,
      '-linfo',
    ];
  }

  public async start(configModifiers = {}): Promise<StartedMetaChainContainer> {
    const network = await new Network().start();
    const defaultConfig = {
      exposedPorts: Object.values(this.config.ports),
      networkMode: network.getName(),
      cmd: this.getCmd(),
      startupTimeout: 120_000,
    };
    const config = { ...defaultConfig, ...configModifiers };
    this.withExposedPorts(config.exposedPorts)
      .withNetworkMode(config.networkMode)
      .withCmd(config.cmd)
      .withStartupTimeout(config.startupTimeout);

    return new StartedMetaChainContainer(await super.start(), this.config);
  }
}

export class StartedMetaChainContainer extends AbstractStartedContainer {
  /**
   * @protected JsonRpcProvider for Container utility methods
   */
  protected readonly rpc: ethers.providers.JsonRpcProvider;

  constructor(startedTestContainer: StartedTestContainer, protected readonly config: NetworkConfig) {
    super(startedTestContainer);
    this.rpc = this.getEthersHttpProvider();
  }

  /**
   * Create a block and finalize it.
   * It will include all previously executed transactions since the last finalized block.
   */
  async createBlock(): Promise<string> {
    const result = await this.rpc.send('engine_createBlock', [true, true, null]);

    if (result === undefined) {
      throw new Error(`unexpected result: ${JSON.stringify(result)}`);
    }

    await new Promise((resolve) => {
      setTimeout(() => resolve(0), 500);
    });
    return result.hash;
  }

  getEthersHttpProvider() {
    const host = this.getHost();
    const rpc = this.getMappedPort(this.config.ports.rpc);
    return new ethers.providers.JsonRpcProvider(`http://${host}:${rpc}`, {
      chainId: this.config.chainId,
      name: 'meta',
    });
  }

  getEthersWsProvider() {
    const host = this.getHost();
    const rpc = this.getMappedPort(this.config.ports.ws);
    return new ethers.providers.JsonRpcProvider(`ws://${host}:${rpc}`, {
      chainId: this.config.chainId,
      name: 'meta',
    });
  }
}
