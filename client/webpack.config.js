
const path = require('path');

const isProd = process.env.NODE_ENV === 'production';

module.exports = {
  entry: {
    app: './src/App.bs.js',
  },
  mode: isProd ? 'production' : 'development',
  output: {
    path: path.join(__dirname, "dist"),
    filename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          "style-loader",
          {
            loader: "css-loader",
            options: {
              modules: true,
              importLoaders: 1,
              localIdentName: "[path]__[local]__[hash:base64:5]",
            },
          },
          {
            loader: "postcss-loader",
            options: {
              plugins: [require("postcss-preset-env")],
            },
          },
        ],
      },
    ],
  },
};
