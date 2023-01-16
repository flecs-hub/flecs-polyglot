const { config } = require('@swc/core/spack');

module.exports = config({
    entry: {
        flecs: './src/index.ts',
    },
    output: {
        filename: 'flecs.js',
        name: 'flecs.js',
        path: './dist'
    },
    target: 'browser',
    options: {
        swcrc: false,
        jsc: {
            parser: {
                syntax: 'typescript',
                tsx: false,
                jsx: false,
                decorators: true,
                dynamicImport: true,
                privateMethod: false,
                functionBind: false,
                exportDefaultFrom: false,
                exportNamespaceFrom: false,
                decoratorsBeforeExport: false,
                topLevelAwait: true,
                importMeta: false,
                preserveAllComments: false
            },
            target: 'es5',
            loose: true,
            externalHelpers: false,
            // Requires v1.2.50 or upper and requires target to be es2016 or upper.
            keepClassNames: false
        },
        minify: false
    }
})