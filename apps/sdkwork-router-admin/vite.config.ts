import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const configDir = fileURLToPath(new URL('.', import.meta.url));
const installedUiRoot = path.join(configDir, 'node_modules', '@sdkwork', 'ui-pc-react');
const workspaceUiRoot = path.join(
  configDir,
  '..',
  '..',
  '..',
  'sdkwork-ui',
  'sdkwork-ui-pc-react',
);
const workspaceUiDistRoot = path.join(workspaceUiRoot, 'dist');
const defaultAdminProxyTarget = 'http://127.0.0.1:9981';

function resolveSdkworkUiDistPath(entryPath: string) {
  const installedCandidate = path.join(installedUiRoot, 'dist', entryPath);
  return existsSync(installedCandidate)
    ? installedCandidate
    : path.join(workspaceUiDistRoot, entryPath);
}

function resolveProxyTarget(envValue: string | undefined, fallbackTarget: string) {
  const trimmedValue = envValue?.trim();
  if (!trimmedValue) {
    return fallbackTarget;
  }

  return /^https?:\/\//i.test(trimmedValue)
    ? trimmedValue
    : `http://${trimmedValue}`;
}

const adminProxyTarget = resolveProxyTarget(
  process.env.SDKWORK_ADMIN_PROXY_TARGET ?? process.env.SDKWORK_ADMIN_BIND,
  defaultAdminProxyTarget,
);

const sharedUiEntryAliases = [
  {
    find: /^motion\/react$/,
    replacement: path.join(configDir, 'src', 'vendor', 'motion-react.tsx'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/styles\.css$/,
    replacement: resolveSdkworkUiDistPath('sdkwork-ui.css'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/theme$/,
    replacement: resolveSdkworkUiDistPath('theme.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/ui$/,
    replacement: resolveSdkworkUiDistPath('components-ui.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/ui\/feedback$/,
    replacement: resolveSdkworkUiDistPath('ui-feedback.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/patterns\/app-shell$/,
    replacement: resolveSdkworkUiDistPath('patterns-app-shell.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react\/components\/patterns\/desktop-shell$/,
    replacement: resolveSdkworkUiDistPath('patterns-desktop-shell.js'),
  },
  {
    find: /^@sdkwork\/ui-pc-react$/,
    replacement: resolveSdkworkUiDistPath('index.js'),
  },
];

function manualChunks(id: string) {
  if (!id.includes('node_modules')) {
    return undefined;
  }

  if (
    id.includes('\\react\\')
    || id.includes('/react/')
    || id.includes('\\react-dom\\')
    || id.includes('/react-dom/')
    || id.includes('\\react-router')
    || id.includes('/react-router')
    || id.includes('\\scheduler\\')
    || id.includes('/scheduler/')
    || id.includes('\\@remix-run\\router\\')
    || id.includes('/@remix-run/router/')
  ) {
    return 'react-vendor';
  }

  if (id.includes('\\@radix-ui\\') || id.includes('/@radix-ui/')) {
    return 'radix-vendor';
  }

  if (id.includes('\\lucide-react\\') || id.includes('/lucide-react/')) {
    return 'icon-vendor';
  }

  if (id.includes('\\motion\\') || id.includes('/motion/')) {
    return 'motion-vendor';
  }

  return undefined;
}

export default defineConfig({
  base: '/admin/',
  plugins: [react(), tailwindcss()],
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },
  resolve: {
    dedupe: ['react', 'react-dom'],
    alias: [
      ...sharedUiEntryAliases,
      {
        find: /^sdkwork-router-admin-apirouter$/,
        replacement: path.join(
          configDir,
          'packages',
          'sdkwork-router-admin-apirouter',
          'src',
          'index.ts',
        ),
      },
    ],
  },
  server: {
    host: '0.0.0.0',
    port: 5173,
    strictPort: true,
    proxy: {
      '/api/admin': {
        target: adminProxyTarget,
        changeOrigin: true,
        rewrite: (requestPath) => requestPath.replace(/^\/api\/admin/, '/admin'),
      },
    },
  },
  preview: {
    host: '0.0.0.0',
    port: 4173,
    strictPort: true,
  },
});
