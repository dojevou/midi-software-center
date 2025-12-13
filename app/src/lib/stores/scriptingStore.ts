// Scripting store - Lua automation
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface ScriptAction {
  name: string;
  description: string;
  function: string;
}

export interface ScriptInfo {
  id: string;
  name: string;
  scriptType: string;
  description: string;
  author: string;
  version: string;
  actions: ScriptAction[];
}

interface ScriptingState {
  scripts: ScriptInfo[];
  activeScript: string | null;
  editorContent: string;
  consoleOutput: string[];
}

export const scriptingState = writable<ScriptingState>({
  scripts: [],
  activeScript: null,
  editorContent: '',
  consoleOutput: [],
});

export const loadedScripts = derived(scriptingState, ($s) => $s.scripts);
export const availableActions = derived(scriptingState, ($s) =>
  $s.scripts.flatMap((script) => script.actions)
);

export const scriptingActions = {
  async loadScript(name: string, code: string, scriptType: string) {
    await invoke('scripting_load_script', { name, code, scriptType });
    await this.refreshScripts();
  },

  async unloadScript(name: string) {
    await invoke('scripting_unload_script', { name });
    await this.refreshScripts();
  },

  async refreshScripts() {
    const scripts = await invoke<ScriptInfo[]>('scripting_list_scripts');
    scriptingState.update((s) => ({ ...s, scripts }));
  },

  async runAction(actionName: string) {
    await invoke('scripting_run_action', { actionName });
  },

  async getExamples(): Promise<Array<{ name: string; type: string; code: string }>> {
    const examples = await invoke<[string, string, string][]>('scripting_get_example_scripts');
    return examples.map(([name, type, code]) => ({ name, type, code }));
  },

  setEditorContent(content: string) {
    scriptingState.update((s) => ({ ...s, editorContent: content }));
  },

  appendConsole(line: string) {
    scriptingState.update((s) => ({
      ...s,
      consoleOutput: [...s.consoleOutput.slice(-99), line],
    }));
  },
};
