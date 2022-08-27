export default {
  preset: 'ts-jest',
  verbose: true,
  clearMocks: true,
  testTimeout: 180000,
  transform: {
    '^.+\\.ts?$': 'ts-jest',
    '^.+\\.js?$': 'babel-jest'
  },
  transformIgnorePatterns: ['node_modules/?!(@polkadot)']
};
