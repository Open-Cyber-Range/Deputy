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
        source: '/api/v1/:path*',
        destination: 'http://localhost:9000/api/v1/:path*',
      },
    ];
  },
});
