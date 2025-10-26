import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	plugins: [svelte()],

	// Vite options tailored for Tauri development
	clearScreen: false,

	// Tauri expects a fixed port
	server: {
		port: 5174,
		strictPort: true,
		host: '0.0.0.0',
		hmr: {
			protocol: 'ws',
			host: '0.0.0.0',
			port: 5184
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
					vendor: ['svelte', 'tone']
				}
			}
		}
	},

	// Optimize dependencies
	optimizeDeps: {
		include: ['@tauri-apps/api', 'tone']
	},

	// Resolve configuration
	resolve: {
		alias: {
			$lib: path.resolve('./src/lib'),
			$components: path.resolve('./src/lib/components'),
			$stores: path.resolve('./src/lib/stores'),
			$utils: path.resolve('./src/lib/utils'),
			$types: path.resolve('./src/lib/types')
		}
	}
});
