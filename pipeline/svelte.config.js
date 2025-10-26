import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://kit.svelte.dev/docs/integrations#preprocessors
  // for more information about preprocessors
  preprocess: vitePreprocess(),

  kit: {
    // adapter-static is required for Tauri
    adapter: adapter({
      // Default options for adapter-static
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),

    // Tauri expects static HTML output
    prerender: {
      handleHttpError: 'warn'
    },

    // Alias configuration
    alias: {
      $lib: 'src/lib',
      $components: 'src/lib/components',
      $stores: 'src/lib/stores',
      $utils: 'src/lib/utils',
      $types: 'src/lib/types'
    }
  },

  // Compiler options
  compilerOptions: {
    // Enable runtime checks in development
    dev: process.env.NODE_ENV === 'development'
  },

  // Vite plugin options
  vitePlugin: {
    // Inspector configuration for development
    inspector: {
      toggleKeyCombo: 'meta-shift',
      showToggleButton: 'always',
      toggleButtonPos: 'bottom-right'
    }
  }
};

export default config;
