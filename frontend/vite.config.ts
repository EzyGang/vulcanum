import { defineConfig } from 'vite';
import preact from '@preact/preset-vite';
import tailwindcss from '@tailwindcss/vite';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [preact({
       jsxImportSource: 'preact',
     }),
     tailwindcss(),
  ],
  "server": {
    allowedHosts: ["localhost", "0.0.0.0", "127.0.0.1", "localho.st"],
    proxy: {
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
        secure: false
      }
    }
  }
});
