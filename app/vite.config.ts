import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte()],

  resolve: {
    alias: {
      '$lib': path.resolve(__dirname, './src/lib'),
      '@': path.resolve(__dirname, './src')
    }
  },

  // CRITICAL: Base path for Tauri
  base: './',

  clearScreen: false,

  server: {
    port: 5173,
    strictPort: true,
    host: '0.0.0.0', // Allow access from Tauri webview
  },

  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});