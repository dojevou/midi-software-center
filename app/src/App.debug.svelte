<script lang="ts">
  import { onMount } from 'svelte';

  let step = 0;
  let error = '';
  let MenuBar: any, StatusBar: any;
  let DAWWindow: any, MixerWindow: any, DatabaseWindow: any, PipelineWindow: any;

  onMount(async () => {
    try {
      console.log('Step 1: Importing MenuBar');
      step = 1;
      MenuBar = (await import('$lib/components/MenuBar.svelte')).default;

      console.log('Step 2: Importing StatusBar');
      step = 2;
      StatusBar = (await import('$lib/components/StatusBar.svelte')).default;

      console.log('Step 3: Importing DAWWindow');
      step = 3;
      DAWWindow = (await import('$lib/windows/DAWWindow.svelte')).default;

      console.log('Step 4: Importing MixerWindow');
      step = 4;
      MixerWindow = (await import('$lib/windows/MixerWindow.svelte')).default;

      console.log('Step 5: Importing DatabaseWindow');
      step = 5;
      DatabaseWindow = (await import('$lib/windows/DatabaseWindow.svelte')).default;

      console.log('Step 6: Importing PipelineWindow');
      step = 6;
      PipelineWindow = (await import('$lib/windows/PipelineWindow.svelte')).default;

      console.log('Step 7: All components loaded!');
      step = 7;
    } catch (e: any) {
      error = e.message || String(e);
      console.error('Component loading failed:', e);
    }
  });
</script>

<div style="padding: 20px; background: #1e1e1e; color: #fff; min-height: 100vh;">
  <h1>🔍 Component Loading Debug</h1>
  <p>Current Step: {step}/7</p>

  {#if error}
    <div style="background: #e74c3c; padding: 15px; margin: 10px 0; border-radius: 5px;">
      <strong>Error:</strong> {error}
    </div>
  {/if}

  {#if step >= 1 && MenuBar}
    <div style="background: #27ae60; padding: 10px; margin: 5px 0;">✅ MenuBar loaded</div>
    <svelte:component this={MenuBar} />
  {/if}

  {#if step >= 2 && StatusBar}
    <div style="background: #27ae60; padding: 10px; margin: 5px 0;">✅ StatusBar loaded</div>
    <svelte:component this={StatusBar} />
  {/if}

  {#if step >= 3 && DAWWindow}
    <div style="background: #27ae60; padding: 10px; margin: 5px 0;">✅ DAWWindow loaded</div>
  {/if}

  {#if step >= 4 && MixerWindow}
    <div style="background: #27ae60; padding: 10px; margin: 5px 0;">✅ MixerWindow loaded</div>
  {/if}

  {#if step >= 5 && DatabaseWindow}
    <div style="background: #27ae60; padding: 10px; margin: 5px 0;">✅ DatabaseWindow loaded</div>
  {/if}

  {#if step >= 6 && PipelineWindow}
    <div style="background: #27ae60; padding: 10px; margin: 5px 0;">✅ PipelineWindow loaded</div>
  {/if}

  {#if step === 7}
    <div style="background: #3498db; padding: 15px; margin: 10px 0; border-radius: 5px;">
      <strong>🎉 All components loaded successfully!</strong>
      <p>The issue is likely in how components are rendered together.</p>
    </div>
  {/if}
</div>
