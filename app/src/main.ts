console.log('🚀 Starting Svelte app initialization');
import './app.css';
console.log('✅ CSS imported successfully');
import App from './App.svelte';
console.log('📦 Svelte App imported successfully');

console.log('📦 Svelte App imported, mounting to #app');
console.log('🔍 DOM ready state:', document.readyState);
try {
  console.log('🔍 Checking if #app element exists');
  const appElement = document.getElementById('app');
  if (!appElement) {
    throw new Error('Element #app not found in DOM');
  }
  console.log('✅ #app element found');
  const app = new App({
    target: appElement,
  });
  console.log('✅ Svelte app mounted successfully');
  console.log('📊 App instance created:', app);
} catch (error) {
  console.error('❌ FATAL ERROR mounting app:');
  console.error(error);
  const errorAsAny = error as Error;
  console.error('Stack trace:', errorAsAny.stack);
  const appDiv = document.getElementById('app');
  if (appDiv) {
    const errorDiv = document.createElement('div');
    errorDiv.style.cssText =
      'background: #ff0000; color: white; padding: 20px; font-family: monospace;';
    const h1 = document.createElement('h1');
    h1.textContent = 'App Mount Error';
    const pre = document.createElement('pre');
    pre.textContent = String(error);
    errorDiv.appendChild(h1);
    errorDiv.appendChild(pre);
    appDiv.appendChild(errorDiv);
  }
  throw error;
}
