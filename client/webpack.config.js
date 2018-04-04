
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
};
