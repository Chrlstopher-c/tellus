import { resolve } from 'node:path';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'electron-vite';
import { loadEnv } from 'vite';

const ROOT = process.cwd();
const DEV_PORT = Number(loadEnv('development', ROOT, '').VITE_PORT ?? 5173);

export default defineConfig({
  main: {
    build: {
      outDir: resolve(ROOT, 'out/main'),
      rollupOptions: {
        input: { index: resolve(ROOT, 'desktop/main.ts'), sim: resolve(ROOT, 'desktop/sim-process.ts') },
      },
    },
  },
  preload: {
    build: {
      outDir: resolve(ROOT, 'out/preload'),
      rollupOptions: { input: { index: resolve(ROOT, 'desktop/preload.ts') } },
    },
  },
  renderer: {
    root: resolve(ROOT, 'interface'),
    plugins: [react()],
    server: { port: DEV_PORT, strictPort: true },
    build: {
      outDir: resolve(ROOT, 'out/renderer'),
      emptyOutDir: true,
      rollupOptions: { input: { index: resolve(ROOT, 'interface/index.html') } },
    },
  },
});
