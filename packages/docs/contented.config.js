/** @type {import('@birthdayresearch/contented').ContentedConfig} */
const config = {
  preview: {
    name: 'DeFi Meta Chain',
    url: 'https://defimetachain.org',
    github: {
      url: 'https://github.com/defich/metachain',
    },
  },
  processor: {
    rootDir: '../../',
    pipelines: [
      {
        type: 'Meta',
        pattern: ['./README.md', './packages/meta-docs/**/*.md'],
        processor: 'md',
        fields: {
          title: {
            type: 'string',
          },
          description: {
            type: 'string',
          },
        },
        transform: (file) => {
          if (file.path === '/readme') {
            file.fields.title = 'DeFi Meta Chain';
            file.path = '/';
            file.sections = [];
          } else {
            file.path = file.path.replaceAll(/^\/packages\/meta-docs\/?/g, '/');
            file.sections = file.sections.slice(2);
          }
          return file;
        },
        sort: (a, b) => {
          return a.path === '/' ? -1 : 0;
        },
      },
    ],
  },
};

export default config;
