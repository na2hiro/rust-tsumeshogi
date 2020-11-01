const CopyWebpackPlugin = require("copy-webpack-plugin");
const WorkerPlugin = require("worker-plugin");
const path = require('path');

console.log(process.env.NODE_ENV)

module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: process.env.NODE_ENV === 'production' ? "production" : "development",
  plugins: [
    new CopyWebpackPlugin(['index.html']),
    new WorkerPlugin()
  ],
};
