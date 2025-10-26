import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    // Test environment
    environment: 'jsdom',

    // Global test utilities
    globals: true,

    // Setup files
    setupFiles: [],

    // Coverage configuration
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src-tauri/',
        'build/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/mockData.ts',
        '**/__tests__/**',
      ],
      // Aim for good coverage
      thresholds: {
        lines: 70,
        functions: 70,
        branches: 70,
        statements: 70,
      },
    },

    // Include patterns
    include: ['src/**/*.{test,spec}.{js,ts}'],

    // Test timeout
    testTimeout: 10000,

    // Hooks timeout
    hookTimeout: 10000,
  },

  resolve: {
    alias: {
      // SvelteKit aliases
      $lib: path.resolve('./src/lib'),
      $app: path.resolve('./node_modules/@sveltejs/kit/src/runtime/app'),

      // Standard alias
      '@': path.resolve('./src'),
    },
  },
});
