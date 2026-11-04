import path from 'node:path';
import { fileURLToPath } from 'node:url';
import type { NextConfig } from 'next';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.join(here, '../..');

/** Địa chỉ API mà máy chủ Next gọi tới (Server Components, rewrites). Không có `/api/v1` ở cuối. */
const apiInternal = (process.env.API_INTERNAL_URL ?? 'http://127.0.0.1:4000').replace(/\/+$/, '');

const nextConfig: NextConfig = {
  output: 'standalone',
  // Monorepo npm workspaces: truy vết tệp từ gốc repo để bản standalone mang đủ node_modules hoisted + packages/*.
  outputFileTracingRoot: repoRoot,
  // Font nhúng vào ảnh Open Graph (đọc bằng fs lúc chạy) phải có trong bản standalone.
  outputFileTracingIncludes: { '/**': ['./assets/fonts/**'] },
  turbopack: { root: repoRoot },
  transpilePackages: ['@tkh/shared', '@tkh/ui'],
  poweredByHeader: false,
  reactStrictMode: true,
  // Không để `next dev` tự sinh/ghi AGENTS.md vào thư mục app.
  agentRules: false,
  // Luôn đặt metadata (title, description, canonical, og:*) trong <head> của HTML đầu tiên — không stream xuống <body>
  // (mọi trình thu thập, kể cả loại không chạy JavaScript, đều đọc được).
  htmlLimitedBots: /.*/,
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'Referrer-Policy', value: 'strict-origin-when-cross-origin' },
          { key: 'X-Frame-Options', value: 'SAMEORIGIN' },
        ],
      },
    ];
  },
  // /api và /uploads cùng origin với trang đọc (cookie phiên đi kèm). Production: nginx chuyển thẳng tới API,
  // các rewrite này chỉ có tác dụng khi chạy Next trực tiếp (dev, `next start` không có nginx).
  async rewrites() {
    return [
      { source: '/api/:path*', destination: `${apiInternal}/api/:path*` },
      { source: '/uploads/:path*', destination: `${apiInternal}/uploads/:path*` },
    ];
  },
};

export default nextConfig;
