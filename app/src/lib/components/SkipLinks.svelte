<script lang="ts">
  import { a11yStore } from '$lib/stores/a11yStore';

  interface SkipLink {
    id: string;
    label: string;
  }

  export let links: SkipLink[] = [
    { id: 'main-content', label: 'Skip to main content' },
    { id: 'main-navigation', label: 'Skip to navigation' },
  ];
</script>

<nav class="skip-links" aria-label="Skip navigation">
  {#each links as link (link.id)}
    <a
      href="#{link.id}"
      class="skip-link"
      on:click|preventDefault={() => a11yStore.skipToContent(link.id)}
    >
      {link.label}
    </a>
  {/each}
</nav>

<style>
  .skip-links {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 100001;
  }

  .skip-link {
    position: absolute;
    top: -100%;
    left: 0;
    padding: 12px 24px;
    background: var(--accent);
    color: var(--text-inverse);
    text-decoration: none;
    font-weight: 600;
    border-radius: 0 0 var(--border-radius) 0;
    transition: top 0.2s ease;
  }

  .skip-link:focus {
    top: 0;
    outline: 3px solid var(--warning);
    outline-offset: 2px;
  }
</style>
