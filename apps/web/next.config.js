/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'standalone',
  // Termux runs on Android/ARM64, for which Next.js has no native SWC binary.
  // Use the official WebAssembly compiler fallback instead.
  experimental: {
    useWasmBinary: true,
  },
  async rewrites() {
    // For local dev use localhost, for Docker use portal hostname
    const portalUrl = process.env.PORTAL_API_URL || 'http://127.0.0.1:8080';

    return [
      {
        source: '/api/:path*',
        destination: `${portalUrl}/api/:path*`,
      },
    ];
  },
};

module.exports = nextConfig;
