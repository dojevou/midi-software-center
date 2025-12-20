# Day 4, Part 3B: Effects Frontend

**Duration:** 2 hours
**Prerequisites:** Part 3A complete
**Files to create:** 2

---

## Overview

Create effect rack UI:
1. Effect selector (add effects)
2. Effect rack component (list with controls)
3. Parameter controls
4. Drag-and-drop reordering

---

## Step 1: TypeScript Types (15 min)

Add to `app/src/lib/types/daw.ts`:

```typescript
export type EffectType =
  | 'Gain'
  | 'EQ'
  | 'Reverb'
  | 'Delay'
  | 'Compressor'
  | 'Limiter'
  | { Custom: string };

export type ParameterValue =
  | { type: 'Float'; value: number }
  | { type: 'Int'; value: number }
  | { type: 'Bool'; value: boolean }
  | { type: 'String'; value: string };

export interface EffectParameter {
  id: string;
  name: string;
  value: ParameterValue;
  default: ParameterValue;
  min?: ParameterValue;
  max?: ParameterValue;
  unit?: string;
}

export interface Effect {
  id: string;
  effectType: EffectType;
  name: string;
  enabled: boolean;
  parameters: Record<string, EffectParameter>;
}

export interface EffectChain {
  trackId: number;
  effects: Effect[];
}

// Helper to get parameter float value
export function getFloatParameter(param: EffectParameter): number | null {
  if (param.value.type === 'Float') {
    return param.value.value;
  }
  return null;
}

// Helper to get parameter bool value
export function getBoolParameter(param: EffectParameter): boolean | null {
  if (param.value.type === 'Bool') {
    return param.value.value;
  }
  return null;
}
```

---

## Step 2: Effects API (15 min)

Create `app/src/lib/api/effectsApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Effect, EffectChain, EffectType, ParameterValue } from '../types/daw';

export class EffectsApi {
  static async addEffect(
    trackId: number,
    effectId: string,
    effectType: EffectType,
    name: string
  ): Promise<Effect> {
    return await invoke<Effect>('add_effect', {
      trackId,
      effectId,
      effectType,
      name,
    });
  }

  static async removeEffect(trackId: number, effectId: string): Promise<Effect> {
    return await invoke<Effect>('remove_effect', { trackId, effectId });
  }

  static async moveEffect(
    trackId: number,
    effectId: string,
    newIndex: number
  ): Promise<void> {
    await invoke('move_effect', { trackId, effectId, newIndex });
  }

  static async setEffectEnabled(
    trackId: number,
    effectId: string,
    enabled: boolean
  ): Promise<void> {
    await invoke('set_effect_enabled', { trackId, effectId, enabled });
  }

  static async setEffectParameter(
    trackId: number,
    effectId: string,
    paramId: string,
    value: ParameterValue
  ): Promise<void> {
    await invoke('set_effect_parameter', { trackId, effectId, paramId, value });
  }

  static async getEffectChain(trackId: number): Promise<EffectChain> {
    return await invoke<EffectChain>('get_effect_chain', { trackId });
  }

  static async getEffect(trackId: number, effectId: string): Promise<Effect> {
    return await invoke<Effect>('get_effect', { trackId, effectId });
  }
}
```

---

## Step 3: Effect Rack Component (1 hour)

