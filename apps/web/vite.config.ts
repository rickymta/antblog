import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

// Chỉ nghe trên máy này. /api đi qua proxy tới tech-blog-api để cookie phiên cùng origin.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    proxy: { '/api': { target: process.env.TKH_API_URL ?? 'http://127.0.0.1:4000', changeOrigin: false } },
  },
  preview: { host: '127.0.0.1', port: 6173 },
});
