import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react({
    babel: {
      plugins: ['babel-plugin-react-compiler'],
    },
  })],
  build: {
    outDir: 'dist',
    // Static build output for Rust backend to serve
    emptyOutDir: true,
  },
  server: {
    port: 3000,
    proxy: {
      // Proxy API requests to Rust backend during development
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
