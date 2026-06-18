import { defineConfig, loadEnv } from 'vite';
import preact from '@preact/preset-vite';
import tailwindcss from '@tailwindcss/vite';

// https://vitejs.dev/config/
export default defineConfig(({mode}) => {
  const env = loadEnv(mode, process.cwd(), '');

  return {
    plugins: [preact({
      jsxImportSource: 'preact',
    }),
    tailwindcss(),
    ],
    "server": {
      allowedHosts: ["localhost", "0.0.0.0", "127.0.0.1", "localho.st"],
      proxy: {
        '/api': {
          target: env.VITE_API_TARGET || 'http://localhost:8000',
          changeOrigin: true,
          secure: false
        }
      }
    },
    resolve: {
      alias: {
        'react': 'preact/compat',
        'react-dom': 'preact/compat',
      }
    }
  }
});
