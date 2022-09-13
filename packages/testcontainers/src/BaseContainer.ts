import { NetworkConfig } from '@defimetachain/network';
import { StartOptions } from './MetaChainContainer';

export abstract class BaseContainer {
  protected constructor(public readonly config: NetworkConfig) {}

  static readonly PREFIX = 'metachain-testcontainers-';

  static get image(): string {
    if (process?.env?.METACHAIN_DOCKER_IMAGE !== undefined) {
      return process.env.METACHAIN_DOCKER_IMAGE;
    }
    return 'ghcr.io/defich/metachain:af2e7d03b061352491d550c8923d1dfac4f65095';
  }

  protected getCmd(opts: StartOptions): string[] {
    return [
      '--execution=Native', // Faster execution compare to `Wasm`
      '--no-telemetry', // disable connecting to substrate telemetry server
      '--no-prometheus', // do not expose a Prometheus exporter endpoint
      '--no-grandpa',
      `--port=${opts.port}`,
      `--rpc-port=${opts.rpcPort}`,
      `--ws-port=${opts.wsPort}`,
      '--rpc-external',
      '--ws-external',
      '--sealing=manual',
      '--force-authoring', // enable authoring even when offline
      '--rpc-cors=all',
      '--alice', // shortcut for `--name Alice --validator` with session keys for `Alice` added to keystore, required by manual sealing to author the blocks
      '--tmp', // run a temporary node,
      `-linfo`,
    ];
  }
}