Create `app/src/lib/components/DAW/EffectRack.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { EffectsApi } from '../../api/effectsApi';
  import { getFloatParameter, getBoolParameter } from '../../types/daw';
  import type { Effect, EffectChain } from '../../types/daw';

  export let trackId: number;

  let effectChain: EffectChain | null = null;
  let showAddEffect = false;
  let draggedEffectId: string | null = null;

  onMount(async () => {
    await loadEffectChain();
  });

  async function loadEffectChain() {
    try {
      effectChain = await EffectsApi.getEffectChain(trackId);
    } catch (error) {
      console.error('Failed to load effect chain:', error);
    }
  }

  async function handleAddEffect(effectType: string) {
    const effectId = `${effectType.toLowerCase()}-${Date.now()}`;
    const name = effectType;

    try {
      await EffectsApi.addEffect(trackId, effectId, effectType as any, name);
      await loadEffectChain();
      showAddEffect = false;
    } catch (error) {
      console.error('Failed to add effect:', error);
    }
  }

  async function handleRemoveEffect(effectId: string) {
    if (!confirm('Remove this effect?')) return;

    try {
      await EffectsApi.removeEffect(trackId, effectId);
      await loadEffectChain();
    } catch (error) {
      console.error('Failed to remove effect:', error);
    }
  }

  async function handleToggleEffect(effectId: string, currentState: boolean) {
    try {
      await EffectsApi.setEffectEnabled(trackId, effectId, !currentState);
      await loadEffectChain();
    } catch (error) {
      console.error('Failed to toggle effect:', error);
    }
  }

  async function handleParameterChange(
    effectId: string,
    paramId: string,
    value: number
  ) {
    try {
      await EffectsApi.setEffectParameter(trackId, effectId, paramId, {
        type: 'Float',
        value,
      });
      // Optimistically update local state
      if (effectChain) {
        const effect = effectChain.effects.find((e) => e.id === effectId);
        if (effect && effect.parameters[paramId]) {
          effect.parameters[paramId].value = { type: 'Float', value };
        }
      }
    } catch (error) {
      console.error('Failed to set parameter:', error);
    }
  }

  // Drag and drop
  function handleDragStart(e: DragEvent, effectId: string) {
    draggedEffectId = effectId;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  async function handleDrop(e: DragEvent, targetEffectId: string) {
    e.preventDefault();
    if (!draggedEffectId || !effectChain) return;

    const targetIndex = effectChain.effects.findIndex((e) => e.id === targetEffectId);

    try {
      await EffectsApi.moveEffect(trackId, draggedEffectId, targetIndex);
      await loadEffectChain();
    } catch (error) {
      console.error('Failed to move effect:', error);
    }

    draggedEffectId = null;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
  }
</script>

<div class="effect-rack">
  <div class="rack-header">
    <h3>Effects - Track {trackId}</h3>
    <button class="add-btn" on:click={() => (showAddEffect = !showAddEffect)}>
      {showAddEffect ? '✕' : '+'} Add Effect
    </button>
  </div>

  {#if showAddEffect}
    <div class="effect-selector">
      <button on:click={() => handleAddEffect('Gain')}>Gain</button>
      <button on:click={() => handleAddEffect('EQ')}>EQ</button>
      <button on:click={() => handleAddEffect('Reverb')}>Reverb</button>
      <button on:click={() => handleAddEffect('Delay')}>Delay</button>
      <button on:click={() => handleAddEffect('Compressor')}>Compressor</button>
      <button on:click={() => handleAddEffect('Limiter')}>Limiter</button>
    </div>
  {/if}

  {#if effectChain}
    {#if effectChain.effects.length === 0}
      <p class="empty-state">No effects. Click "+ Add Effect" to get started.</p>
    {:else}
      <div class="effect-list">
        {#each effectChain.effects as effect (effect.id)}
          <div
            class="effect-item"
            class:disabled={!effect.enabled}
            draggable="true"
            on:dragstart={(e) => handleDragStart(e, effect.id)}
            on:drop={(e) => handleDrop(e, effect.id)}
            on:dragover={handleDragOver}
          >
            <div class="effect-header">
              <button
                class="power-btn"
                class:active={effect.enabled}
                on:click={() => handleToggleEffect(effect.id, effect.enabled)}
                title={effect.enabled ? 'Bypass' : 'Enable'}
              >
                {effect.enabled ? '●' : '○'}
              </button>

              <span class="effect-name">{effect.name}</span>

              <button
                class="remove-btn"
                on:click={() => handleRemoveEffect(effect.id)}
                title="Remove effect"
              >
                ✕
              </button>
            </div>

            <div class="effect-parameters">
              {#each Object.values(effect.parameters) as param (param.id)}
                {#if param.value.type === 'Float'}
                  {@const value = getFloatParameter(param)}
                  {@const min = param.min?.type === 'Float' ? param.min.value : 0}
                  {@const max = param.max?.type === 'Float' ? param.max.value : 1}

                  <div class="parameter">
                    <label for={param.id}>{param.name}</label>
                    <input
                      id={param.id}
                      type="range"
                      min={min}
                      max={max}
                      step={(max - min) / 100}
                      value={value ?? 0}
                      on:input={(e) =>
                        handleParameterChange(
                          effect.id,
                          param.id,
                          parseFloat(e.currentTarget.value)
                        )}
                    />
                    <span class="param-value">
                      {value?.toFixed(1)}{param.unit || ''}
                    </span>
                  </div>
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <p class="loading">Loading effects...</p>
  {/if}
</div>

<style>
  .effect-rack {
    background: #1a1a1a;
    border-radius: 8px;
    padding: 16px;
    min-width: 300px;
  }

  .rack-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .rack-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .add-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
  }

  .add-btn:hover {
    background: #2563eb;
  }

  .effect-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    margin-bottom: 16px;
    padding: 12px;
    background: #252525;
    border-radius: 4px;
  }

  .effect-selector button {
    padding: 8px;
    background: #333;
    color: #fff;
    border: 1px solid #444;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .effect-selector button:hover {
    background: #3a3a3a;
    border-color: #555;
  }

  .effect-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .effect-item {
    background: #252525;
    border: 1px solid #333;
    border-radius: 6px;
    padding: 12px;
    cursor: move;
  }

  .effect-item.disabled {
    opacity: 0.5;
  }

  .effect-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
  }

  .power-btn {
    width: 28px;
    height: 28px;
    border: 2px solid #444;
    border-radius: 50%;
    background: #333;
    color: #666;
    font-size: 16px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .power-btn.active {
    border-color: #22c55e;
    color: #22c55e;
  }

  .effect-name {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
    color: #fff;
  }

  .remove-btn {
    width: 24px;
    height: 24px;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
  }

  .remove-btn:hover {
    background: #b91c1c;
  }

  .effect-parameters {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .parameter {
    display: grid;
    grid-template-columns: 1fr 2fr auto;
    align-items: center;
    gap: 8px;
  }

  .parameter label {
    font-size: 12px;
    color: #999;
  }

  .parameter input[type='range'] {
    width: 100%;
  }

  .param-value {
    font-size: 12px;
    color: #3b82f6;
    min-width: 60px;
    text-align: right;
    font-weight: 600;
  }

  .empty-state,
  .loading {
    padding: 24px;
    text-align: center;
    color: #666;
  }
</style>
```

