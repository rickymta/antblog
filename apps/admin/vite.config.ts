import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

// Chỉ nghe trên máy này. /api đi qua proxy tới API (thư mục api/) để cookie phiên cùng origin.
// Bản Docker build với TKH_ADMIN_BASE=/admin/ (nginx phục vụ trang quản trị dưới /admin/).
export default defineConfig({
  base: process.env.TKH_ADMIN_BASE ?? '/',
  plugins: [react(), tailwindcss()],
  server: {
    host: '127.0.0.1',
    port: 5174,
    strictPort: true,
    proxy: { '/api': { target: process.env.TKH_API_URL ?? 'http://127.0.0.1:4000', changeOrigin: false } },
  },
  preview: { host: '127.0.0.1', port: 6174 },
});
