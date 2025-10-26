<!--
  Connection Test Page
  ARCHETYPE: CONTAINER (Task-O-Matic)
  PURPOSE: Page to test database connection using ConnectionTest component
-->

<script lang="ts">
  import ConnectionTest from '$lib/components/ConnectionTest.svelte';
</script>

<svelte:head>
  <title>Database Connection Test</title>
</svelte:head>

<div class="page-container">
  <div class="page-header">
    <h1>Database Connection Test</h1>
    <p class="page-description">
      Verify the database connection and check system health.
    </p>
  </div>

  <div class="page-content">
    <ConnectionTest />
  </div>

  <div class="page-footer">
    <a href="/" class="back-link">← Back to Home</a>
  </div>
</div>

<style>
  .page-container {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--color-bg, #1a1a1a);
    color: var(--color-text, #e5e5e5);
  }

  .page-header {
    padding: var(--spacing-xl, 32px);
    text-align: center;
    border-bottom: 1px solid var(--color-border, #404040);
  }

  .page-header h1 {
    font-size: var(--font-size-3xl, 32px);
    font-weight: 700;
    margin: 0 0 var(--spacing-md, 16px) 0;
    color: var(--color-text, #e5e5e5);
  }

  .page-description {
    font-size: var(--font-size-base, 16px);
    color: var(--color-text-secondary, #a3a3a3);
    margin: 0;
  }

  .page-content {
    flex: 1;
    padding: var(--spacing-xl, 32px);
  }

  .page-footer {
    padding: var(--spacing-xl, 32px);
    text-align: center;
    border-top: 1px solid var(--color-border, #404040);
  }

  .back-link {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-sm, 8px) var(--spacing-lg, 24px);
    background: var(--color-surface, #2d2d2d);
    color: var(--color-text, #e5e5e5);
    text-decoration: none;
    border-radius: var(--radius-md, 8px);
    border: 1px solid var(--color-border, #404040);
    font-size: var(--font-size-sm, 14px);
    font-weight: 500;
    transition: all 0.2s;
  }

  .back-link:hover {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
    transform: translateY(-1px);
  }
</style>
