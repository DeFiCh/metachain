import { Keyring } from '@polkadot/api';

const keyringEth = new Keyring({ type: 'ethereum' });

// constant
export const GENESIS_ACCOUNT = '0x6be02d1d3665660d22ff9624b7be0551ee1ac91b';
export const GENESIS_ACCOUNT_BALANCE =
  '340282366920938463463374607431768210955';
export const GENESIS_ACCOUNT_PRIVATE_KEY =
  '0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342';
export const genesis = keyringEth.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY);

// META_LOG="warn,rpc=trace" npmt
export const META_LOG = process.env.META_LOG || 'info';

export const CHAIN_ID = 988;
export const BLOCK_GAS_LIMIT = 75000000;
export const BLOCK_HASH_COUNT = 2400;
export const BLOCK_TIMESTAMP = 6; // 6 seconds per block
export const EXISTENTIAL_DEPOSIT = 500; // The minimum amount required to keep an account open
export const FIRST_CONTRACT_ADDRESS =
  '0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a';
