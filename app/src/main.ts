console.log('🚀 Starting Svelte app initialization');
import './app.css';
import App from './App.svelte';

console.log('📦 Svelte App imported, mounting to #app');
const app = new App({
  target: document.getElementById('app')!,
});
console.log('✅ Svelte app mounted successfully');

export default app;
