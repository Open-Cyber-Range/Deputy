/** @type {import('next').NextConfig} */
const nextTranslate = require('next-translate');

module.exports = nextTranslate({
  reactStrictMode: true,
  swcMinify: true,
  env: {
    DOCUMENTATION_URL: process.env.DOCUMENTATION_URL,
  },
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:8080/api/:path*',
      },
    ];
  },
});
