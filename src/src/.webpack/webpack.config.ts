const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    entry: './src/index.js',
    output: {
        filename: 'bundle.js',
        path: path.resolve(__dirname, 'dist'),
        webassemblyModuleFilename: 'module.wasm'
    },
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'webassembly/async'
            }
        ]
    },
    experiments: {
        asyncWebAssembly: true
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '.'),
            outDir: path.resolve(__dirname, 'pkg')
        })
    ]
};