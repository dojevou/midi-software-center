console.log('🚀 MAIN.TS EXECUTING');

async function initApp() {
  try {
    console.log('📦 Importing app.css');
    await import('./app.css');

    console.log('📦 Importing App component');
    const { default: App } = await import('./App.minimal.svelte');

    console.log('🎯 Getting #app element');
    const target = document.getElementById('app');
    console.log('Element found:', target);

    if (!target) {
      throw new Error('No #app element found!');
    }

    console.log('🔨 Creating Svelte app');
    const app = new App({ target });

    console.log('✅ SVELTE APP MOUNTED SUCCESSFULLY');
    return app;
  } catch (error) {
    console.error('❌ FATAL ERROR:', error);
    const errorDiv = document.createElement('div');
    errorDiv.style.cssText = 'padding: 50px; background: #e74c3c; color: white;';

    const h1 = document.createElement('h1');
    h1.textContent = 'Fatal Error';

    const pre = document.createElement('pre');
    pre.textContent = String(error);

    errorDiv.appendChild(h1);
    errorDiv.appendChild(pre);
    document.body.appendChild(errorDiv);

    throw error;
  }
}

const app = await initApp();
export default app;