---

## Step 4: Integration (20 min)

Add effect rack to mixer panel or create separate effects view:

```svelte
<!-- In MixerPanel.svelte or separate EffectsPanel.svelte -->
<script lang="ts">
  import EffectRack from './EffectRack.svelte';

  let selectedTrackId = 1; // Track selected for effects editing
</script>

<div class="effects-panel">
  <EffectRack trackId={selectedTrackId} />
</div>
```

---

## Verification (10 min)

```bash
npm run check
make dev
```

Test effect rack:
1. Click "+ Add Effect" and select an effect type
2. Adjust parameter sliders
3. Toggle effect on/off (power button)
4. Drag effects to reorder
5. Remove effects
6. Verify parameters persist across reloads

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Effects not appearing | Check effect chain API response, verify backend integration |
| Parameters not updating | Verify API calls, check parameter value types |
| Drag-and-drop not working | Check dragstart/drop event handlers |
| UI not refreshing | Call `loadEffectChain()` after mutations |

---

## What's Next?

✅ **Day 4 Complete! You've built:**
- Effect chain backend with 6 effect types
- 7 Tauri commands for effect management
- Effect rack UI with parameter controls
- Drag-and-drop reordering

**Next:** [Day 5, Part 4A: Routing Backend](./DAY5_PART_A_ROUTING_BACKEND.md)
- Audio routing matrix
- Aux sends and returns
- Bus management
- Routing graph
