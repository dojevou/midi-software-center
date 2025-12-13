<script lang="ts">
  import WindowBase from '../components/WindowBase.svelte';
  import ScriptEditor from '../components/ScriptEditor.svelte';
  import { scriptingState, scriptingActions } from '../stores/scriptingStore';
  import { onMount } from 'svelte';
  import type { WindowId } from '../types';

  export let windowId: WindowId = 'script-editor';

  let selectedExample = '';
  let examples: Array<{ name: string; type: string; code: string }> = [];

  onMount(async () => {
    examples = await scriptingActions.getExamples();
    await scriptingActions.refreshScripts();
  });

  async function loadExample() {
    const example = examples.find(e => e.name === selectedExample);
    if (example) {
      scriptingActions.setEditorContent(example.code);
    }
  }

  async function runScript() {
    const code = $scriptingState.editorContent;
    if (code) {
      try {
        await scriptingActions.loadScript('temp_script', code, 'utility');
        scriptingActions.appendConsole('Script loaded successfully');
      } catch (e) {
        scriptingActions.appendConsole(`Error: ${e}`);
      }
    }
  }
</script>

<WindowBase {windowId} title="Script Editor" minWidth={700} minHeight={500}>
  <div class="script-editor-window">
    <div class="toolbar">
      <select bind:value={selectedExample} on:change={loadExample}>
        <option value="">Load Example...</option>
        {#each examples as example}
          <option value={example.name}>{example.name} ({example.type})</option>
        {/each}
      </select>
      <button on:click={runScript}>Run Script</button>
      <button on:click={() => scriptingActions.refreshScripts()}>Refresh</button>
    </div>
    <div class="editor-area">
      <ScriptEditor />
    </div>
    <div class="console">
      <h4>Console Output</h4>
      <pre>{$scriptingState.consoleOutput.join('\n')}</pre>
    </div>
    <div class="scripts-list">
      <h4>Loaded Scripts ({$scriptingState.scripts.length})</h4>
      <ul>
        {#each $scriptingState.scripts as script}
          <li>
            <strong>{script.name}</strong> ({script.scriptType})
            <button on:click={() => scriptingActions.unloadScript(script.name)}>×</button>
          </li>
        {/each}
      </ul>
    </div>
  </div>
</WindowBase>

<style>
  .script-editor-window {
    display: grid;
    grid-template-rows: auto 1fr auto auto;
    height: 100%;
    gap: 8px;
  }
  .toolbar {
    padding: 8px;
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border-color);
  }
  .editor-area {
    flex: 1;
    min-height: 200px;
  }
  .console {
    height: 120px;
    overflow: auto;
    background: #1a1a1a;
    color: #0f0;
    padding: 8px;
    font-family: monospace;
  }
  .scripts-list {
    padding: 8px;
    max-height: 150px;
    overflow: auto;
  }
  .scripts-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .scripts-list li {
    display: flex;
    justify-content: space-between;
    padding: 4px 0;
  }
</style>
