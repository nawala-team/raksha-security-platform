/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'standalone',
  env: {
    API_BASE_URL: process.env.API_BASE_URL || 'http://localhost:3001/api',
  },
};

module.exports = nextConfig;
