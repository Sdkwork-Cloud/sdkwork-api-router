import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const __dirname = fileURLToPath(new URL('.', import.meta.url));

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api/admin': {
        target: process.env.SDKWORK_ADMIN_PROXY_TARGET ?? 'http://127.0.0.1:8081',
        changeOrigin: true,
        rewrite: (sourcePath) => sourcePath.replace(/^\/api\/admin/, '/admin'),
      },
      '/api/portal': {
        target: process.env.SDKWORK_PORTAL_PROXY_TARGET ?? 'http://127.0.0.1:8082',
        changeOrigin: true,
        rewrite: (sourcePath) => sourcePath.replace(/^\/api\/portal/, '/portal'),
      },
      '/api/v1': {
        target: process.env.SDKWORK_GATEWAY_PROXY_TARGET ?? 'http://127.0.0.1:8080',
        changeOrigin: true,
        rewrite: (sourcePath) => sourcePath.replace(/^\/api/, ''),
      },
    },
  },
  build: {
    rollupOptions: {
      input: {
        landing: path.resolve(__dirname, 'index.html'),
        admin: path.resolve(__dirname, 'admin/index.html'),
        portal: path.resolve(__dirname, 'portal/index.html'),
      },
    },
  },
});
