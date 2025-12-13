import { healthStore } from '$lib/stores/healthStore';
import { offlineStore } from '$lib/stores/offlineStore';

export async function initializeHealthSystem(): Promise<void> {
  // Start health monitoring
  healthStore.startMonitoring(30000); // Check every 30 seconds

  // Do initial health check
  const health = await healthStore.checkHealth();

  if (health && health.overall_status !== 'healthy') {
    console.warn('System starting with degraded/unhealthy services:', health);
  }

  // If coming online after being offline, sync pending operations
  if (health && health.overall_status === 'healthy') {
    const { synced, failed } = await offlineStore.syncPendingOperations();
    if (synced > 0 || failed > 0) {
      console.log(`Startup sync: ${synced} synced, ${failed} failed`);
    }
  }
}

export function cleanupHealthSystem(): void {
  healthStore.stopMonitoring();
}
