import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],

  // Vite options tailored for Tauri development
  clearScreen: false,

  // Tauri expects a fixed port
  server: {
    port: 5173,
    strictPort: true,
    host: '0.0.0.0',
    hmr: {
      protocol: 'ws',
      host: '0.0.0.0',
      port: 5183
    }
  },

  // Environment prefix for exposing variables to client
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',

    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,

    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,

    // Chunk size warnings
    chunkSizeWarningLimit: 1000,

    rollupOptions: {
      output: {
        // Manual chunks for better caching
        manualChunks: {
          vendor: ['svelte']
          // Note: @tauri-apps/api is external and handled by Tauri
        }
      }
    }
  },

  // Optimize dependencies
  optimizeDeps: {
    include: ['@tauri-apps/api']
  },

  // Resolve configuration
  resolve: {
    alias: {
      $lib: '/src/lib',
      $components: '/src/lib/components',
      $stores: '/src/lib/stores',
      $utils: '/src/lib/utils',
      $types: '/src/lib/types'
    }
  },

  // Test configuration (for Vitest)
  test: {
    globals: true,
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{js,ts}'],
    coverage: {
      provider: 'istanbul',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src-tauri/',
        '.svelte-kit/',
        'build/'
      ]
    }
  }
});
