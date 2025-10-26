import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5174,
		strictPort: true
	},
	resolve: {
		alias: {
			'$lib': path.resolve('./src/lib'),
			'$lib/*': path.resolve('./src/lib/*')
		}
	}
});
