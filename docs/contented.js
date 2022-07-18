const Doc = {
  name: 'Doc',
  filePathPattern: `**/*.md`,
  fields: {
    title: {
      type: 'string',
      description: 'The title of the documentation.',
      required: true
    }
  }
}

export default {
  rootDir: './',
  types: [Doc],
};
