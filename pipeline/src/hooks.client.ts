// Client-side hooks for SvelteKit
// Ensures Tauri API is properly initialized

// Type declarations for Tauri globals
declare global {
  interface Window {
    __TAURI__?: any;
    __TAURI_INTERNALS__?: any;
  }
}

export async function handleError({ error }: { error: any; event?: any }) {
  console.error('Client error:', error);

  // Log Tauri API availability for debugging
  if (typeof window !== 'undefined') {
    console.log('Tauri API available:', !!window.__TAURI__);
    console.log('Tauri Internals available:', !!window.__TAURI_INTERNALS__);
  }

  return {
    message: error?.message || 'An error occurred',
  };
}
