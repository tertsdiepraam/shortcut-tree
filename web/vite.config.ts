import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  base: "/shortcut-tree/",
  build: {
    target: "esnext",
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        implementation: resolve(__dirname, 'implementation/index.html'),
        show: resolve(__dirname, 'show/index.html')
      }
    }
  }
});
