/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index.js",
	output: {
		filename: "[name].js",
		chunkFilename: "[name].js"
	},
	optimization: {
		concatenateModules: true,
		minimize: false,
		usedExports: true,
		sideEffects: true,
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				common: {
					minChunks: 2,
					chunks: "all",
					name: "common",
					enforce: true
				}
			}
		}
	}
};
