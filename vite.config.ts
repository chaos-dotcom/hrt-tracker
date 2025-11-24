import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
const root = fileURLToPath(new URL('.', import.meta.url));
export default defineConfig({
  plugins: [sveltekit(), tailwindcss()],
  resolve: {
    alias: {
      '@estrannaise': resolve(root, 'vendor/estrannaise/src')
    }
  },
  server: {
    fs: {
      allow: [
        resolve(root, 'vendor'),
        resolve(root, 'src'),
        resolve(root, '.svelte-kit'),
        resolve(root, 'node_modules')
      ]
    }
  }
});
